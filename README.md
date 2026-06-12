# spmc-waker

A synchronization primitive for task wakeup.

## Features

#### SPMC (single-producer, multi-consumer)

Contrary to `AtomicWaker`, which is MPMC while explicitly advising against it, `SpmcWaker` assumes to be SPMC: `SpmcWaker::register` cannot be called concurrently. It allows algorithm optimizations, but requires the method to be *unsafe*.

#### Optional synchronization

By default, `SpmcWaker::wake` always synchronizes with `SpmcWaker::register`, same as `AtomicWaker`.

However, `SpmcWaker` has a generic `SYNC` parameter (true by default) which can be set to false. In that case, `SpmcWaker<false>`, aliased to `UnsynchronizedSpmcWaker`, relies on `SeqCst` being used on the wakeup condition, but makes `wake` significantly lighter, see [benchmarks](benches/README.md).

#### Waker caching

Most of the time, there is a single task registering its waker, so the waker is always the same. That's why `SpmcWaker` provides a second generic parameter `CACHED`. By default (`CACHED=true`), the registered waker is cached, i.e. it's not dropped when `SpmcWaker::wake` is called, using `Waker::wake_by_ref`. So there is no need to clone it when the same waker is registered again. As wakers are often `Arc`s, caching avoids atomic RMW operations updating the reference counter.

#### Cold path outlining

Some algorithms require `SpmcWaker::wake` to be called in hot path, even if there is no waker registered most of the time. This is exactly the use case for `SpmcWaker::wake_cold`, which starts by checking if a waker is registered before outlining the wake code in a cold function. It's also possible to check if a waker is registered with `SpmcWaker::has_waker_registered` before calling `SpmcWaker::wake`; even if no waker is registered, `has_waker_registered` still synchronizes with `SpmcWaker::register` when `SYNC=false`.
  
#### Unwind-safety

If any waker operation (clone/wake/wake_by_ref/drop) panics, `SpmcWaker` state is cleaned up to an unregistered state.

#### Two-pointers-sized

`SpmcWaker` has same size as `Waker`, i.e., two pointers.

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
        // quick check to avoid registration if already done.
        if self.0.notified.load(Relaxed) {
            return Poll::Ready(());
        }

        // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
        let registered = unsafe { self.0.waker.register(cx.waker()) };

        // Need to check condition **after** `register` to avoid a race
        // condition that would result in lost notifications.
        if self.0.notified.load(Relaxed) {
            // Unregister the waker to avoid spurious wakeups.
            // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
            unsafe { self.0.waker.unregister() };
            Poll::Ready(())
        } else {
            // Waker wasn't registered, but wake condition is still not fulfilled.
            // Reschedule to retry later.
            if !registered {
                cx.waker().wake_by_ref();
            }
            Poll::Pending
        }
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
| wake (waker registered)      | RMW(AcqRel) + RMW(Release) | RMW(Release) + fence(Acquire) + RMW(Release) | load(SeqCst) + RMW(Acquire) + store(Release) |
| wake_cold (waker registered) | RMW(AcqRel) + RMW(Release) | load(Relaxed) + RMW(AcqRel) + RMW(Release)   | load(SeqCst) + RMW(Acquire) + store(Release) |
| wake_cold (no waker)         | RMW(AcqRel) + RMW(Release) | load(Relaxed) + RMW(Release)                 | load(SeqCst)                                 |
|                              |                            |                                              |                                              |

Compared to `AtomicWaker`, `SpmcWaker` reduces the number of RMW operations for `register`, and for `wake_cold` when there is no waker. `SpmcWaker<false>` goes even further, reducing `wake`/`wake_cold` to a single atomic load when there is no waker registered. 

Atomic operations related to waker cloning/dropping are not counted in the table. As `SpmcWaker` caches the waker, these operations don't add overhead, but for `AtomicWaker`, an additional RMW(Relaxed) for `register`, as well as a RMW(Acquire) for `wake` (waker registered) should be expected.

As illustrated in the example, `SpmcWaker` is designed to be used in MPSC algorithms, i.e. one waiter registering its waker with multiple notifiers. In an MPSC channel case with some throughput, the receiver waker is rarely registered, as there are more often already items waiting in the queue. However, the receiver waker is systematically woken by producers, so optimizing `wake` when there is no waker registered becomes the most important. `SpmcWaker` algorithm was built with this goal in mind.

### Replacing `AtomicWaker` in `tokio::sync::mpsc`

TODO: run tokio benchmarks and compare baseline with version with SpmcWaker and SpmcWaker<false>

### Replacing `DiatomicWaker` in `tachyonix`

TODO: run tachyonix benchmarks and compare baseline with version with SpmcWaker and SpmcWaker<false>

## Safety

This crate uses unsafe code, as well as exposing unsafe methods. It is extensively tested with both [`miri`](https://github.com/rust-lang/miri) and [`loom`](https://github.com/tokio-rs/loom), including tests adapted from `AtomicWaker`. 

Each memory ordering is carefully chosen and tested; downgrading any of them makes test suite fail. 