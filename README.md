# spmc-waker

A synchronization primitive for task wakeup.

## Features

- `no_std`
- optimized for a single producer (the polled future registering its waker) and multiple consumers (threads calling `wake`)
- opt out of acquire-release synchronization between `register` and `wake` for extra performance

## Example

```rust
use std::{
    future::poll_fn,
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc,
    },
    task::Poll,
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
    fn notify(&self) {
        self.0.notified.store(true, Relaxed);
        self.0.waker.wake();
    }
}

#[derive(Default)]
struct Waiter(Arc<Inner>);

impl Waiter {
    async fn wait(&mut self) {
        poll_fn(move |cx| {
            // quick check to avoid registration if already done.
            if self.0.notified.swap(false, Relaxed) {
                return Poll::Ready(());
            }
            // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
            unsafe { self.0.waker.register(cx.waker()) };
            // Need to check condition **after** `register` to avoid a race
            // condition that would result in lost notifications.
            if self.0.notified.swap(false, Relaxed) {
                // Unregister the waker to avoid spurious wakeups.
                // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
                unsafe { self.0.waker.unregister() };
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    fn notifier(&self) -> Notifier {
        Notifier(self.0.clone())
    }
}

fn event() -> (Notifier, Waiter) {
    let waiter = Waiter::default();
    (waiter.notifier(), waiter)
}
```

## Differences with [`AtomicWaker`](https://docs.rs/futures/latest/futures/task/struct.AtomicWaker.html)

`AtomicWaker`'s algorithm is MPMC (multi-producer, multi-consumer): it supports concurrent calls to `register`, although it explicitly advises against doing it. On the contrary, `SpmcWaker` assumes to be SPMC: `SpmcWaker::register` cannot be called concurrently. It allows algorithm optimizations, but requires the method to be *unsafe*.

`AtomicWaker::wake` always synchronizes with `AtomicWaker::register`. For its part, `SpmcWaker` comes with a generic boolean parameter `SYNC`, which decides if `SpmcWaker::wake` synchronizes with `SpmcWaker::register` (`SYNC=true`, the default), or not (`SYNC=false`). Workflows using `SpmcWaker<false>` need to pair it with a total order, like atomic `SeqCst` or RMW operations — in the example above, it could be done by replacing `Relaxed` with `SeqCst` in `notified` accesses. The unsynchronized algorithm is even more optimized, so when the surrounding code already uses a total order[^1], it can significantly benefit from it.

## Performance

The following table compares the atomic operations of `register` and `wake` methods for the different primitives.

|                         | `AtomicWaker`              | `SpmcWaker`                               | `SpmcWaker<false>`                          |
|-------------------------|----------------------------|-------------------------------------------|---------------------------------------------|
| `register` (from empty) | RMW(Acquire) + RMW(AcqRel) | load(SeqCst) + RMW(SeqCst)                | load(SeqCst) + store(SeqCst)                |
| `register` (overwrite)  | RMW(Acquire) + RMW(AcqRel) | load(SeqCst) + RMW(SeqCst)                | load(SeqCst) + RMW(SeqCst)                  |
| `wake` (waker present)  | RMW(AcqRel) + RMW(Release) | load(Relaxed) + RMW(SeqCst) + RMW(SeqCst) | load(SeqCst) + RMW(SeqCst)  + store(SeqCst) |
| `wake` (no waker)       | RMW(AcqRel) + RMW(Release) | load(Relaxed) + RMW(SeqCst)               | load(SeqCst)                                |

Compared to `AtomicWaker`, `SpmcWaker` reduces the number of RMW operations for `register`, and for `wake` when there is no waker. `SpmcWaker<false>` goes even further by replacing a few `SeqCst` RMW with `SeqCst` stores[^2], and more importantly by removing all RMWs on `wake` when there is no waker.

As illustrated in the example, `SpmcWaker` is designed to be used in MPSC algorithms, i.e. one waiter registering its waker with multiple notifiers. In an MPSC channel case with some throughput, receiver waker is rarely registered, as there are more often already items waiting in the queue. However, receiver waker is systematically woken by producers, so optimizing `wake` when there is no waker registered becomes the most important. `SpmcWaker` algorithm was built with this goal in mind.

## Safety

This crate uses unsafe code, as well as exposing unsafe methods. It is tested with both [`miri`](https://github.com/rust-lang/miri) and [`loom`](https://github.com/tokio-rs/loom), including tests adapted from `AtomicWaker`.

[^1]: On x86_64, `SeqCst` ordering makes no difference compared to `Relaxed` for RMW operations and load, while on aarch64, `SeqCst` adds a small overhead for load and stores. If the waking condition is already set with a RMW operation, using `SeqCst` in combination with `SpmcWaker<false>` can be worth it.
[^2]: It has no effect on x86_64, as a `SeqCst` store is compiled to an `xchg` instruction — same as swap, but it matters on aarch64.