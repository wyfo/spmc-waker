# spmc-waker

A synchronization primitive for task wakeup.

## Features

- optimized for a single producer (the polled future registering its waker) and multiple consumers (threads calling `wake`)
- opt out of acquire-release synchronization between `register` and `wake` for extra performance
- Waker caching to save cloning/dropping when the waker is reused in a task

## Example

```rust
use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            Ordering::{Relaxed, SeqCst},
        },
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
        // Use seqcst ordering to synchronize with the load after `register`
        self.0.notified.store(true, SeqCst);
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
        unsafe { self.0.waker.register(cx.waker()) };

        // Need to check condition **after** `register` to avoid a race
        // condition that would result in lost notifications.
        // Use seqcst ordering so it synchronizes with the store before wake.
        if self.0.notified.load(SeqCst) {
            // Unregister the waker to avoid spurious wakeups.
            // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
            unsafe { self.0.waker.unregister() };
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

fn event() -> (Notifier, Waiter) {
    let waiter = Waiter::default();
    (waiter.notifier(), waiter)
}
```

## Differences with [`AtomicWaker`](https://docs.rs/futures/latest/futures/task/struct.AtomicWaker.html)

#### SPMC (single-producer, multi-consumer)

`AtomicWaker`'s algorithm is MPMC (multi-producer, multi-consumer): it supports concurrent calls to `register`, although it explicitly advises against doing it. On the contrary, `SpmcWaker` assumes to be SPMC: `SpmcWaker::register` cannot be called concurrently. It allows algorithm optimizations, but requires the method to be *unsafe*.

#### Optional synchronization

`AtomicWaker::wake` always synchronizes with `AtomicWaker::register`. For its part, `SpmcWaker` comes with a generic boolean parameter `SYNC`, which decides if `SpmcWaker::wake` synchronizes with `SpmcWaker::register` (`SYNC=true`), or not (`SYNC=false`, the default). Workflows using `SpmcWaker<false>` require the waking condition to use a total order, such as `SeqCst` ordering or RMW operations. It ensures that checking the waking condition after `register` succeeds even when a concurrent `wake` misses the registered waker.

#### Waker caching

`AtomicWaker` always clones the registered waker, and drops it with `Waker::wake`. However, most of the time, the waker registered is always the same, as there is a single task polling it. That's why `SpmcWaker` provides a second generic parameter `CACHED`. By default (`CACHED=true`), the registered waker is cached, i.e. it's not dropped when `SpmcWaker::wake` is called, using `Waker::wake_by_ref`.
So there is no need to clone it when the same waker is registered again. As wakers are often `Arc`s, caching avoids atomic RMW operations updating the reference counter.

## Performance

See [benchmark results](benches/README.md). The following table compares the atomic operations of `register` and `wake` methods for the different primitives.

|                         | `AtomicWaker`              | `SpmcWaker<true>`                         | `SpmcWaker`                                 |
|-------------------------|----------------------------|-------------------------------------------|---------------------------------------------|
| `register` (from empty) | RMW(Acquire) + RMW(AcqRel) | load(SeqCst) + RMW(SeqCst)                | load(SeqCst) + store(SeqCst)                |
| `register` (overwrite)  | RMW(Acquire) + RMW(AcqRel) | load(SeqCst) + RMW(SeqCst)                | load(SeqCst) + RMW(SeqCst)                  |
| `wake` (waker present)  | RMW(AcqRel) + RMW(Release) | load(Relaxed) + RMW(SeqCst) + RMW(SeqCst) | load(SeqCst) + RMW(SeqCst)  + store(SeqCst) |
| `wake` (no waker)       | RMW(AcqRel) + RMW(Release) | load(Relaxed) + RMW(SeqCst)               | load(SeqCst)                                |

Compared to `AtomicWaker`, `SpmcWaker<true>` reduces the number of RMW operations for `register`, and for `wake` when there is no waker. `SpmcWaker` goes even further by replacing a few `SeqCst` RMW with `SeqCst` stores[^1], and more importantly by removing all RMWs on `wake` when there is no waker. 

Atomic operations related to waker cloning/dropping are not counted in the table. As `SpmcWaker` caches the waker, these operations don't add overhead, but for `AtomicWaker`, an additional RMW(Relaxed) for `register`, as well as a RMW(Acquire) for `wake` (waker present) can be expected.

As illustrated in the example, `SpmcWaker` is designed to be used in MPSC algorithms, i.e. one waiter registering its waker with multiple notifiers. In an MPSC channel case with some throughput, the receiver waker is rarely registered, as there are more often already items waiting in the queue. However, the receiver waker is systematically woken by producers, so optimizing `wake` when there is no waker registered becomes the most important. `SpmcWaker` algorithm was built with this goal in mind.

As a concrete example, replacing `AtomicWaker` with `SpmcWaker` in `tokio::sync::mpsc` improves tokio's benchmark up to 30%.

## Safety

This crate uses unsafe code, as well as exposing unsafe methods. It is tested with both [`miri`](https://github.com/rust-lang/miri) and [`loom`](https://github.com/tokio-rs/loom), including tests adapted from `AtomicWaker`.

## Acknowledgement

The idea of waker caching has been borrowed from [diatomic-waker](https://crates.io/crates/diatomic-waker) crate.

[^1]: It has no effect on x86_64, as a `SeqCst` store is compiled to an `xchg` instruction — same as swap, but it matters on aarch64.