# spmc-waker

[![Crates.io](https://img.shields.io/crates/v/spmc-waker.svg)](https://crates.io/crates/spmc-waker)
[![Documentation](https://docs.rs/spmc-waker/badge.svg)](https://docs.rs/spmc-waker)
[![CI](https://github.com/wyfo/spmc-waker/actions/workflows/ci.yml/badge.svg)](https://github.com/wyfo/spmc-waker/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue.svg)](https://github.com/wyfo/spmc-waker#license)

A wait-free synchronization primitive for task wakeup.

## Features

#### Customizable synchronization

By default, `SpmcWaker::wake` always synchronizes with `SpmcWaker::register`, same as `AtomicWaker`.

However, `SpmcWaker` provides a generic `S: Synchronization` parameter, which allows customizing its synchronization guarantees, for example to relax them if the wake condition is already synchronized enough by itself. It can reduce `wake` to a simple atomic load when no waker is registered, which can bring a significant performance gain in some workflows.

#### Waker caching

Most of the time, `SpmcWaker` is used in a single task, so the waker registered is always the same. That's why it provides a second generic parameter `CACHING`.

With `CACHING=true`, the latest waker registered is kept cached to avoid cloning on the next registration. As a consequence, tasks are woken with `Waker::wake_by_ref` instead of `Waker::wake`.

As wakers are often `Arc`s, caching avoids atomic RMW operations updating the reference counter. However, it adds an RMW operation to `SpmcWaker::wake`, so the benefit mostly concerns `SpmcWaker::register`.

#### Non-concurrent registration optional enforcement

SPMC means `SpmcWaker` supports a single thread registering a waker at a time. Trying to register multiple wakers concurrently may panic (or not depending on the actual operations ordering).

However, this behavior is configurable with a third generic parameter `R: RegistrationPolicy`. It is notably possible to disable the check for concurrent registration with `R=Unchecked`, saving some atomic operations at the cost of registration methods becoming unsafe.

#### Progress guarantee

Without taking in account `Waker` operations (clone/wake/drop), every `SpmcWaker` operation is wait-free, i.e. it terminates in a bounded number of operations.

`SpmcWaker` contains no loops and does not reschedule the task while waiting for another operation to complete, unlike `AtomicWaker` (which is not lock-free).

#### Cold path outlining

Some algorithms require `SpmcWaker::wake` to be called in hot path, even if there is no waker registered most of the time. This is exactly the use case for `SpmcWaker::wake_cold`, which starts by checking if a waker is registered before outlining the wake code in a cold function. 

More generally, each `SpmcWaker`'s method is carefully implemented to be inlinable in a dozen or less assembly instructions. Assembly scrutiny even allowed to spot an [issue](https://github.com/rust-lang/rust/issues/159335) in rustc.
  
#### Unwind-safety

`SpmcWaker` is unwind-safe, i.e. every waker operation (clone/wake/drop) may panic, but it doesn't break `SpmcWaker` internal invariants.

## Example

```rust
use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::Relaxed},
    },
    task::{Context, Poll},
};

use spmc_waker::SpmcWaker;

#[derive(Default)]
struct Inner {
    notified: AtomicBool,
    waker: SpmcWaker,
}

#[derive(Clone)]
struct Notifier(Arc<Inner>);

impl Notifier {
    pub fn new() -> Self {
        Self(Arc::new(Inner {
            waker: SpmcWaker::new(),
            notified: AtomicBool::new(false),
        }))
    }

    pub fn signal(&self) {
        self.0.notified.store(true, Relaxed);
        self.0.waker.wake();
    }
}

#[derive(Default)]
struct Waiter(Arc<Inner>);

impl Waiter {
    fn notifier(&self) -> Notifier {
        Notifier(self.0.clone())
    }
}

impl Future for Waiter {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let waker = &self.0.waker;
        waker.poll_wait_until(cx, |_| self.0.notified.load(Relaxed))
    }
}

fn event() -> (Notifier, Waiter) {
    let waiter = Waiter::default();
    (waiter.notifier(), waiter)
}
```

## Performance

See [benchmark results](benches/README.md). The following table compares the atomic operations of `register` and `wake` methods for the different primitives.

|                         | `AtomicWaker`              | `SpmcWaker<Synchronized>` (default)           | `SpmcWaker<Sequential>`          | `SpmcWaker<Unsynchronized>`                   |
|-------------------------|----------------------------|-----------------------------------------------|----------------------------------|-----------------------------------------------|
| register                | RMW(Acquire) + RMW(AcqRel) | RMW(Relaxed)[^1] + RMW(AcqRel)                | RMW(Relaxed)[^1] + store(SeqCst) | RMW(Relaxed)[^1] + store(Release)             |
| wake (waker registered) | RMW(AcqRel) + RMW(Release) | load(Relaxed) + fence(Acquire) + RMW(Release) | load(SeqCst) + RMW(Release)      | load(Relaxed) + fence(Acquire) + RMW(Release) |
| wake (no waker)         | RMW(AcqRel) + RMW(Release) | load(Relaxed) + RMW(Release)[^2]              | load(SeqCst)                     | load(Relaxed)                                 |

[^1]: the RMW is only present for safe registration policies, i.e. `R=Strict`/`R=Lenient`; with `R=Unchecked` and unsafe `register`, it is replaced by `load(Relaxed)`.

[^2]: the RMW is a `fetch_add(0)`, which is equivalent **on x86** to a `SeqCst` fence; as a consequence (still on x86), it doesn't touch the `SpmcWaker` cache line, which stays read-only and uncontended.

Compared to `AtomicWaker`, `SpmcWaker` reduces the number of RMW operations in all operations. `Sequential` and `Unsynchronized` variants go even further by reducing `wake` to single atomic load when no waker is registered.

Atomic operations related to waker cloning/dropping are not counted in the table. Waker caching can eliminate them in `SpmcWaker`, at the cost of an additional `RMW(Release)` in `wake`.

As illustrated in the example, `SpmcWaker` is designed to be used in MPSC algorithms, i.e. one waiter registering its waker with multiple notifiers. In an MPSC channel case with some throughput, the receiver waker is rarely registered, as there are more often already items waiting in the queue. However, the receiver waker is systematically woken by producers, so optimizing `wake` when there is no waker registered becomes the most important. `SpmcWaker` algorithm was built with this goal in mind.

### Replacing `AtomicWaker` in `tokio::sync::mpsc`

`tokio::sync::mpsc` uses `AtomicWaker` internally (actually, it uses a custom implementation with better panic handling but the exact same algorithm), calling `AtomicWaker::wake` in `send` hot path. As a result, it systematically pays two contended RMWs.

Replacing `AtomicWaker` with `SpmcWaker<Synchronized>` removes one RMW from the hot path (the other one is uncontended on x86). Because the mpsc receiver is a single consumer, registration is never concurrent, so the `Unchecked` registration policy is used with unsafe block around registration.

Also, because the wake condition is already set with a `Release` RMW, `SpmcWaker<Unsynchronized>` variant can be used by replacing the RMW ordering with `AcqRel` (and checking the wake condition with an `AcqRel` RMW instead of an `Acquire` load, but only after registering the waker). As a consequence, it removes the second RMW from the hot path when no waker is registered, which is significant on x86.

The following table presents the results of tokio's own mpsc benchmark depending on the atomic waker used. These results come with an important caveat: channel benchmarks are often dominated by cache-line contention, whose effects are hardly predictable; a "faster" algorithm may sometimes lead to more contention and give disastrous results in benchmarks. Also, the atomic waker is not the main part of the whole MPSC channel algorithm. Still, the replacement of `AtomicWaker` seems to produce a noticeable effect on the results.

| Benchmark                         | `AtomicWaker`        | `SpmcWaker<Synchronized>` | `SpmcWaker<Synchronized, true>` | `SpmcWaker<Unsynchronized>` | `SpmcWaker<Unsynchronized, true>` |
|-----------------------------------|----------------------|---------------------------|---------------------------------|-----------------------------|-----------------------------------|
| contention/bounded                | 725.0 µs (baseline)  | 684.9 µs (-6%)            | 685.1 µs (-6%)                  | 657.7 µs (-9%)              | 666.6 µs (-8%)                    |
| contention/bounded_recv_many      | 665.5 µs (baseline)  | 595.5 µs (-11%)           | 593.8 µs (-11%)                 | 561.2 µs (-16%)             | 572.6 µs (-14%)                   |
| contention/bounded_full           | 1227.6 µs (baseline) | 1258.2 µs (+2%)           | 1279.5 µs (+4%)                 | 1220.3 µs (-1%)             | 1236.6 µs (+1%)                   |
| contention/bounded_full_recv_many | 665.6 µs (baseline)  | 592.4 µs (-11%)           | 601.3 µs (-10%)                 | 568.8 µs (-15%)             | 571.0 µs (-14%)                   |
| contention/unbounded              | 698.4 µs (baseline)  | 597.6 µs (-14%)           | 611.4 µs (-12%)                 | 569.6 µs (-18%)             | 576.9 µs (-17%)                   |
| contention/unbounded_recv_many    | 679.6 µs (baseline)  | 614.8 µs (-10%)           | 601.2 µs (-12%)                 | 571.5 µs (-16%)             | 560.2 µs (-18%)                   |
| uncontented/bounded               | 402.6 µs (baseline)  | 378.4 µs (-6%)            | 390.0 µs (-3%)                  | 376.6 µs (-6%)              | 359.1 µs (-11%)                   |
| uncontented/bounded_recv_many     | 286.3 µs (baseline)  | 271.2 µs (-5%)            | 261.5 µs (-9%)                  | 229.0 µs (-20%)             | 240.3 µs (-16%)                   |
| uncontented/unbounded             | 231.5 µs (baseline)  | 215.3 µs (-7%)            | 221.0 µs (-5%)                  | 179.9 µs (-22%)             | 178.7 µs (-23%)                   |
| uncontented/unbounded_recv_many   | 211.1 µs (baseline)  | 177.3 µs (-16%)           | 170.5 µs (-19%)                 | 134.9 µs (-36%)             | 132.1 µs (-37%)                   |

## Safety

This crate uses unsafe code, as well as exposing unsafe methods. It is extensively tested with both [`miri`](https://github.com/rust-lang/miri) and [`loom`](https://github.com/tokio-rs/loom), including tests adapted from `AtomicWaker`. 

Each memory ordering for atomic operations is carefully chosen and tested; downgrading any of them makes the test suite fail. 

## Comparison with `DiatomicWaker`

`DiatomicWaker` is an alternative to `AtomicWaker`, that aims to be faster and better suited for an MPSC channel — the same goal as `SpmcWaker` in the end.
 
While discovered after starting the project, it was a great inspiration, not for its algorithm, but for its innovative ideas which `SpmcWaker` took up:
- assuming an SPMC algorithm with unsafe methods
- waker caching
- wait-free (non-spinning) registration

However, `SpmcWaker`'s algorithm was mainly focused on making `wake_cold` as light as possible; that's why the initial algorithm used `SeqCst` (`S=Sequential` variant). `DiatomicWaker` isn't optimized for this use case.

All in all, `SpmcWaker` pushes algorithm (fully wait-free, less storage used), customization (generic synchronization, etc.) and performance a lot further than `DiatomicWaker`, as shown in [benchmarks](benches/README.md).

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)

at your option.