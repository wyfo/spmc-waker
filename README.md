# spmc-waker

A synchronization primitive for task wakeup.

## Features

#### SPMC (single-producer, multi-consumer)

Contrary to `AtomicWaker`, which is MPMC while explicitly advising against it, `SpmcWaker` assumes to be SPMC: `SpmcWaker::register` cannot be called concurrently. It allows algorithm optimizations, but requires the method to be *unsafe*.

#### Rich and high-level API

In addition to basic `SpmcWaker::register`, this crate provides `SpmcWaker::try_register`, which early returns if a concurrent `SpmcWaker::wake`, as the wake condition is probably fulfilled. On the other hand, `SpmcWaker::unregister` can prevent a registered waker to be called, avoiding spurious wakeup of the task.

However, the advised way to do waker registration is through `SpmcWaker::wait_until` async function, which calls `SpmcWaker::poll_wait_until`. This method combines `try_register`, `register`, and `unregister` with wake condition check for an optimal workflow. 

#### Optional synchronization

By default, `SpmcWaker::wake` always synchronizes with `SpmcWaker::register`, same as `AtomicWaker`.

However, `SpmcWaker` has a generic `SYNC` parameter (true by default) which can be set to false. In that case, `SpmcWaker<false>`, aliased to `UnsynchronizedSpmcWaker`, relies on `SeqCst` being used on the wakeup condition, but makes `wake` significantly lighter, see [benchmarks](benches/README.md).

#### Waker caching

Most of the time, there is a single task registering its waker, so the waker is always the same. That's why `SpmcWaker` provides a second generic parameter `CACHED`. By default (`CACHED=true`), the registered waker is cached, i.e. it's not dropped when `SpmcWaker::wake` is called, using `Waker::wake_by_ref`. So there is no need to clone it when the same waker is registered again. As wakers are often `Arc`s, caching avoids atomic RMW operations updating the reference counter.

#### Progress guarantee

Waker registration is wait-free, while task waking is lock-free (without taking in account waker clone/wake/drop operations).

When waker registration is only done through `try_register`, `wake` becomes
wait-free, but registration then requires spinning until it succeeds. This is by the way the workflow used by `AtomicWaker`, which reschedule the task as spinning mechanism.

#### Cold path outlining

Some algorithms require `SpmcWaker::wake` to be called in hot path, even if there is no waker registered most of the time. This is exactly the use case for `SpmcWaker::wake_cold`, which starts by checking if a waker is registered before outlining the wake code in a cold function. It's also possible to check if a waker is registered with `SpmcWaker::has_waker_registered` before calling `SpmcWaker::wake`; even if no waker is registered, `has_waker_registered` still synchronizes with `SpmcWaker::register` when `SYNC=false`.

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

|                              | `AtomicWaker`              | `SpmcWaker`                                  | `SpmcWaker<false>`                           |
|------------------------------|----------------------------|----------------------------------------------|----------------------------------------------|
| register                     | RMW(Acquire) + RMW(AcqRel) | load(Relaxed) + RMW(Acquire)                 | load(Relaxed) + store(SeqCst)                |
| wake_cold (waker registered) | RMW(AcqRel) + RMW(Release) | load(Relaxed) + RMW(AcqRel) + RMW(Release)   | load(SeqCst) + RMW(Acquire) + store(Release) |
| wake_cold (no waker)         | RMW(AcqRel) + RMW(Release) | load(Relaxed) + RMW(Release)                 | load(SeqCst)                                 |

Compared to `AtomicWaker`, `SpmcWaker` reduces the number of RMW operations for `register`, and for `wake_cold` when there is no waker. `SpmcWaker<false>` goes even further, reducing `wake`/`wake_cold` to a single atomic load when there is no waker registered. 

Atomic operations related to waker cloning/dropping are not counted in the table. As `SpmcWaker` caches the waker, these operations don't add overhead, but for `AtomicWaker`, an additional RMW(Relaxed) for `register`, as well as a RMW(Acquire) for `wake` (waker registered) should be expected.

As illustrated in the example, `SpmcWaker` is designed to be used in MPSC algorithms, i.e. one waiter registering its waker with multiple notifiers. In an MPSC channel case with some throughput, the receiver waker is rarely registered, as there are more often already items waiting in the queue. However, the receiver waker is systematically woken by producers, so optimizing `wake` when there is no waker registered becomes the most important. `SpmcWaker` algorithm was built with this goal in mind.

### Replacing `AtomicWaker` in `tokio::sync::mpsc`

#### x86_64

| Benchmark                          | `AtomicWaker`         | `SpmcWaker`            | `SpmcWaker<false>`     |
|------------------------------------|-----------------------|------------------------|------------------------|
| contention/bounded                 | 720.7 µs (baseline)   | 676.8 µs (-6%)         | 671.3 µs (-7%)         |
| contention/bounded_recv_many       | 575.0 µs (baseline)   | 469.2 µs (-18%)        | 443.4 µs (-23%)        |
| contention/bounded_full            | 897.1 µs (baseline)   | 849.0 µs (-5%)         | 841.0 µs (-6%)         |
| contention/bounded_full_recv_many  | 568.3 µs (baseline)   | 501.9 µs (-12%)        | 468.2 µs (-18%)        |
| contention/unbounded               | 630.8 µs (baseline)   | 594.2 µs (-6%)         | 540.0 µs (-14%)        |
| contention/unbounded_recv_many     | 600.9 µs (baseline)   | 493.5 µs (-18%)        | 488.4 µs (-19%)        |
| uncontented/bounded                | 433.4 µs (baseline)   | 398.0 µs (-8%)         | 408.5 µs (-6%)         |
| uncontented/bounded_recv_many      | 311.4 µs (baseline)   | 273.3 µs (-12%)        | 251.2 µs (-19%)        |
| uncontented/unbounded              | 237.7 µs (baseline)   | 238.7 µs (+0%)         | 180.4 µs (-24%)        |
| uncontented/unbounded_recv_many    | 190.5 µs (baseline)   | 160.1 µs (-16%)        | 133.5 µs (-30%)        |

