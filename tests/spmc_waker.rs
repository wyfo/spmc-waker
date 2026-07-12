#[cfg(all(loom, not(debug_assertions)))]
compile_error!("loom tests requires debug_assertions enabled");

use std::{
    future::poll_fn,
    marker::PhantomData,
    ptr,
    sync::{
        Arc, OnceLock,
        atomic::Ordering::{Acquire, Relaxed, Release, SeqCst},
    },
    task::{RawWaker, RawWakerVTable, Wake, Waker},
};
#[cfg(not(loom))]
use std::{
    hint::spin_loop,
    panic::catch_unwind,
    sync::atomic::{AtomicUsize, fence},
    task::Poll,
    thread,
};

#[cfg(not(loom))]
use futures::executor::block_on;
#[cfg(loom)]
use loom::hint::spin_loop;
use rstest::rstest;
#[cfg(loom)]
use spmc_waker::loom::{AtomicUsize, fence};
use spmc_waker::{
    SpmcWaker,
    registration::{Lenient, RegistrationPolicy, Strict, Unchecked},
    synchronization::{Sequential, Synchronization, Synchronized, Unsynchronized},
    wait_until::WakeCondition,
};

#[cfg(loom)]
fn model(f: impl Fn() + Sync + Send + 'static) {
    loom::model(move || {
        spmc_waker::loom::clear_trace();
        f()
    });
}

#[cfg(loom)]
mod thread {
    use std::{
        cell::RefCell,
        marker::PhantomData,
        panic::{AssertUnwindSafe, catch_unwind},
    };

    use loom::thread::JoinHandle;
    pub use loom::thread::spawn;

