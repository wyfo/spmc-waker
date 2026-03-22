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
use std::{sync::atomic::AtomicUsize, thread};

#[cfg(not(loom))]
use futures::executor::block_on;
#[cfg(loom)]
use loom::{future::block_on, model, sync::atomic::AtomicUsize, thread};
use spmc_waker::SpmcWaker;

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
        fn noop(_: *const ()) {}
        fn clone(_: *const ()) -> RawWaker {
            #[cfg(not(loom))]
            while !STOP.load(SeqCst) {
                std::hint::spin_loop();
            }
            raw_waker()
        }
        fn raw_waker() -> RawWaker {
            RawWaker::new(ptr::null(), &RawWakerVTable::new(clone, noop, noop, noop))
        }
        struct StopGuard;
        impl Drop for StopGuard {
            fn drop(&mut self) {
                STOP.store(true, SeqCst);
            }
        }
        let spmc = SpmcWaker::<{ super::SYNC }>::new();
        let waker = unsafe { Waker::from_raw(raw_waker()) };
        #[cfg(not(loom))]
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
        #[cfg(loom)]
        {
            let spmc = Arc::new(spmc);
            let waker = Arc::new(waker);
            let t1 = thread::spawn({
                let spmc = spmc.clone();
                let waker = waker.clone();
                move || {
                    let _guard = StopGuard;
                    unsafe { spmc.register(&*waker) };
                }
            });
            let t2 = thread::spawn({
                let spmc = spmc.clone();
                let waker = waker.clone();
                move || {
                    let _guard = StopGuard;
                    unsafe { spmc.register(&*waker) };
                }
            });
            t1.join().unwrap();
            t2.join().unwrap();
        }
    });
}

#[derive(Default)]
struct CounterWaker(AtomicUsize);

impl Wake for CounterWaker {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Relaxed);
    }
}

fn concurrent_try_register_and_wake(spmc: SpmcWaker<{ super::SYNC }>, arc: &Arc<CounterWaker>) {
    let waker = Waker::from(arc.clone());
    #[cfg(not(loom))]
    thread::scope(|s| {
        s.spawn(|| spmc.wake());
        s.spawn(|| spmc.wake());
        s.spawn(|| unsafe { spmc.register(waker) });
    });
    #[cfg(loom)]
    let spmc = Arc::new(spmc);
    #[cfg(loom)]
    {
        let wake1 = thread::spawn({
            let spmc = spmc.clone();
            move || spmc.wake()
        });
        let wake2 = thread::spawn({
            let spmc = spmc.clone();
            move || spmc.wake()
        });
        let register = thread::spawn({
            let spmc = spmc.clone();
            move || unsafe { spmc.register(waker) }
        });
        wake1.join().unwrap();
        wake2.join().unwrap();
        register.join().unwrap();
    }
    let wake_count = arc.0.load(Relaxed);
    let waker_count = Arc::strong_count(arc);
    match (wake_count, waker_count) {
        (1, 1) => {} // register called before wake or raced with it
        (0, 2) => {} // register called after wake
        other => panic!("unexpected outcome: {other:?}"),
    }
}

#[test]
fn concurrent_try_register_empty_and_wake() {
    model(|| {
        let spmc = SpmcWaker::<{ super::SYNC }>::new();
        let arc = Arc::<CounterWaker>::default();
        concurrent_try_register_and_wake(spmc, &arc);
    });
}

#[test]
fn concurrent_try_register_overwrite_and_wake() {
    model(|| {
        let spmc = SpmcWaker::<{ super::SYNC }>::new();
        let arc = Arc::<CounterWaker>::default();
        unsafe { spmc.register(Waker::from(Arc::<CounterWaker>::default())) };
        concurrent_try_register_and_wake(spmc, &arc);
    });
}

#[test]
fn concurrent_unregister_and_wake() {
    model(|| {
        let spmc = SpmcWaker::<{ super::SYNC }>::new();
        let arc = Arc::<CounterWaker>::default();
        let waker = Waker::from(arc.clone());
        unsafe { spmc.register(waker) };
        #[cfg(not(loom))]
        let unregistered = thread::scope(|s| {
            s.spawn(|| spmc.wake());
            s.spawn(|| spmc.wake());
            s.spawn(|| unsafe { spmc.unregister() }).join().unwrap()
        });
        #[cfg(loom)]
        let spmc = Arc::new(spmc);
        #[cfg(loom)]
        let unregistered = {
            let wake1 = thread::spawn({
                let spmc = spmc.clone();
                move || spmc.wake()
            });
            let wake2 = thread::spawn({
                let spmc = spmc.clone();
                move || spmc.wake()
            });
            let register = thread::spawn({
                let spmc = spmc.clone();
                move || unsafe { spmc.unregister() }
            });
            wake1.join().unwrap();
            wake2.join().unwrap();
            register.join().unwrap()
        };
        let wake_count = arc.0.load(Relaxed);
        match (unregistered.is_some(), wake_count) {
            (true, 0) => {}  // unregister called before wake
            (false, 1) => {} // unregister raced with wake or called after it
            other => panic!("unexpected outcome: {other:?}"),
        }
    });
}

// From futures test suite
#[cfg(not(loom))]
#[test]
fn basic() {
    let ordering = if super::SYNC { Relaxed } else { SeqCst };
    let atomic_waker = Arc::new(SpmcWaker::<{ super::SYNC }>::new());
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
                unsafe { atomic_waker_copy.register(cx.waker()) };

                returned_pending_copy.store(1, Relaxed);

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
#[test]
fn basic_notification() {
    struct Chan {
        num: AtomicUsize,
        task: SpmcWaker<{ super::SYNC }>,
    }

    const NUM_NOTIFY: usize = 2;
    #[cfg(loom)]
    use loom::sync::Arc;

    model(|| {
        let chan = Arc::new(Chan {
            num: AtomicUsize::new(0),
            task: SpmcWaker::<{ super::SYNC }>::new(),
        });

        for _ in 0..NUM_NOTIFY {
            let chan = chan.clone();

            thread::spawn(move || {
                chan.num.fetch_add(1, Relaxed);
                chan.task.wake();
            });
        }

        block_on(poll_fn(move |cx| {
            unsafe { chan.task.register(cx.waker()) };

            let n = if super::SYNC {
                chan.num.load(Relaxed)
            } else {
                chan.num.fetch_add(0, Relaxed)
            };
            if NUM_NOTIFY == n {
                return Poll::Ready(());
            }

            Poll::Pending
        }));
    });
}
