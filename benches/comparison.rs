use std::{
    hint::spin_loop,
    sync::Arc,
    task::{Wake, Waker},
};

use diatomic_waker::DiatomicWaker;
use divan::Bencher;
use spmc_waker::SpmcWaker;

trait AtomicWaker: Default + Send + Sync + 'static {
    unsafe fn register(&self, waker: &Waker);
    fn wake(&self);
}

impl<const SYNC: bool, const CACHED: bool> AtomicWaker for SpmcWaker<SYNC, CACHED> {
    unsafe fn register(&self, waker: &Waker) {
        unsafe { self.register(waker) };
    }
    fn wake(&self) {
        self.wake();
    }
}

impl AtomicWaker for futures::task::AtomicWaker {
    unsafe fn register(&self, waker: &Waker) {
        self.register(waker);
    }
    fn wake(&self) {
        self.wake();
    }
}

impl AtomicWaker for DiatomicWaker {
    unsafe fn register(&self, waker: &Waker) {
        unsafe { self.register(waker) };
    }
    fn wake(&self) {
        self.notify();
    }
}

struct FakeWaker;
impl Wake for FakeWaker {
    fn wake(self: Arc<Self>) {}
}

#[divan::bench(types = [SpmcWaker<true, true>, SpmcWaker<false, true>, SpmcWaker<true, false>, SpmcWaker<false, false>, futures::task::AtomicWaker, DiatomicWaker])]
fn register<W: AtomicWaker>(bencher: Bencher) {
    let atomic_waker = W::default();
    let waker = Waker::from(Arc::new(FakeWaker));
    bencher.bench(|| {
        unsafe { atomic_waker.register(&waker) };
    });
}

#[divan::bench(types = [SpmcWaker<true, true>, SpmcWaker<false, true>, SpmcWaker<true, false>, SpmcWaker<false, false>, futures::task::AtomicWaker, DiatomicWaker])]
fn register_wake<W: AtomicWaker>(bencher: Bencher) {
    let atomic_waker = W::default();
    let waker = Waker::from(Arc::new(FakeWaker));
    bencher.bench(|| {
        unsafe { atomic_waker.register(&waker) };
        atomic_waker.wake();
    });
}

#[divan::bench(types = [SpmcWaker<true, true>, SpmcWaker<false, true>, SpmcWaker<true, false>, SpmcWaker<false, false>, futures::task::AtomicWaker, DiatomicWaker])]
fn register_spin_wake<W: AtomicWaker>(bencher: Bencher) {
    let atomic_waker = W::default();
    let waker = Waker::from(Arc::new(FakeWaker));
    bencher.bench(|| {
        unsafe { atomic_waker.register(&waker) };
        spin_loop();
        atomic_waker.wake();
    });
}

#[divan::bench(types = [SpmcWaker<true, true>, SpmcWaker<false, true>, SpmcWaker<true, false>, SpmcWaker<false, false>, futures::task::AtomicWaker, DiatomicWaker])]
fn register_overwrite<W: AtomicWaker>(bencher: Bencher) {
    let atomic_waker = W::default();
    let waker1 = Waker::from(Arc::new(FakeWaker));
    let waker2 = Waker::from(Arc::new(FakeWaker));
    bencher.bench(|| {
        unsafe { atomic_waker.register(&waker1) };
        unsafe { atomic_waker.register(&waker2) };
    });
}

#[divan::bench(types = [SpmcWaker<true, true>, SpmcWaker<false, true>, SpmcWaker<true, false>, SpmcWaker<false, false>, futures::task::AtomicWaker, DiatomicWaker], threads = [1, 2, 4])]
fn wake_empty<W: AtomicWaker>(bencher: Bencher) {
    let atomic_waker = W::default();
    bencher.bench(|| atomic_waker.wake());
}

#[divan::bench(types = [SpmcWaker<true, true>, SpmcWaker<false, true>, SpmcWaker<true, false>, SpmcWaker<false, false>, futures::task::AtomicWaker, DiatomicWaker], threads = [1, 2, 4])]
fn wake_empty_spin<W: AtomicWaker>(bencher: Bencher) {
    let atomic_waker = W::default();
    bencher.bench(|| {
        atomic_waker.wake();
        spin_loop();
    });
}

fn main() {
    divan::main();
}