    #[derive(Default)]
    pub struct Scope<'env> {
        handles: RefCell<Vec<Option<JoinHandle<std::thread::Result<()>>>>>,
        dummy: loom::sync::Arc<loom::sync::atomic::AtomicUsize>,
        _env: PhantomData<&'env mut ()>,
    }

    impl Drop for Scope<'_> {
        fn drop(&mut self) {
            for handle in self.handles.get_mut().drain(..).flatten() {
                if let Err(err) = handle.join().unwrap() {
                    std::panic::resume_unwind(err);
                }
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
            let dummy = self.dummy.clone();
            handles.push(Some(spawn(unsafe {
                core::mem::transmute::<
                    Box<dyn FnOnce() -> std::thread::Result<()> + Send + 'env>,
                    Box<dyn FnOnce() -> std::thread::Result<()> + Send + 'static>,
                >(Box::new(move || {
                    // https://github.com/tokio-rs/loom/issues/392
                    dummy.store(1, loom::sync::atomic::Ordering::Relaxed);
                    // https://github.com/tokio-rs/loom/issues/417
                    catch_unwind(AssertUnwindSafe(|| {
                        f();
                    }))
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
        pub fn join(self) -> std::thread::Result<()> {
            self.scope.handles.borrow_mut()[self.handle_idx]
                .take()
                .unwrap()
                .join()
                .unwrap()
        }
    }

    pub fn scope<'env, T>(f: impl FnOnce(&Scope<'env>) -> T) -> T {
        let scope = Scope::default();
        scope.dummy.store(1, loom::sync::atomic::Ordering::Relaxed);
        f(&scope)
    }
}

// loom::future::block_on waker vtable is not stable across clone
// https://github.com/tokio-rs/loom/issues/416
// so here is a wrapper to make it stable
#[cfg(loom)]
fn block_on<F: Future>(f: F) -> F::Output {
    let mut f = core::pin::pin!(f);
    loom::future::block_on(poll_fn(|cx| {
        let waker = cx.waker().clone();
        let mut cx = std::task::Context::from_waker(&waker);
        f.as_mut().poll(&mut cx)
    }))
}

struct SyncMode<S: Synchronization> {
    _sync: PhantomData<S>,
    rmw: bool,
}
impl<S: Synchronization> Copy for SyncMode<S> {}
impl<S: Synchronization> Clone for SyncMode<S> {
    fn clone(&self) -> Self {
        *self
    }
}

const SYNC: SyncMode<Synchronized> = SyncMode {
    _sync: PhantomData,
    rmw: false,
};
const SEQ: SyncMode<Sequential> = SyncMode {
    _sync: PhantomData,
    rmw: false,
};
const UNSYNC: SyncMode<Unsynchronized> = SyncMode {
    _sync: PhantomData,
    rmw: false,
};
const UNSYNC_RMW: SyncMode<Unsynchronized> = SyncMode {
    _sync: PhantomData,
    rmw: true,
};

trait WakeConditionAccess {
    fn add(self, c: &AtomicUsize, v: usize);
    fn get(self, c: &AtomicUsize, registered: bool) -> usize;
}
impl WakeConditionAccess for SyncMode<Synchronized> {
    fn add(self, c: &AtomicUsize, v: usize) {
        c.fetch_add(v, Relaxed);
    }
    fn get(self, c: &AtomicUsize, _registered: bool) -> usize {
        c.load(Relaxed)
    }
}
impl WakeConditionAccess for SyncMode<Sequential> {
    fn add(self, c: &AtomicUsize, v: usize) {
        c.fetch_add(v, SeqCst);
    }
    fn get(self, c: &AtomicUsize, registered: bool) -> usize {
        if registered {
            c.load(SeqCst)
        } else {
            c.load(Relaxed)
        }
    }
}
impl WakeConditionAccess for SyncMode<Unsynchronized> {
    fn add(self, c: &AtomicUsize, v: usize) {
        if self.rmw {
            c.fetch_add(v, Acquire);
        } else {
            c.fetch_add(v, Relaxed);
            fence(SeqCst);
        }
    }
    fn get(self, c: &AtomicUsize, registered: bool) -> usize {
        if self.rmw {
            if registered {
                c.fetch_add(0, Release)
            } else {
                c.load(Relaxed)
            }
        } else {
            if registered {
                fence(SeqCst);
            }
            c.load(Relaxed)
        }
    }
}

#[derive(Clone, Copy)]
struct Caching<const BOOL: bool>;
const NO_CACHING: Caching<false> = Caching;
const CACHING: Caching<true> = Caching;

struct RegistrationMode<R: RegistrationPolicy>(PhantomData<R>);
const STRICT: RegistrationMode<Strict> = RegistrationMode(PhantomData);
const LENIENT: RegistrationMode<Lenient> = RegistrationMode(PhantomData);
const UNCHECKED: RegistrationMode<Unchecked> = RegistrationMode(PhantomData);

#[derive(Clone, Copy)]
enum WaitMode {
    Normal,
    Minimal,
}

trait SpmcWakerExt<S: Synchronization, const CACHING: bool, R: RegistrationPolicy> {
    async fn wait_until2<F: FnMut(bool) -> W, W: WakeCondition + Default>(
        &self,
        mode: WaitMode,
        wake_condition: F,
    ) -> W::Output;
    fn register2(&self, waker: &Waker);
}

impl<S: Synchronization, const CACHING: bool, R: RegistrationPolicy> SpmcWakerExt<S, CACHING, R>
    for SpmcWaker<S, CACHING, R>
{
    async fn wait_until2<F: FnMut(bool) -> W, W: WakeCondition + Default>(
        &self,
        mode: WaitMode,
        mut wake_condition: F,
    ) -> W::Output {
        let wake_condition = |registered: bool| match mode {
            WaitMode::Minimal if !registered => W::default(),
            _ => wake_condition(registered),
        };
        unsafe { R::wait_until(self, wake_condition) }.await
    }
    fn register2(&self, waker: &Waker) {
        unsafe { R::register(self, waker) };
    }
}

#[cfg(not(loom))]
fn model(f: impl Fn() + Sync + Send + 'static) {
    f();
}

#[derive(Default)]
struct TestWaker(AtomicUsize);
impl TestWaker {
    fn new() -> Arc<Self> {
        Default::default()
    }
    fn waker(self: &Arc<Self>) -> Waker {
        self.clone().into()
    }
    fn wake_count(&self) -> usize {
        self.0.load(Relaxed)
    }
}
impl Wake for TestWaker {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Relaxed);
    }
}

#[rstest]
fn it_works<S: Synchronization, const CACHING: bool, R: RegistrationPolicy>(
    #[values(SYNC, SEQ, UNSYNC, UNSYNC_RMW)] sync: SyncMode<S>,
    #[values(NO_CACHING, CACHING)] _caching: Caching<CACHING>,
    #[values(STRICT, UNCHECKED)] _reg: RegistrationMode<R>,
) where
    SyncMode<S>: WakeConditionAccess,
{
    model(move || {
        let spmc = SpmcWaker::<S, CACHING, R>::new();
        let waker = TestWaker::new();
        let wake_cond = AtomicUsize::new(0);
        let wake_cond_loaded = OnceLock::new();
        thread::scope(|s| {
            s.spawn(|| {
                sync.add(&wake_cond, 1);
                spmc.wake();
            });
            s.spawn(|| {
                spmc.register2(&waker.waker());
                let _ = wake_cond_loaded.set(sync.get(&wake_cond, true));
            });
        });
        assert!(*wake_cond_loaded.wait() == 1 || waker.wake_count() == 1);
    });
}

#[rstest]
fn concurrent_register_and_wake<S: Synchronization, const CACHING: bool, R: RegistrationPolicy>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(NO_CACHING, CACHING)] _caching: Caching<CACHING>,
    #[values(STRICT, UNCHECKED)] _reg: RegistrationMode<R>,
) {
    model(move || {
        let spmc = SpmcWaker::<S, CACHING, R>::new();
        let waker = TestWaker::new();
        thread::scope(|s| {
            s.spawn(|| spmc.register2(&waker.waker()));
            s.spawn(|| spmc.wake());
        });
        assert!(waker.wake_count() == 1 || spmc.registered().is_some());
    });
}

#[rstest]
fn register_synchronizes_with_wake<const CACHING: bool, R: RegistrationPolicy>(
    #[values(NO_CACHING, CACHING)] _caching: Caching<CACHING>,
    #[values(STRICT, UNCHECKED)] _reg: RegistrationMode<R>,
    #[values(false, true)] same_waker: bool,
) {
    model(move || {
        let spmc = SpmcWaker::<Synchronized, CACHING, R>::new();
        let wake_cond = AtomicUsize::new(0);
        let wake_cond_loaded = OnceLock::new();
        let waker = TestWaker::new().waker();
        let other_waker = if same_waker {
            waker.clone()
        } else {
            TestWaker::new().waker()
        };
        spmc.register2(&waker);
        thread::scope(|s| {
            s.spawn(|| {
                wake_cond.store(1, Relaxed);
                spmc.wake();
            });
            s.spawn(|| {
                spmc.register2(&other_waker);
                let _ = wake_cond_loaded.set(wake_cond.load(Relaxed));
            });
        });
        if spmc.registered().is_some() {
            assert_eq!(*wake_cond_loaded.wait(), 1);
        }
    });
}

#[rstest]
fn wait_until<S: Synchronization, const CACHING: bool, R: RegistrationPolicy>(
    #[values(SYNC, SEQ, UNSYNC, UNSYNC_RMW)] sync: SyncMode<S>,
    #[values(NO_CACHING, CACHING)] _caching: Caching<CACHING>,
    #[values(STRICT, UNCHECKED)] _reg: RegistrationMode<R>,
    #[values(WaitMode::Normal, WaitMode::Minimal)] wait_mode: WaitMode,
) where
    SyncMode<S>: WakeConditionAccess,
{
    model(move || {
        let spmc = SpmcWaker::<S, CACHING, R>::new();
        let wake_condition = AtomicUsize::new(0);
        thread::scope(|s| {
            s.spawn(|| {
                sync.add(&wake_condition, 1);
                spmc.wake();
            });
            s.spawn(|| {
                assert!(block_on(spmc.wait_until2(wait_mode, |registered| {
                    (sync.get(&wake_condition, registered) == 1).then_some(true)
                })));
            });
        });
    });
}

#[rstest]
#[case::strict(STRICT, true)]
#[case::lenient(LENIENT, false)]
fn conflicting_registrations<R: RegistrationPolicy>(
    #[case] _reg: RegistrationMode<R>,
    #[case] strict: bool,
) {
    model(move || {
        fn waker(atomic: &AtomicUsize) -> Waker {
            fn clone(atomic: *const ()) -> RawWaker {
                while unsafe { (*atomic.cast::<AtomicUsize>()).load(Relaxed) == 0 } {
                    spin_loop();
                }
                unsafe { (*atomic.cast::<AtomicUsize>()).store(2, SeqCst) };
                RawWaker::new(atomic, VTABLE)
            }
            const VTABLE: &RawWakerVTable = &RawWakerVTable::new(clone, |_| (), |_| (), |_| ());
            unsafe { Waker::new(ptr::from_ref(atomic).cast(), VTABLE) }
        }
        struct StopGuard<'a>(&'a AtomicUsize);
        impl Drop for StopGuard<'_> {
            fn drop(&mut self) {
                self.0.store(1, SeqCst);
            }
        }
        let spmc = SpmcWaker::<Synchronized, false, R>::new();
        let atomic1 = AtomicUsize::new(0);
        let atomic2 = AtomicUsize::new(0);
        let waker1 = waker(&atomic1);
        let waker2 = waker(&atomic2);
        let (res1, res2) = thread::scope(|s| {
            let t1 = s.spawn(|| {
                let _guard = StopGuard(&atomic2);
                spmc.register2(&waker1);
            });
            let t2 = s.spawn(|| {
                let _guard = StopGuard(&atomic1);
                spmc.register2(&waker2);
            });
            (t1.join(), t2.join())
        });
        assert_eq!(atomic1.load(SeqCst) + atomic2.load(SeqCst), 3);
        let waker = spmc.take().unwrap().data().cast::<AtomicUsize>();
        assert_eq!(unsafe { (*waker).load(Relaxed) }, 2);
        if strict {
            assert!((res1.is_ok() && waker == &atomic1) ^ (res2.is_ok() && waker == &atomic2));
        } else {
            assert!(res1.is_ok() && res2.is_ok());
        }
    });
}

