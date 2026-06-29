use std::{
    future::poll_fn,
    marker::PhantomData,
    ptr,
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            Ordering::{Acquire, Relaxed, Release, SeqCst},
        },
    },
    task::{Context, Poll, RawWaker, RawWakerVTable, Wake, Waker},
};
#[cfg(not(loom))]
use std::{
    panic::catch_unwind,
    sync::atomic::{AtomicUsize, fence},
    thread,
};

#[cfg(not(loom))]
use futures::executor::block_on;
#[cfg(loom)]
use loom::sync::atomic::{AtomicUsize, fence};
use rstest::rstest;
use spmc_waker::{Sequential, SpmcWaker, Synchronization, Synchronized, Unsynchronized};

#[cfg(loom)]
fn model(f: impl Fn() + Sync + Send + 'static) {
    loom::model(move || {
        spmc_waker::clear_loom_trace();
        f()
    });
}

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
        pub fn join(self) -> std::thread::Result<()> {
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

// loom::future::block_on waker vtable is not stable across clone
// https://github.com/tokio-rs/loom/issues/416
// so here is a wrapper to make it stable
#[cfg(loom)]
fn block_on<F: Future>(f: F) -> F::Output {
    let mut f = core::pin::pin!(f);
    loom::future::block_on(poll_fn(|cx| {
        let waker = cx.waker().clone();
        let mut cx = Context::from_waker(&waker);
        f.as_mut().poll(&mut cx)
    }))
}

struct SyncMode<S: Synchronization, const RMW: bool = false>(PhantomData<S>);
const SYNC: SyncMode<Synchronized> = SyncMode(PhantomData);
const SEQ: SyncMode<Sequential> = SyncMode(PhantomData);
const UNSYNC: SyncMode<Unsynchronized> = SyncMode(PhantomData);
const UNSYNC_RMW: SyncMode<Unsynchronized, true> = SyncMode(PhantomData);

trait WakeConditionAccess {
    fn add(c: &AtomicUsize, v: usize);
    fn get(c: &AtomicUsize, registered: bool) -> usize;
}
impl WakeConditionAccess for SyncMode<Synchronized> {
    fn add(c: &AtomicUsize, v: usize) {
        c.fetch_add(v, Relaxed);
    }
    fn get(c: &AtomicUsize, _registered: bool) -> usize {
        c.load(Relaxed)
    }
}
impl WakeConditionAccess for SyncMode<Sequential> {
    fn add(c: &AtomicUsize, v: usize) {
        c.fetch_add(v, SeqCst);
    }
    fn get(c: &AtomicUsize, registered: bool) -> usize {
        if registered {
            c.load(SeqCst)
        } else {
            c.load(Relaxed)
        }
    }
}
impl WakeConditionAccess for SyncMode<Unsynchronized, false> {
    fn add(c: &AtomicUsize, v: usize) {
        c.fetch_add(v, Relaxed);
        fence(SeqCst);
    }
    fn get(c: &AtomicUsize, registered: bool) -> usize {
        if registered {
            fence(SeqCst);
        }
        c.load(Relaxed)
    }
}
impl WakeConditionAccess for SyncMode<Unsynchronized, true> {
    fn add(c: &AtomicUsize, v: usize) {
        c.fetch_add(v, Acquire);
    }
    fn get(c: &AtomicUsize, registered: bool) -> usize {
        if registered {
            c.fetch_add(0, Release)
        } else {
            c.load(Relaxed)
        }
    }
}

struct Bool<const BOOL: bool>;
const TRUE: Bool<true> = Bool::<true>;
const FALSE: Bool<false> = Bool::<false>;

#[derive(Clone, Copy)]
enum WaitUntilMode {
    Normal,
    TryRegisterOnly,
    RegisterOnly,
}

#[derive(Clone, Copy)]
enum WakeMode {
    Normal,
    Cold,
    CheckBefore,
}

trait PollWaitUntilExt {
    unsafe fn poll_wait_until2(
        &self,
        cx: &mut Context,
        predicate: impl FnMut(bool) -> bool,
        mode: WaitUntilMode,
    ) -> Poll<()>;
    fn wake2(&self, mode: WakeMode);
}

impl<S: Synchronization, const CACHED: bool> PollWaitUntilExt for SpmcWaker<S, CACHED> {
    unsafe fn poll_wait_until2(
        &self,
        cx: &mut Context,
        mut predicate: impl FnMut(bool) -> bool,
        mode: WaitUntilMode,
    ) -> Poll<()> {
        match mode {
            WaitUntilMode::Normal => unsafe { self.poll_wait_until(cx, predicate) },
            WaitUntilMode::TryRegisterOnly => {
                if predicate(false) {
                    return Poll::Ready(());
                }
                let registered = unsafe { self.try_register(cx.waker()) };
                if predicate(registered) {
                    unsafe { self.unregister() };
                    return Poll::Ready(());
                } else if !registered {
                    #[cfg(loom)]
                    loom::hint::spin_loop();
                    cx.waker().wake_by_ref();
                }
                Poll::Pending
            }
            WaitUntilMode::RegisterOnly => {
                if predicate(false) {
                    return Poll::Ready(());
                }
                unsafe { self.register(cx.waker()) };
                if predicate(true) {
                    unsafe { self.unregister() };
                    return Poll::Ready(());
                }
                Poll::Pending
            }
        }
    }
    fn wake2(&self, mode: WakeMode) {
        match mode {
            WakeMode::Normal => self.wake(),
            WakeMode::Cold => self.wake_cold(),
            WakeMode::CheckBefore => {
                if self.has_waker_registered() {
                    self.wake();
                }
            }
        }
    }
}

#[cfg(not(loom))]
fn model(f: impl Fn() + Sync + Send + 'static) {
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
        self.clone().into()
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

fn concurrent_try_register_and_wake<S: Synchronization, const CACHED: bool>(
    spmc: SpmcWaker<S, CACHED>,
    waker: &Arc<CounterWaker>,
    wake_mode: WakeMode,
) {
    let registered = thread::scope(|s| {
        s.spawn(|| spmc.wake2(wake_mode));
        s.spawn(|| spmc.wake2(wake_mode));
        unsafe { spmc.try_register(&waker.waker()) }
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
fn concurrent_try_register_empty_and_wake<S: Synchronization, const CACHED: bool>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
    #[values(WakeMode::Normal, WakeMode::Cold, WakeMode::CheckBefore)] wake_mode: WakeMode,
) {
    model(move || {
        let spmc = SpmcWaker::<S, CACHED>::new();
        let waker = CounterWaker::new();
        concurrent_try_register_and_wake(spmc, &waker, wake_mode);
    });
}

#[rstest]
fn concurrent_try_register_overwrite_and_wake<S: Synchronization, const CACHED: bool>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
    #[values(WakeMode::Normal, WakeMode::Cold, WakeMode::CheckBefore)] wake_mode: WakeMode,
) {
    model(move || {
        let spmc = SpmcWaker::<S, CACHED>::new();
        let waker = CounterWaker::new();
        unsafe { spmc.register(Waker::noop()) };
        concurrent_try_register_and_wake(spmc, &waker, wake_mode);
    });
}

#[rstest]
fn concurrent_unregister_and_wake<S: Synchronization, const CACHED: bool>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
    #[values(WakeMode::Normal, WakeMode::Cold, WakeMode::CheckBefore)] wake_mode: WakeMode,
) {
    model(move || {
        let spmc = SpmcWaker::<S, CACHED>::new();
        let waker = Arc::<CounterWaker>::default();
        unsafe { spmc.register(&waker.waker()) };
        let unregistered = thread::scope(|s| {
            s.spawn(|| spmc.wake2(wake_mode));
            s.spawn(|| spmc.wake2(wake_mode));
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
fn concurrent_overwrite_and_wake<S: Synchronization, const CACHED: bool>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
    #[values(WakeMode::Normal, WakeMode::Cold, WakeMode::CheckBefore)] wake_mode: WakeMode,
) {
    model(move || {
        let spmc = SpmcWaker::<S, CACHED>::new();
        let waker1 = CounterWaker::new();
        let waker2 = CounterWaker::new();
        unsafe { spmc.register(&waker1.waker()) };
        thread::scope(|s| {
            s.spawn(|| spmc.wake2(wake_mode));
            unsafe { spmc.try_register(&waker2.waker()) };
        });
        assert!(waker1.wake_count() + waker2.wake_count() <= 1);
    });
}

#[rstest]
fn concurrent_register_and_wake<S: Synchronization, const CACHED: bool>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
    #[values(WakeMode::Normal, WakeMode::Cold, WakeMode::CheckBefore)] wake_mode: WakeMode,
) {
    model(move || {
        let spmc = SpmcWaker::<S, CACHED>::new();
        let waker = CounterWaker::new();
        thread::scope(|s| {
            s.spawn(|| spmc.wake2(wake_mode));
            unsafe { spmc.register(&waker.waker()) };
        });
        assert!(waker.wake_count() == 1 || spmc.has_waker_registered());
    });
}

#[rstest]
fn register_synchronizes_with_wake<const CACHED: bool>(
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
    #[values(WakeMode::Normal, WakeMode::Cold, WakeMode::CheckBefore)] wake_mode: WakeMode,
    #[values(false, true)] try_register: bool,
    #[values(false, true)] same_waker: bool,
) {
    model(move || {
        let spmc = SpmcWaker::<Synchronized, CACHED>::new();
        let condition = AtomicUsize::new(0);
        let waker = CounterWaker::new().waker();
        let other_waker = if same_waker {
            waker.clone()
        } else {
            CounterWaker::new().waker()
        };
        unsafe { spmc.register(&waker) };
        let (registered, loaded) = thread::scope(|s| {
            s.spawn(|| {
                condition.store(1, Relaxed);
                spmc.wake2(wake_mode);
            });
            #[cfg(loom)] // https://github.com/tokio-rs/loom/issues/392
            condition.load(Relaxed);
            let registered = if try_register {
                unsafe { spmc.try_register(&other_waker) }
            } else {
                unsafe { spmc.register(&other_waker) };
                true
            };
            (registered, condition.load(Relaxed))
        });
        if !registered || spmc.has_waker_registered() {
            assert_eq!(loaded, 1);
        }
    });
}

// Ensure that
#[test]
fn unsynchronized_cached_reregister_synchronizes_data() {
    model(|| {
        let spmc = SpmcWaker::<Unsynchronized, true>::new();
        thread::scope(|s| {
            s.spawn(|| {
                unsafe { spmc.register(Waker::noop()) };
                unsafe { spmc.unregister() };
                unsafe { spmc.register(Waker::noop()) };
            });
            s.spawn(|| spmc.wake());
        });
    });
}

// From futures test suite
#[cfg(not(loom))]
#[rstest]
fn basic<S: Synchronization, const RMW: bool, const CACHED: bool>(
    #[values(SYNC, SEQ, UNSYNC, UNSYNC_RMW)] _sync: SyncMode<S, RMW>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
    #[values(WakeMode::Normal, WakeMode::Cold, WakeMode::CheckBefore)] wake_mode: WakeMode,
) where
    SyncMode<S, RMW>: WakeConditionAccess,
{
    let atomic_waker = Arc::new(SpmcWaker::<S, CACHED>::new());
    let atomic_waker_copy = atomic_waker.clone();

    let returned_pending = Arc::new(AtomicUsize::new(0));
    let returned_pending_copy = returned_pending.clone();

    let woken = Arc::new(AtomicUsize::new(0));
    let woken_copy = woken.clone();

    let t = thread::spawn(move || {
        let mut pending_count = 0;

        block_on(poll_fn(move |cx| {
            // the waking condition is not checked after registration so registered=true is passed
            if SyncMode::<S, RMW>::get(&woken_copy, true) == 1 {
                Poll::Ready(())
            } else {
                // Assert we return pending exactly once
                assert_eq!(0, pending_count);
                pending_count += 1;
                unsafe { atomic_waker_copy.register(cx.waker()) };

                returned_pending_copy.store(1, Relaxed);

                Poll::Pending
            }
        }));
    });

    while returned_pending.load(Relaxed) == 0 {}

    // give spawned thread some time to sleep in `block_on`
    thread::yield_now();

    SyncMode::<S, RMW>::add(&woken, 1);
    atomic_waker.wake2(wake_mode);

    t.join().unwrap();
}

// From tokio test suite
#[rstest]
fn basic_notification<S: Synchronization, const RMW: bool, const CACHED: bool>(
    #[values(SYNC, SEQ, UNSYNC, UNSYNC_RMW)] _sync: SyncMode<S, RMW>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
    #[values(
        WaitUntilMode::Normal,
        WaitUntilMode::TryRegisterOnly,
        WaitUntilMode::RegisterOnly
    )]
    wait_until_mode: WaitUntilMode,
    #[values(WakeMode::Normal, WakeMode::Cold, WakeMode::CheckBefore)] wake_mode: WakeMode,
) where
    SyncMode<S, RMW>: WakeConditionAccess,
{
    struct Chan<S: Synchronization, const C: bool> {
        num: AtomicUsize,
        task: SpmcWaker<S, C>,
    }

    const NUM_NOTIFY: usize = 2;
    #[cfg(loom)]
    use loom::sync::Arc;

    model(move || {
        let chan = Arc::new(Chan::<S, CACHED> {
            num: AtomicUsize::new(0),
            task: SpmcWaker::<S, CACHED>::new(),
        });

        for _ in 0..NUM_NOTIFY {
            let chan = chan.clone();

            thread::spawn(move || {
                SyncMode::<S, RMW>::add(&chan.num, 1);
                chan.task.wake2(wake_mode);
            });
        }

        block_on(poll_fn(move |cx| unsafe {
            chan.task.poll_wait_until2(
                cx,
                |registered| SyncMode::<S, RMW>::get(&chan.num, registered) == NUM_NOTIFY,
                wait_until_mode,
            )
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
fn check_panic_recovered<S: Synchronization, const CACHED: bool, R>(
    spmc: SpmcWaker<S, CACHED>,
    op: impl FnOnce(&SpmcWaker<S, CACHED>) -> R + std::panic::UnwindSafe,
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
fn clone_panic_in_register_can_be_recovered<S: Synchronization, const CACHED: bool>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    let spmc = SpmcWaker::<S, CACHED>::new();
    check_panic_recovered(spmc, |spmc| unsafe { spmc.register(&panic_on_clone()) });
}

#[cfg(not(loom))]
#[rstest]
fn clone_panic_in_register_overwrite_can_be_recovered<S: Synchronization, const CACHED: bool>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    let spmc = SpmcWaker::<S, CACHED>::new();
    unsafe { spmc.register(Waker::noop()) };
    check_panic_recovered(spmc, |spmc| unsafe { spmc.register(&panic_on_clone()) });
}

#[cfg(not(loom))]
#[rstest]
fn drop_panic_in_unregister_can_be_recovered<S: Synchronization>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
) {
    let spmc = SpmcWaker::<S, false>::new();
    unsafe { spmc.register(&panic_on_drop()) };
    check_panic_recovered(spmc, |spmc| unsafe { spmc.unregister() });
}

#[cfg(not(loom))]
#[rstest]
fn drop_panic_in_register_overwrite_can_be_recovered<S: Synchronization, const CACHED: bool>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    let spmc = SpmcWaker::<S, CACHED>::new();
    unsafe { spmc.register(&panic_on_drop()) };
    let waker = CounterWaker::new();
    check_panic_recovered(spmc, |spmc| unsafe { spmc.register(&waker.waker()) });
    assert_eq!(waker.strong_count(), 1); // waker was not registered because of panic
}
#[cfg(not(loom))]
#[rstest]
fn drop_panic_in_register_overwrite_cached_can_be_recovered<S: Synchronization>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
) {
    let spmc = SpmcWaker::<S, true>::new();
    unsafe { spmc.register(&panic_on_drop()) };
    spmc.wake();
    let waker = CounterWaker::new();
    check_panic_recovered(spmc, |spmc| unsafe { spmc.register(&waker.waker()) });
    assert_eq!(waker.strong_count(), 1); // waker was not registered because of panic
}

#[cfg(not(loom))]
#[rstest]
fn wake_panic_can_be_recovered<S: Synchronization, const CACHED: bool>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
    #[values(FALSE, TRUE)] _cached: Bool<CACHED>,
) {
    let spmc = SpmcWaker::<S, CACHED>::new();
    unsafe { spmc.register(&panic_on_wake()) };
    check_panic_recovered(spmc, |spmc| spmc.wake());
}

#[cfg(not(loom))]
struct LeakWaker {
    state: AtomicUsize,
    panic: bool,
}

#[cfg(not(loom))]
impl LeakWaker {
    fn new(panic: bool) -> Arc<Self> {
        Arc::new(Self {
            state: AtomicUsize::new(0),
            panic,
        })
    }
    fn waker(self: &Arc<Self>) -> Waker {
        self.clone().into()
    }
    fn wait_wake(&self) {
        while self.state.load(Relaxed) == 0 {
            std::hint::spin_loop();
        }
    }
    fn end_wake(&self) {
        self.state.store(2, Relaxed);
    }
}

#[cfg(not(loom))]
impl Wake for LeakWaker {
    fn wake(self: Arc<Self>) {
        unreachable!()
    }
    fn wake_by_ref(self: &Arc<Self>) {
        if self.state.compare_exchange(0, 1, Relaxed, Relaxed).is_ok() {
            while self.state.load(Relaxed) == 1 {
                std::hint::spin_loop();
            }
        }
        if self.panic {
            panic!("Waker::wake_by_ref panic");
        }
    }
}

#[cfg(not(loom))]
#[rstest]
fn fallback_waker_leaked_when_main_wake_panics<S: Synchronization>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
) {
    let spmc = SpmcWaker::<S, true>::new();

    let main = LeakWaker::new(true);
    unsafe { spmc.register(&main.waker()) };

    let fallback = LeakWaker::new(false);

    thread::scope(|s| {
        s.spawn(|| {
            assert!(catch_unwind(|| spmc.wake()).is_err());
        });
        main.wait_wake();
        unsafe { spmc.register(&fallback.waker()) };
        main.end_wake();
    });
    assert_eq!(Arc::strong_count(&main), 2);
    assert_eq!(Arc::strong_count(&fallback), 2);
    drop(spmc);
    assert_eq!(Arc::strong_count(&main), 1);
    assert_eq!(Arc::strong_count(&fallback), 2);
    unsafe { Arc::decrement_strong_count(Arc::as_ptr(&fallback)) };
}

#[cfg(not(loom))]
#[rstest]
fn fallback_waker_leaked_when_fallback_wake_panics<S: Synchronization>(
    #[values(SYNC, SEQ, UNSYNC)] _sync: SyncMode<S>,
) {
    let spmc = SpmcWaker::<S, true>::new();

    let main = LeakWaker::new(false);
    unsafe { spmc.register(&main.waker()) };

    let fallback = LeakWaker::new(true);
    fallback.end_wake();

    thread::scope(|s| {
        s.spawn(|| {
            assert!(catch_unwind(|| spmc.wake()).is_err());
        });
        main.wait_wake();
        unsafe { spmc.register(&fallback.waker()) };
        s.spawn(|| spmc.wake()).join().unwrap();
        main.end_wake();
    });
    assert_eq!(Arc::strong_count(&main), 2);
    assert_eq!(Arc::strong_count(&fallback), 2);
    drop(spmc);
    assert_eq!(Arc::strong_count(&main), 1);
    assert_eq!(Arc::strong_count(&fallback), 2);
    unsafe { Arc::decrement_strong_count(Arc::as_ptr(&fallback)) };
}
