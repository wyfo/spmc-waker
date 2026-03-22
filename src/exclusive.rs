use core::{
    cell::Cell,
    panic::Location,
    sync::atomic::{
        AtomicBool,
        Ordering::{Acquire, Relaxed, Release},
    },
};

#[derive(Debug)]
pub(crate) struct Exclusive {
    lock: AtomicBool,
    location: Cell<Option<&'static Location<'static>>>,
}

unsafe impl Send for Exclusive {}
unsafe impl Sync for Exclusive {}

impl Exclusive {
    pub(crate) const fn new() -> Self {
        Self {
            lock: AtomicBool::new(false),
            location: Cell::new(None),
        }
    }

    fn with_loc(&self, f: impl FnOnce(&Cell<Option<&'static Location<'static>>>)) {
        struct Guard<'a>(&'a AtomicBool);
        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                self.0.store(false, Release);
            }
        }
        while ((self.lock).compare_exchange_weak(false, true, Acquire, Relaxed)).is_err() {}
        let _guard = Guard(&self.lock);
        f(&self.location);
    }

    #[track_caller]
    pub(crate) fn check(&self) -> ExclusiveGuard<'_> {
        let location = Location::caller();
        self.with_loc(|loc|{
            match loc.get() {
                Some(loc) => panic!(
                    "concurrent access detected:\n  previous access at {loc}\n  current access at {location}"
                ),
                None => loc.set(Some(location)),
            }
        });
        ExclusiveGuard(self)
    }
}

pub(crate) struct ExclusiveGuard<'a>(&'a Exclusive);

impl Drop for ExclusiveGuard<'_> {
    fn drop(&mut self) {
        self.0.with_loc(|loc| loc.set(None));
    }
}