#[rstest]
fn concurrent_registrations<S: Synchronization, const CACHING: bool>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(NO_CACHING, CACHING)] _caching: Caching<CACHING>,
) {
    model(move || {
        let spmc = SpmcWaker::<S, CACHING, Lenient>::new();
        let waker1 = TestWaker::new().waker();
        let waker2 = TestWaker::new().waker();
        thread::scope(|s| {
            s.spawn(|| {
                spmc.register(&waker1);
                spmc.register(&waker2).unregister();
                spmc.take();
            });
            s.spawn(|| {
                spmc.register(&waker1);
                spmc.register(&waker2).unregister();
                spmc.take();
            });
        });
    });
}

#[rstest]
fn reregistration<S: Synchronization, const CACHING: bool, R: RegistrationPolicy>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(NO_CACHING, CACHING)] _caching: Caching<CACHING>,
    #[values(STRICT, UNCHECKED)] _reg: RegistrationMode<R>,
) {
    model(move || {
        let spmc = SpmcWaker::<S, CACHING, R>::new();
        let waker1 = TestWaker::new().waker();
        let waker2 = TestWaker::new().waker();
        spmc.register2(&waker1);
        thread::scope(|s| {
            s.spawn(|| {
                spmc.register2(&waker1);
                spmc.register2(&waker2);
            });
            s.spawn(|| {
                spmc.wake();
            });
            s.spawn(|| {
                spmc.wake();
            });
        });
    });
}

