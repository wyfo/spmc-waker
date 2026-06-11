use std::{
    hint::black_box,
    sync::Arc,
    task::{Wake, Waker},
};

use diatomic_waker::DiatomicWaker;
use divan::Bencher;
use spmc_waker::SpmcWaker;

trait AtomicWaker: Default + Send + Sync + 'static {
    unsafe fn register(&self, waker: &Waker);
    fn wake(&self);
    fn wake_cold(&self) {
        self.wake();
    }
}

impl<const SYNC: bool, const CACHED: bool> AtomicWaker for SpmcWaker<SYNC, CACHED> {
    unsafe fn register(&self, waker: &Waker) {
        unsafe { self.register(waker) };
    }
    fn wake(&self) {
        self.wake();
    }
    fn wake_cold(&self) {
        self.wake_cold();
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
// A manual noop waker is required to make Arc cloning visible in benchmarks,
// and to have different wakers for overwrite bench.
#[allow(clippy::manual_noop_waker)]
impl Wake for FakeWaker {
    fn wake(self: Arc<Self>) {}
    fn wake_by_ref(self: &Arc<Self>) {}
}
fn fake_waker() -> Waker {
    black_box(Waker::from(Arc::new(FakeWaker)))
}

#[divan::bench(types = [SpmcWaker<true, true>, SpmcWaker<false, true>, SpmcWaker<true, false>, SpmcWaker<false, false>, futures::task::AtomicWaker, DiatomicWaker])]
fn register<W: AtomicWaker>(bencher: Bencher) {
    let waker = fake_waker();
    bencher
        .with_inputs(|| {
            let atomic_waker = W::default();
            unsafe { atomic_waker.register(&waker) };
            atomic_waker.wake();
            atomic_waker
        })
        .bench_local_refs(|atomic_waker| unsafe { atomic_waker.register(&waker) });
}

#[divan::bench(types = [SpmcWaker<true, true>, SpmcWaker<false, true>, SpmcWaker<true, false>, SpmcWaker<false, false>, futures::task::AtomicWaker, DiatomicWaker])]
fn register_already_registered<W: AtomicWaker>(bencher: Bencher) {
    let waker = fake_waker();
    bencher
        .with_inputs(|| {
            let atomic_waker = W::default();
            unsafe { atomic_waker.register(&waker) };
            atomic_waker
        })
        .bench_local_refs(|atomic_waker| unsafe { atomic_waker.register(&waker) });
}

#[divan::bench(types = [SpmcWaker<true, true>, SpmcWaker<false, true>, SpmcWaker<true, false>, SpmcWaker<false, false>, futures::task::AtomicWaker, DiatomicWaker])]
fn register_overwrite<W: AtomicWaker>(bencher: Bencher) {
    let waker1 = fake_waker();
    let waker2 = fake_waker();
    bencher
        .with_inputs(|| {
            let atomic_waker = W::default();
            unsafe { atomic_waker.register(&waker1) };
            atomic_waker
        })
        .bench_local_refs(|atomic_waker| {
            unsafe { atomic_waker.register(&waker2) };
        });
}

#[divan::bench(types = [SpmcWaker<true, true>, SpmcWaker<false, true>, SpmcWaker<true, false>, SpmcWaker<false, false>, futures::task::AtomicWaker, DiatomicWaker])]
fn wake<W: AtomicWaker>(bencher: Bencher) {
    let waker = fake_waker();
    bencher
        .with_inputs(|| {
            let atomic_waker = W::default();
            unsafe { atomic_waker.register(&waker) };
            atomic_waker
        })
        .bench_local_refs(|atomic_waker| atomic_waker.wake());
}

#[divan::bench(types = [SpmcWaker<true, true>, SpmcWaker<false, true>, SpmcWaker<true, false>, SpmcWaker<false, false>, futures::task::AtomicWaker, DiatomicWaker])]
fn wake_cold<W: AtomicWaker>(bencher: Bencher) {
    let waker = fake_waker();
    bencher
        .with_inputs(|| {
            let atomic_waker = W::default();
            unsafe { atomic_waker.register(&waker) };
            atomic_waker
        })
        .bench_local_refs(|atomic_waker| atomic_waker.wake_cold());
}

#[divan::bench(types = [SpmcWaker<true, true>, SpmcWaker<false, true>, SpmcWaker<true, false>, SpmcWaker<false, false>, futures::task::AtomicWaker, DiatomicWaker], threads = [1, 2, 4])]
fn wake_cold_empty<W: AtomicWaker>(bencher: Bencher) {
    let atomic_waker = W::default();
    bencher.bench(|| black_box(&atomic_waker).wake_cold());
}

fn main() {
    divan::main();
}
