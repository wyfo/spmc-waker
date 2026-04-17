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

type SpmcWaker = spmc_waker::SpmcWaker<{ super::SYNC }, { super::CACHED }>;

#[cfg(not(loom))]
fn model(f: impl FnOnce()) {
    f();
}

#[cfg(any(debug_assertions, loom))]
#[test]
#[should_panic]
fn exclusive_access() {
    #[cfg(loom)] // loom is not able to detect the data race
    if true {
        panic!();
    }
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
        let spmc = SpmcWaker::new();
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

impl CounterWaker {
    fn waker(self: &Arc<Self>) -> Waker {
        Waker::from(self.clone())
    }
}

impl Wake for CounterWaker {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Relaxed);
    }
}

fn concurrent_register_and_wake(spmc: SpmcWaker, arc: &Arc<CounterWaker>) {
    #[cfg(not(loom))]
    let registered = thread::scope(|s| {
        s.spawn(|| spmc.wake());
        s.spawn(|| spmc.wake());
        s.spawn(|| unsafe { spmc.register(&arc.waker()) })
            .join()
            .unwrap()
    });
    #[cfg(loom)]
    let spmc = Arc::new(spmc);
    #[cfg(loom)]
    let registered = {
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
            let arc = arc.clone();
            move || unsafe { spmc.register(&arc.waker()) }
        });
        wake1.join().unwrap();
        wake2.join().unwrap();
        register.join().unwrap()
    };
    let waker_count = Arc::strong_count(arc) - 1;
    let wake_count = arc.0.load(Relaxed);
    match (wake_count, waker_count, registered) {
        (1, 1, true) if super::CACHED => {} // register called before wake
        (1, 0, true) => {}                  // register called before wake
        (0, 0, false) => {}                 // register raced with wake
        (0, 1, true) => {}                  // register called after wake
        other => panic!("unexpected outcome: {other:?}"),
    }
}

#[test]
fn concurrent_register_empty_and_wake() {
    model(|| {
        let spmc = SpmcWaker::new();
        let arc = Arc::<CounterWaker>::default();
        concurrent_register_and_wake(spmc, &arc);
    });
}

#[test]
fn concurrent_register_overwrite_and_wake() {
    model(|| {
        let spmc = SpmcWaker::new();
        let arc = Arc::<CounterWaker>::default();
        unsafe { spmc.register(Waker::noop()) };
        concurrent_register_and_wake(spmc, &arc);
    });
}

#[test]
fn concurrent_unregister_and_wake() {
    model(|| {
        let spmc = SpmcWaker::new();
        let arc = Arc::<CounterWaker>::default();
        assert!(unsafe { spmc.register(&arc.waker()) });
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
        let waker_count = Arc::strong_count(&arc) - 1;
        assert_eq!(waker_count, if super::CACHED { 1 } else { 0 });
        let wake_count = arc.0.load(Relaxed);
        match (unregistered, wake_count) {
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
    let atomic_waker = Arc::new(SpmcWaker::new());
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
#[test]
fn basic_notification() {
    struct Chan {
        num: AtomicUsize,
        task: SpmcWaker,
    }

    const NUM_NOTIFY: usize = 2;
    #[cfg(loom)]
    use loom::sync::Arc;

    model(|| {
        let chan = Arc::new(Chan {
            num: AtomicUsize::new(0),
            task: SpmcWaker::new(),
        });

        for _ in 0..NUM_NOTIFY {
            let chan = chan.clone();

            thread::spawn(move || {
                chan.num.fetch_add(1, Relaxed);
                chan.task.wake();
            });
        }

        block_on(poll_fn(move |cx| {
            let registered = unsafe { chan.task.register(cx.waker()) };

            let n = if super::SYNC {
                chan.num.load(Relaxed)
            } else {
                chan.num.fetch_add(0, Relaxed)
            };
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
