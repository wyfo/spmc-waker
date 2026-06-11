use std::{
    future::poll_fn,
    ptr,
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            Ordering::{Relaxed, SeqCst},
        },
    },
    task::{Poll, RawWaker, RawWakerVTable, Wake, Waker},
};
#[cfg(not(loom))]
use std::{panic::catch_unwind, sync::atomic::AtomicUsize, thread};

#[cfg(not(loom))]
use futures::executor::block_on;
#[cfg(loom)]
use loom::{future::block_on, model, sync::atomic::AtomicUsize};
use rstest::rstest;
use spmc_waker::SpmcWaker;

#[cfg(loom)]
mod thread {
    use std::{cell::RefCell, marker::PhantomData};

    use loom::thread::JoinHandle;
    pub use loom::thread::spawn;

    pub struct Scope<'env> {
        handles: RefCell<Vec<Option<JoinHandle<()>>>>,
        _env: PhantomData<&'env mut ()>,
    }

    impl Drop for Scope<'_> {
        fn drop(&mut self) {
            for handle in self.handles.get_mut().drain(..).flatten() {
                handle.join().unwrap();
            }
        }
    }

    impl<'env> Scope<'env> {
        pub fn spawn<T: Send + 'env>(
            &self,
            f: impl FnOnce() -> T + Send + 'env,
        ) -> ScopedJoinHandle<'_, 'env> {
            let mut handles = self.handles.borrow_mut();
            let handle_idx = handles.len();
            handles.push(Some(spawn(unsafe {
                core::mem::transmute::<
                    Box<dyn FnOnce() + Send + 'env>,
                    Box<dyn FnOnce() + Send + 'static>,
                >(Box::new(|| {
                    f();
                }))
            })));
            ScopedJoinHandle {
                scope: self,
                handle_idx,
            }
        }
    }

    pub struct ScopedJoinHandle<'a, 'env> {
        scope: &'a Scope<'env>,
        handle_idx: usize,
    }

    impl ScopedJoinHandle<'_, '_> {
        #[allow(dead_code)]
        fn join(self) -> std::thread::Result<()> {
            self.scope.handles.borrow_mut()[self.handle_idx]
                .take()
                .unwrap()
                .join()
        }
    }

    pub fn scope<'env, T>(f: impl FnOnce(&Scope<'env>) -> T) -> T {
        let scope = Scope {
            handles: RefCell::new(Vec::new()),
            _env: PhantomData,
        };
        f(&scope)
    }
}

struct Bool<const BOOL: bool>;
const TRUE: Bool<true> = Bool::<true>;
const FALSE: Bool<false> = Bool::<false>;

#[cfg(not(loom))]
fn model(f: impl FnOnce()) {
    f();
}

#[cfg(any(debug_assertions, loom))]
#[test]
#[should_panic]
fn exclusive_access() {
    model(|| {
        static STOP: AtomicBool = AtomicBool::new(false);
        fn raw_waker() -> RawWaker {
            fn clone(_: *const ()) -> RawWaker {
                #[cfg(not(loom))]
                while !STOP.load(SeqCst) {
                    std::hint::spin_loop();
                }
                raw_waker()
            }
            RawWaker::new(
                ptr::null(),
                &RawWakerVTable::new(clone, |_| (), |_| (), |_| ()),
            )
        }
        struct StopGuard;
        impl Drop for StopGuard {
            fn drop(&mut self) {
                STOP.store(true, SeqCst);
            }
        }
        let spmc = <SpmcWaker>::new();
        let waker = unsafe { Waker::from_raw(raw_waker()) };
        thread::scope(|s| {
            s.spawn(|| {
                let _guard = StopGuard;
                unsafe { spmc.register(&waker) };
            });
            s.spawn(|| {
                let _guard = StopGuard;
                unsafe { spmc.register(&waker) };
            });
        });
    });
}

#[derive(Default)]
struct CounterWaker(AtomicUsize);