#### aarch64

| Benchmark                          | `AtomicWaker`         | `SpmcWaker`            | `SpmcWaker<false>`     |
|------------------------------------|-----------------------|------------------------|------------------------|
| contention/bounded                 | 701.6 µs (baseline)   | 746.3 µs (+6%)         | 700.6 µs (-0%)         |
| contention/bounded_recv_many       | 887.2 µs (baseline)   | 779.3 µs (-12%)        | 721.4 µs (-19%)        |
| contention/bounded_full            | 619.1 µs (baseline)   | 564.3 µs (-9%)         | 525.8 µs (-15%)        |
| contention/bounded_full_recv_many  | 394.1 µs (baseline)   | 388.6 µs (-1%)         | 354.4 µs (-10%)        |
| contention/unbounded               | 674.1 µs (baseline)   | 677.1 µs (+0%)         | 649.3 µs (-4%)         |
| contention/unbounded_recv_many     | 650.2 µs (baseline)   | 596.3 µs (-8%)         | 542.1 µs (-17%)        |
| uncontented/bounded                | 111.4 µs (baseline)   | 108.6 µs (-3%)         | 106.0 µs (-5%)         |
| uncontented/bounded_recv_many      | 78.61 µs (baseline)   | 75.32 µs (-4%)         | 71.29 µs (-9%)         |
| uncontented/unbounded              | 63.07 µs (baseline)   | 61.31 µs (-3%)         | 64.20 µs (+2%)         |
| uncontented/unbounded_recv_many    | 43.39 µs (baseline)   | 40.55 µs (-7%)         | 44.04 µs (+2%)         |

### Replacing `DiatomicWaker` in `tachyonix`

`tachyonix` channel uses a custom atomic waker primitive named `DiatomicWaker` (see [next section](#comparison-with-diatomicwaker)).

Throughput of the `pinball` benchmark, in `msg/µs` (higher is better).

#### x86_64

| ball count | `DiatomicWaker`           | `SpmcWaker`            | `SpmcWaker<false>`     |
|------------|---------------------------|------------------------|------------------------|
| 1          | 43.97 msg/µs (baseline)   | 45.38 msg/µs (+3%)     | 44.54 msg/µs (+1%)     |
| 3          | 45.43 msg/µs (baseline)   | 46.66 msg/µs (+3%)     | 46.31 msg/µs (+2%)     |
| 7          | 53.47 msg/µs (baseline)   | 55.65 msg/µs (+4%)     | 55.55 msg/µs (+4%)     |
| 17         | 65.19 msg/µs (baseline)   | 68.17 msg/µs (+5%)     | 69.41 msg/µs (+6%)     |
| 41         | 86.05 msg/µs (baseline)   | 90.77 msg/µs (+5%)     | 91.95 msg/µs (+7%)     |
| 101        | 108.7 msg/µs (baseline)   | 115.5 msg/µs (+6%)     | 116.4 msg/µs (+7%)     |
| 241        | 126.3 msg/µs (baseline)   | 133.8 msg/µs (+6%)     | 135.4 msg/µs (+7%)     |

#### aarch64

| ball count | `DiatomicWaker`           | `SpmcWaker`            | `SpmcWaker<false>`     |
|------------|---------------------------|------------------------|------------------------|
| 1          | 55.86 msg/µs (baseline)   | 61.18 msg/µs (+10%)    | 56.47 msg/µs (+1%)     |
| 3          | 55.95 msg/µs (baseline)   | 61.16 msg/µs (+9%)     | 56.42 msg/µs (+1%)     |
| 7          | 60.91 msg/µs (baseline)   | 68.56 msg/µs (+13%)    | 61.77 msg/µs (+1%)     |
| 17         | 78.57 msg/µs (baseline)   | 86.98 msg/µs (+11%)    | 79.70 msg/µs (+1%)     |
| 41         | 123.7 msg/µs (baseline)   | 137.7 msg/µs (+11%)    | 125.9 msg/µs (+2%)     |
| 101        | 212.5 msg/µs (baseline)   | 238.8 msg/µs (+12%)    | 223.2 msg/µs (+5%)     |
| 241        | 315.7 msg/µs (baseline)   | 341.8 msg/µs (+8%)     | 334.5 msg/µs (+6%)     |

## Safety

This crate uses unsafe code, as well as exposing unsafe methods. It is extensively tested with both [`miri`](https://github.com/rust-lang/miri) and [`loom`](https://github.com/tokio-rs/loom), including tests adapted from `AtomicWaker`. 

Each memory ordering for atomic operations is carefully chosen and tested; downgrading any of them makes the test suite fail. 

## Comparison with `DiatomicWaker`

`DiatomicWaker` is an alternative to `AtomicWaker`, which aims to be faster and better suited for a MPSC channel; the same goal as `SpmcWaker` in the end.
 
While discovered after starting the project, it was a great inspiration, not for its algorithm, but for its innovative ideas which `SpmcWaker` took over:
- assuming SPMC algorithm with unsafe methods
- waker caching
- wait-free (non-spinning) registration

However, `SpmcWaker` algorithm was mainly focused on making `wake_cold` the lightest possible, that's why the initial algorithm used `SeqCst` (`SYNC=false` version); `SYNC=true` algorithm was added later for compatibility — and for its own benefits.

Still, `SpmcWaker` pushes algorithm and code optimizations further than `DiatomicWaker`, as shown in [benchmarks](benches/README.md).