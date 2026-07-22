use std::{
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc,
    },
    thread,
};

use futures::executor::block_on;
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
        let is_notified = |_| self.0.notified.swap(false, Relaxed);
        self.0.waker.wait_until(is_notified).await;
    }

    fn notifier(&self) -> Notifier {
        Notifier(self.0.clone())
    }
}

fn event() -> (Notifier, Waiter) {
    let waiter = Waiter::default();
    (waiter.notifier(), waiter)
}

fn main() {
    let (notifier, mut waiter) = event();
    thread::spawn(move || notifier.notify());
    block_on(waiter.wait());
}