impl CounterWaker {
    fn new() -> Arc<Self> {
        Default::default()
    }
    fn waker(self: &Arc<Self>) -> Waker {
        Waker::from(self.clone())
    }
    fn wake_count(&self) -> usize {
        self.0.load(Relaxed)
    }
    fn strong_count(self: &Arc<Self>) -> usize {
        Arc::strong_count(self)
    }
}

impl Wake for CounterWaker {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Relaxed);
    }
}

fn concurrent_register_and_wake<const SYNC: bool, const CACHED: bool>(
    spmc: SpmcWaker<SYNC, CACHED>,
    waker: &Arc<CounterWaker>,
) {
    let registered = thread::scope(|s| {
        s.spawn(|| spmc.wake());
        s.spawn(|| spmc.wake());
        unsafe { spmc.register(&waker.waker()) }
    });
    let waker_count = waker.strong_count() - 1;
    match (waker.wake_count(), waker_count, registered) {
        (1, 1, true) if CACHED => {} // register called before wake
        (1, 0, true) => {}           // register called before wake
        (0, 0, false) => {}          // register raced with wake
        (0, 1, true) => {}           // register called after wake
        other => panic!("unexpected outcome: {other:?}"),
    }
}

#[rstest]
fn concurrent_register_empty_and_wake<const SYNC: bool, const CACHED: bool>(
    #[values(FALSE, TRUE)] _sync: Bool<SYNC>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    model(|| {
        let spmc = SpmcWaker::<SYNC, CACHED>::new();
        let waker = CounterWaker::new();
        concurrent_register_and_wake(spmc, &waker);
    });
}

#[rstest]
fn concurrent_register_overwrite_and_wake<const SYNC: bool, const CACHED: bool>(
    #[values(FALSE, TRUE)] _sync: Bool<SYNC>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    model(|| {
        let spmc = SpmcWaker::<SYNC, CACHED>::new();
        let waker = CounterWaker::new();
        unsafe { spmc.register(Waker::noop()) };
        concurrent_register_and_wake(spmc, &waker);
    });
}

#[rstest]
fn concurrent_unregister_and_wake<const SYNC: bool, const CACHED: bool>(
    #[values(FALSE, TRUE)] _sync: Bool<SYNC>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    model(|| {
        let spmc = SpmcWaker::<SYNC, CACHED>::new();
        let waker = Arc::<CounterWaker>::default();
        assert!(unsafe { spmc.register(&waker.waker()) });
        let unregistered = thread::scope(|s| {
            s.spawn(|| spmc.wake());
            s.spawn(|| spmc.wake());
            unsafe { spmc.unregister() }
        });
        let waker_count = waker.strong_count() - 1;
        assert_eq!(waker_count, if CACHED { 1 } else { 0 });
        match (unregistered, waker.wake_count()) {
            (true, 0) => {}  // unregister called before wake
            (false, 1) => {} // unregister raced with wake or called after it
            other => panic!("unexpected outcome: {other:?}"),
        }
    });
}

#[rstest]
fn concurrent_reregister_and_wake<const SYNC: bool, const CACHED: bool>(
    #[values(FALSE, TRUE)] _sync: Bool<SYNC>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    model(|| {
        let spmc = SpmcWaker::<SYNC, CACHED>::new();
        let waker1 = CounterWaker::new();
        let waker2 = CounterWaker::new();
        assert!(unsafe { spmc.register(&waker1.waker()) });
        thread::scope(|s| {
            s.spawn(|| spmc.wake());
            unsafe { spmc.register(&waker2.waker()) };
        });
        assert!(waker1.wake_count() + waker2.wake_count() <= 1);
    });
}