// From futures test suite, only tested with SYNC because it requires internal synchronization
#[cfg(not(loom))]
#[rstest]
fn basic<const CACHING: bool, R: RegistrationPolicy>(
    #[values(NO_CACHING, CACHING)] _caching: Caching<CACHING>,
    #[values(STRICT, UNCHECKED)] _reg: RegistrationMode<R>,
) {
    let atomic_waker = Arc::new(SpmcWaker::<Synchronized, CACHING, R>::new());
    let atomic_waker_copy = atomic_waker.clone();

    let returned_pending = Arc::new(AtomicUsize::new(0));
    let returned_pending_copy = returned_pending.clone();

    let woken = Arc::new(AtomicUsize::new(0));
    let woken_copy = woken.clone();

    let t = thread::spawn(move || {
        let mut pending_count = 0;

        block_on(poll_fn(move |cx| {
            // the waking condition is not checked after registration so registered=true is passed
            if woken_copy.load(Relaxed) == 1 {
                Poll::Ready(())
            } else {
                // Assert we return pending exactly once
                assert_eq!(0, pending_count);
                pending_count += 1;
                atomic_waker_copy.register2(cx.waker());

                returned_pending_copy.store(1, Relaxed);

                Poll::Pending
            }
        }));
    });

    while returned_pending.load(Relaxed) == 0 {}

    // give spawned thread some time to sleep in `block_on`
    thread::yield_now();

    woken.store(1, Relaxed);
    atomic_waker.wake();

    t.join().unwrap();
}

