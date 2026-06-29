# spmc-waker

[![Crates.io](https://img.shields.io/crates/v/spmc-waker.svg)](https://crates.io/crates/spmc-waker)
[![Documentation](https://docs.rs/spmc-waker/badge.svg)](https://docs.rs/spmc-waker)
[![CI](https://github.com/wyfo/spmc-waker/actions/workflows/ci.yml/badge.svg)](https://github.com/wyfo/spmc-waker/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue.svg)](https://github.com/wyfo/spmc-waker#license)

A lock-free synchronization primitive for task wakeup.

## Features

#### SPMC (single-producer, multi-consumer)

Contrary to `AtomicWaker`, which is MPMC while explicitly advising against it, `SpmcWaker` assumes to be SPMC: `SpmcWaker::register` cannot be called concurrently. It allows algorithm optimizations, but requires the method to be *unsafe*.

#### Rich and high-level API

In addition to basic `SpmcWaker::register`, this crate provides `SpmcWaker::try_register`, which early returns if a concurrent `SpmcWaker::wake`, as the wake condition is probably fulfilled. On the other hand, `SpmcWaker::unregister` can prevent a registered waker to be called, avoiding spurious wakeup of the task.

However, the advised way to do waker registration is through `SpmcWaker::wait_until` async function, which calls `SpmcWaker::poll_wait_until`. This method combines `try_register`, `register`, and `unregister` with wake condition check for an optimal workflow. 

#### Customizable synchronization

By default, `SpmcWaker::wake` always synchronizes with `SpmcWaker::register`, same as `AtomicWaker`.

However, `SpmcWaker` provides a generic `S: Synchronization` parameter, which allows customizing its synchronization guarantees, for example to relax them if the wake condition is already synchronized enough by itself. It can reduce `wake` to a simple atomic load when no waker is registered, which can bring a significant performance gain in some workflows.

#### Waker caching

Most of the time, there is a single task registering its waker, so the waker is always the same. That's why `SpmcWaker` provides a second generic parameter `CACHED`. By default (`CACHED=true`), the registered waker is cached, i.e. it's not dropped when `SpmcWaker::wake` is called, using `Waker::wake_by_ref`. So there is no need to clone it when the same waker is registered again. As wakers are often `Arc`s, caching avoids atomic RMW operations updating the reference counter.

#### Progress guarantee

Waker registration is wait-free, while task waking is lock-free (without taking into account waker clone/wake/drop operations).

When waker registration is only done through `try_register`, `wake` becomes wait-free, but registration then requires spinning until it succeeds. This is by the way the workflow used by `AtomicWaker`, which reschedule the task as spinning mechanism, and is thus not lock-free.

#### Cold path outlining

Some algorithms require `SpmcWaker::wake` to be called in hot path, even if there is no waker registered most of the time. This is exactly the use case for `SpmcWaker::wake_cold`, which starts by checking if a waker is registered before outlining the wake code in a cold function. It's also possible to check if a waker is registered with `SpmcWaker::has_waker_registered` before calling `SpmcWaker::wake`; even if no waker is registered, `has_waker_registered` still synchronizes with `SpmcWaker::register` with default synchronization.

More generally, each `SpmcWaker`'s method is carefully implemented to be inlinable in a dozen or less assembly instructions.
  
#### Unwind-safety

If any waker operation (clone/wake/wake_by_ref/drop) panics, `SpmcWaker` state is cleaned up to an unregistered state.

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
        // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
        unsafe { waker.poll_wait_until(cx, || self.0.notified.load(Relaxed)) }
    }
}

fn event() -> (Notifier, Waiter) {
    let waiter = Waiter::default();
    (waiter.notifier(), waiter)
}
```

## Performance

See [benchmark results](benches/README.md). The following table compares the atomic operations of `register` and `wake` methods for the different primitives.

|                              | `AtomicWaker`              | `SpmcWaker<Synchronized>` (default)        | `SpmcWaker<Sequential>`                    | `SpmcWaker<Unsynchronized>`                 |
|------------------------------|----------------------------|--------------------------------------------|--------------------------------------------|---------------------------------------------|
| register                     | RMW(Acquire) + RMW(AcqRel) | load(Relaxed) + RMW(Acquire)               | load(Relaxed) + store(SeqCst)              | load(Relaxed) + store(Release)              |
| wake_cold (waker registered) | RMW(AcqRel) + RMW(Release) | load(Relaxed) + RMW(AcqRel) + RMW(Release) | load(SeqCst) + RMW(Acquire) + RMW(Release) | load(Relaxed) + RMW(Acquire) + RMW(Release) |
| wake_cold (no waker)         | RMW(AcqRel) + RMW(Release) | load(Relaxed) + RMW(Release)               | load(SeqCst)                               | load(Relaxed)                               |

Compared to `AtomicWaker`, `SpmcWaker` reduces the number of RMW operations for `register`, and for `wake_cold` when there is no waker. `SpmcWaker<false>` goes even further, reducing `wake`/`wake_cold` to a single atomic load when there is no waker registered. 

Atomic operations related to waker cloning/dropping are not counted in the table. As `SpmcWaker` caches the waker, these operations don't add overhead, but for `AtomicWaker`, an additional RMW(Relaxed) for `register`, as well as an RMW(Acquire) for `wake_cold` (waker registered) should be expected.

As illustrated in the example, `SpmcWaker` is designed to be used in MPSC algorithms, i.e. one waiter registering its waker with multiple notifiers. In an MPSC channel case with some throughput, the receiver waker is rarely registered, as there are more often already items waiting in the queue. However, the receiver waker is systematically woken by producers, so optimizing `wake` when there is no waker registered becomes the most important. `SpmcWaker` algorithm was built with this goal in mind.

### Replacing `AtomicWaker` in `tokio::sync::mpsc`

`tokio::sync::mpsc` uses `AtomicWaker` internally (actually, it uses a custom implementation with better panic handling but the exact same algorithm), calling `AtomicWaker::wake` in `send` hot path. As a result, it systematically pays two contended RMWs.

Replacing `AtomicWaker` with `SpmcWaker` should improve performance, while not needing any other code adjustment than adding an unsafe block around waker registration.

Also, because the wake condition is already set with a `Release` RMW, `SpmcWaker<Unsynchronized>` variant can be used by replacing the RMW ordering with `AcqRel` (and checking the wake condition with an `AcqRel` RMW instead of an `Acquire` load, but only after registering the waker). As a consequence, it removes an RMW from the hot path when no waker is registered, which is significant on x86.

The following tables present the results of tokio's own mpsc benchmark depending on the atomic waker used. These results come with an important caveat: channel benchmarks are often dominated by cache-line contention, whose effects are hardly predictable; a "faster" algorithm may sometimes lead to more contention and give disastrous results in benchmarks. Also, the atomic waker is not the main part of the whole MPSC channel algorithm. Still, the replacement of `AtomicWaker` seems to produce a noticeable effect on the results.

#### x86_64

| Benchmark                          | `AtomicWaker`         | `SpmcWaker`            | `SpmcWaker<Unsynchronized>` |
|------------------------------------|-----------------------|------------------------|-----------------------------|
| contention/bounded                 | 726.0 µs (baseline)   | 758.6 µs (+4%)         | 653.1 µs (-10%)             |
| contention/bounded_recv_many       | 526.5 µs (baseline)   | 517.2 µs (-2%)         | 467.9 µs (-11%)             |
| contention/bounded_full            | 850.2 µs (baseline)   | 820.4 µs (-4%)         | 841.0 µs (-1%)              |
| contention/bounded_full_recv_many  | 530.4 µs (baseline)   | 456.0 µs (-14%)        | 470.5 µs (-11%)             |
| contention/unbounded               | 630.5 µs (baseline)   | 562.7 µs (-11%)        | 545.7 µs (-13%)             |
| contention/unbounded_recv_many     | 600.9 µs (baseline)   | 500.0 µs (-17%)        | 463.7 µs (-23%)             |
| uncontented/bounded                | 423.6 µs (baseline)   | 397.0 µs (-6%)         | 367.4 µs (-13%)             |
| uncontented/bounded_recv_many      | 307.7 µs (baseline)   | 269.3 µs (-12%)        | 257.1 µs (-16%)             |
| uncontented/unbounded              | 238.0 µs (baseline)   | 204.4 µs (-14%)        | 180.7 µs (-24%)             |
| uncontented/unbounded_recv_many    | 189.8 µs (baseline)   | 154.1 µs (-19%)        | 139.6 µs (-26%)             |

#### aarch64

| Benchmark                          | `AtomicWaker`         | `SpmcWaker`            | `SpmcWaker<Unsynchronized>` |
|------------------------------------|-----------------------|------------------------|-----------------------------|
| contention/bounded                 | 713.5 µs (baseline)   | 751.2 µs (+5%)         | 722.8 µs (+1%)              |
| contention/bounded_recv_many       | 850.4 µs (baseline)   | 819.9 µs (-4%)         | 708.5 µs (-17%)             |
| contention/bounded_full            | 627.3 µs (baseline)   | 578.6 µs (-8%)         | 553.8 µs (-12%)             |
| contention/bounded_full_recv_many  | 418.5 µs (baseline)   | 363.8 µs (-13%)        | 367.8 µs (-12%)             |
| contention/unbounded               | 684.1 µs (baseline)   | 669.8 µs (-2%)         | 643.4 µs (-6%)              |
| contention/unbounded_recv_many     | 652.3 µs (baseline)   | 590.7 µs (-9%)         | 582.9 µs (-11%)             |
| uncontented/bounded                | 111.3 µs (baseline)   | 108.5 µs (-3%)         | 109.6 µs (-1%)              |
| uncontented/bounded_recv_many      | 78.73 µs (baseline)   | 74.31 µs (-6%)         | 73.71 µs (-6%)              |
| uncontented/unbounded              | 63.34 µs (baseline)   | 62.69 µs (-1%)         | 66.21 µs (+5%)              |
| uncontented/unbounded_recv_many    | 43.31 µs (baseline)   | 41.50 µs (-4%)         | 45.40 µs (+5%)              |


## Safety

This crate uses unsafe code, as well as exposing unsafe methods. It is extensively tested with both [`miri`](https://github.com/rust-lang/miri) and [`loom`](https://github.com/tokio-rs/loom), including tests adapted from `AtomicWaker`. 

Each memory ordering for atomic operations is carefully chosen and tested; downgrading any of them makes the test suite fail. 

## Comparison with `DiatomicWaker`

`DiatomicWaker` is an alternative to `AtomicWaker`, that aims to be faster and better suited for an MPSC channel — the same goal as `SpmcWaker` in the end.
 
While discovered after starting the project, it was a great inspiration, not for its algorithm, but for its innovative ideas which `SpmcWaker` took up:
- assuming an SPMC algorithm with unsafe methods
- waker caching
- wait-free (non-spinning) registration

However, `SpmcWaker`'s algorithm was mainly focused on making `wake_cold` as light as possible; that's why the initial algorithm used `SeqCst` (`S=Sequential` version). The default synchronized algorithm was added later for compatibility — and for its own benefits.

Still, `SpmcWaker` pushes algorithm and code optimizations further than `DiatomicWaker`, as shown in [benchmarks](benches/README.md).

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)

at your option.