#[rstest]
fn register_synchronizes_with_wake<const CACHED: bool>(
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    model(|| {
        let spmc = SpmcWaker::<true, CACHED>::new();
        let condition = AtomicUsize::new(0);
        let waker = CounterWaker::new();
        assert!(unsafe { spmc.register(&waker.waker()) });
        thread::scope(|s| {
            s.spawn(|| {
                condition.store(1, Relaxed);
                spmc.wake();
            });
            let registered = unsafe { spmc.register(Waker::noop()) };
            if !registered || waker.wake_count() == 1 {
                assert_eq!(condition.load(Relaxed), 1);
            }
        });
    });
}

// From futures test suite
#[cfg(not(loom))]
#[rstest]
fn basic<const SYNC: bool, const CACHED: bool>(
    #[values(FALSE, TRUE)] _sync: Bool<SYNC>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    let ordering = if SYNC { Relaxed } else { SeqCst };
    let atomic_waker = Arc::new(SpmcWaker::<SYNC, CACHED>::new());
    let atomic_waker_copy = atomic_waker.clone();

    let returned_pending = Arc::new(AtomicUsize::new(0));
    let returned_pending_copy = returned_pending.clone();

    let woken = Arc::new(AtomicUsize::new(0));
    let woken_copy = woken.clone();

    let t = thread::spawn(move || {
        let mut pending_count = 0;

        block_on(poll_fn(move |cx| {
            if woken_copy.load(ordering) == 1 {
                Poll::Ready(())
            } else {
                // Assert we return pending exactly once
                assert_eq!(0, pending_count);
                pending_count += 1;
                let registered = unsafe { atomic_waker_copy.register(cx.waker()) };

                returned_pending_copy.store(1, Relaxed);

                if !registered {
                    cx.waker().wake_by_ref();
                }

                Poll::Pending
            }
        }));
    });

    while returned_pending.load(Relaxed) == 0 {}

    // give spawned thread some time to sleep in `block_on`
    thread::yield_now();

    woken.store(1, ordering);
    atomic_waker.wake();

    t.join().unwrap();
}

// From tokio test suite
#[rstest]
fn basic_notification<const SYNC: bool, const CACHED: bool>(
    #[values(FALSE, TRUE)] _sync: Bool<SYNC>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    struct Chan<const S: bool, const C: bool> {
        num: AtomicUsize,
        task: SpmcWaker<S, C>,
    }

    const NUM_NOTIFY: usize = 2;
    #[cfg(loom)]
    use loom::sync::Arc;

    model(|| {
        let ordering = if SYNC { Relaxed } else { SeqCst };
        let chan = Arc::new(Chan::<SYNC, CACHED> {
            num: AtomicUsize::new(0),
            task: SpmcWaker::<SYNC, CACHED>::new(),
        });

        for _ in 0..NUM_NOTIFY {
            let chan = chan.clone();

            thread::spawn(move || {
                chan.num.fetch_add(1, ordering);
                chan.task.wake();
            });
        }

        block_on(poll_fn(move |cx| {
            let registered = unsafe { chan.task.register(cx.waker()) };

            let n = chan.num.load(ordering);
            if NUM_NOTIFY == n {
                return Poll::Ready(());
            }

            if !registered {
                cx.waker().wake_by_ref();
            }

            Poll::Pending
        }));
    });
}

#[cfg(not(loom))]
fn panic_on_clone() -> Waker {
    const VTABLE: RawWakerVTable =
        RawWakerVTable::new(|_| panic!("Waker::clone panic"), |_| (), |_| (), |_| ());
    unsafe { Waker::new(ptr::null(), &VTABLE) }
}
#[cfg(not(loom))]
fn panic_on_drop() -> std::mem::ManuallyDrop<Waker> {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        |_| RawWaker::new(ptr::null(), &VTABLE),
        |_| (),
        |_| (),
        |_| panic!("Waker::drop panic"),
    );
    std::mem::ManuallyDrop::new(unsafe { Waker::new(ptr::null(), &VTABLE) })
}
#[cfg(not(loom))]
fn panic_on_wake() -> Waker {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        |_| RawWaker::new(ptr::null(), &VTABLE),
        |_| panic!("Waker::wake panic"),
        |_| panic!("Waker::wake_by_ref panic"),
        |_| (),
    );
    unsafe { Waker::new(ptr::null(), &VTABLE) }
}

