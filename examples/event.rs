use std::{
    future::poll_fn,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::Relaxed},
    },
    task::Poll,
    thread,
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

#[tokio::main]
async fn main() {
    let (notifier, mut waiter) = event();
    thread::spawn(move || notifier.notify());
    waiter.wait().await;
}
