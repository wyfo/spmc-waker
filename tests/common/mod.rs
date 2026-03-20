#[cfg(not(loom))]
use std::thread;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering::Relaxed},
        Arc,
    },
    task::{Wake, Waker},
};

#[cfg(loom)]
use loom::{model, thread};

use super::SpmcWaker;

#[derive(Default)]
struct CounterWaker(AtomicUsize);

impl Wake for CounterWaker {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Relaxed);
    }
}

#[cfg(not(loom))]
fn model(f: impl FnOnce()) {
    f()
}

fn concurrent_register_and_wake(spmc: SpmcWaker, arc: &Arc<CounterWaker>) {
    let waker = Waker::from(arc.clone());
    #[cfg(not(loom))]
    let registered = thread::scope(|s| {
        s.spawn(|| spmc.wake());
        s.spawn(|| spmc.wake());
        s.spawn(|| unsafe { spmc.register(waker) }).join().unwrap()
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
            move || unsafe { spmc.register(waker) }
        });
        wake1.join().unwrap();
        wake2.join().unwrap();
        register.join().unwrap()
    };
    let wake_count = arc.0.load(Relaxed);
    let waker_count = Arc::strong_count(arc);
    match (registered, wake_count, waker_count) {
        (true, 1, 1) => {}  // register called before wake
        (false, 0, 1) => {} // register raced with wake
        (true, 0, 2) => {}  // register called after wake
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
        unsafe { spmc.register(Waker::from(Arc::<CounterWaker>::default())) };
        concurrent_register_and_wake(spmc, &arc);
    });
}

#[test]
fn concurrent_unregister_and_wake() {
    model(|| {
        let spmc = SpmcWaker::new();
        let arc = Arc::<CounterWaker>::default();
        let waker = Waker::from(arc.clone());
        assert!(unsafe { spmc.register(waker) });
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
        match (unregistered, wake_count) {
            (true, 0) => {}  // unregister called before wake
            (false, 1) => {} // unregister raced with wake or called after wake
            other => panic!("unexpected outcome: {other:?}"),
        }
    });
}