#[cfg(not(loom))]
fn check_panic_recovered<const SYNC: bool, const CACHED: bool, R>(
    spmc: SpmcWaker<SYNC, CACHED>,
    op: impl FnOnce(&SpmcWaker<SYNC, CACHED>) -> R + std::panic::UnwindSafe,
) {
    assert!(catch_unwind(|| op(&spmc)).is_err());
    let waker = CounterWaker::new();
    unsafe { spmc.register(&waker.waker()) };
    assert!(spmc.has_waker_registered());
    spmc.wake();
    assert_eq!(waker.wake_count(), 1);
    drop(spmc);
    assert_eq!(waker.strong_count(), 1);
}

#[cfg(not(loom))]
#[rstest]
fn clone_panic_in_register_can_be_recovered<const SYNC: bool, const CACHED: bool>(
    #[values(FALSE, TRUE)] _sync: Bool<SYNC>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    let spmc = SpmcWaker::<SYNC, CACHED>::new();
    check_panic_recovered(spmc, |spmc| unsafe { spmc.register(&panic_on_clone()) });
}

#[cfg(not(loom))]
#[rstest]
fn clone_panic_in_register_overwrite_can_be_recovered<const SYNC: bool, const CACHED: bool>(
    #[values(FALSE, TRUE)] _sync: Bool<SYNC>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    let spmc = SpmcWaker::<SYNC, CACHED>::new();
    unsafe { spmc.register(Waker::noop()) };
    check_panic_recovered(spmc, |spmc| unsafe { spmc.register(&panic_on_clone()) });
}

#[cfg(not(loom))]
#[rstest]
fn drop_panic_in_unregister_can_be_recovered<const SYNC: bool>(
    #[values(FALSE, TRUE)] _sync: Bool<SYNC>,
) {
    let spmc = SpmcWaker::<SYNC, false>::new();
    unsafe { spmc.register(&panic_on_drop()) };
    check_panic_recovered(spmc, |spmc| unsafe { spmc.unregister() });
}

#[cfg(not(loom))]
#[rstest]
fn drop_panic_in_register_overwrite_can_be_recovered<const SYNC: bool, const CACHED: bool>(
    #[values(FALSE, TRUE)] _sync: Bool<SYNC>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    let spmc = SpmcWaker::<SYNC, CACHED>::new();
    unsafe { spmc.register(&panic_on_drop()) };
    let waker = CounterWaker::new();
    check_panic_recovered(spmc, |spmc| unsafe { spmc.register(&waker.waker()) });
    assert_eq!(waker.strong_count(), 1); // waker was not registered because of panic
}
#[cfg(not(loom))]
#[rstest]
fn drop_panic_in_register_overwrite_cached_can_be_recovered<const SYNC: bool>(
    #[values(FALSE, TRUE)] _sync: Bool<SYNC>,
) {
    let spmc = SpmcWaker::<SYNC, true>::new();
    assert!(unsafe { spmc.register(&panic_on_drop()) }); // panic recovery always let the waker unregistered
    spmc.wake();
    let waker = CounterWaker::new();
    check_panic_recovered(spmc, |spmc| unsafe { spmc.register(&waker.waker()) });
    assert_eq!(waker.strong_count(), 1); // waker was not registered because of panic
}

#[cfg(not(loom))]
#[rstest]
fn wake_panic_can_be_recovered<const SYNC: bool, const CACHED: bool>(
    #[values(FALSE, TRUE)] _sync: Bool<SYNC>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    let spmc = SpmcWaker::<SYNC, CACHED>::new();
    unsafe { spmc.register(&panic_on_wake()) };
    check_panic_recovered(spmc, |spmc| spmc.wake());
}