// From tokio test suite
#[rstest]
fn basic_notification<S: Synchronization, const CACHING: bool, R: RegistrationPolicy>(
    #[values(SYNC, SEQ, UNSYNC, UNSYNC_RMW)] sync: SyncMode<S>,
    #[values(NO_CACHING, CACHING)] _caching: Caching<CACHING>,
    #[values(STRICT, UNCHECKED)] _reg: RegistrationMode<R>,
    #[values(WaitMode::Normal, WaitMode::Minimal)] wait_mode: WaitMode,
) where
    SyncMode<S>: WakeConditionAccess,
{
    struct Chan<S: Synchronization, const C: bool, R: RegistrationPolicy> {
        num: AtomicUsize,
        task: SpmcWaker<S, C, R>,
    }
    const NUM_NOTIFY: usize = 2;
    model(move || {
        let chan = Chan::<S, CACHING, R> {
            num: AtomicUsize::new(0),
            task: SpmcWaker::new(),
        };
        thread::scope(|s| {
            for _ in 0..NUM_NOTIFY {
                s.spawn(|| {
                    sync.add(&chan.num, 1);
                    chan.task.wake();
                });
            }
            s.spawn(|| {
                block_on(chan.task.wait_until2(wait_mode, |registered| {
                    sync.get(&chan.num, registered) == NUM_NOTIFY
                }));
            });
        });
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
fn check_panic_recovered<S: Synchronization, const CACHING: bool, R: RegistrationPolicy, T>(
    spmc: SpmcWaker<S, CACHING, R>,
    op: impl FnOnce(&SpmcWaker<S, CACHING, R>) -> T + std::panic::UnwindSafe,
) {
    assert!(catch_unwind(|| op(&spmc)).is_err());
    let waker = TestWaker::new();
    spmc.register2(&waker.waker());
    assert!(spmc.registered().is_some());
    spmc.wake();
    assert_eq!(waker.wake_count(), 1);
    drop(spmc);
    assert_eq!(Arc::strong_count(&waker), 1);
}

#[cfg(not(loom))]
#[rstest]
fn clone_panic_in_register_can_be_recovered<
    S: Synchronization,
    const CACHING: bool,
    R: RegistrationPolicy,
>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(NO_CACHING, CACHING)] _caching: Caching<CACHING>,
    #[values(STRICT, UNCHECKED)] _reg: RegistrationMode<R>,
) {
    let spmc = SpmcWaker::<S, CACHING, R>::new();
    check_panic_recovered(spmc, |spmc| spmc.register2(&panic_on_clone()));
}

#[cfg(not(loom))]
#[rstest]
fn clone_panic_in_register_overwrite_can_be_recovered<
    S: Synchronization,
    const CACHING: bool,
    R: RegistrationPolicy,
>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(NO_CACHING, CACHING)] _caching: Caching<CACHING>,
    #[values(STRICT, UNCHECKED)] _reg: RegistrationMode<R>,
) {
    let spmc = SpmcWaker::<S, CACHING, R>::new();
    spmc.register2(Waker::noop());
    check_panic_recovered(spmc, |spmc| spmc.register2(&panic_on_clone()));
}

#[cfg(not(loom))]
#[rstest]
fn drop_panic_in_unregister_can_be_recovered<S: Synchronization, R: RegistrationPolicy>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(STRICT, UNCHECKED)] _reg: RegistrationMode<R>,
) {
    let spmc = SpmcWaker::<S, false, R>::new();
    spmc.register2(&panic_on_drop());
    check_panic_recovered(spmc, |spmc| spmc.registered().unwrap().unregister());
}

#[cfg(not(loom))]
#[rstest]
fn drop_panic_in_register_overwrite_can_be_recovered<
    S: Synchronization,
    const CACHING: bool,
    R: RegistrationPolicy,
>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(NO_CACHING, CACHING)] _caching: Caching<CACHING>,
    #[values(STRICT, UNCHECKED)] _reg: RegistrationMode<R>,
) {
    let spmc = SpmcWaker::<S, CACHING, R>::new();
    spmc.register2(&panic_on_drop());
    check_panic_recovered(spmc, |spmc| spmc.register2(Waker::noop()));
}
#[cfg(not(loom))]
#[rstest]
fn drop_panic_in_register_overwrite_cached_can_be_recovered<
    S: Synchronization,
    R: RegistrationPolicy,
>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(STRICT, UNCHECKED)] _reg: RegistrationMode<R>,
) {
    let spmc = SpmcWaker::<S, true, R>::new();
    spmc.register2(&panic_on_drop());
    spmc.wake();
    check_panic_recovered(spmc, |spmc| spmc.register2(Waker::noop()));
}

#[cfg(not(loom))]
#[rstest]
fn wake_panic_can_be_recovered<S: Synchronization, const CACHING: bool, R: RegistrationPolicy>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(NO_CACHING, CACHING)] _caching: Caching<CACHING>,
    #[values(STRICT, UNCHECKED)] _reg: RegistrationMode<R>,
) {
    let spmc = SpmcWaker::<S, CACHING, R>::new();
    spmc.register2(&panic_on_wake());
    check_panic_recovered(spmc, |spmc| spmc.wake());
}
