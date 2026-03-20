use core::sync::atomic::{AtomicBool, Ordering::Relaxed};

#[derive(Debug)]
pub(crate) struct Exclusive(AtomicBool);

impl Exclusive {
    pub(crate) const fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    pub(crate) fn check(&self) -> ExclusiveGuard<'_> {
        assert!(
            !self.0.swap(true, Relaxed),
            "concurrent register/unregister",
        );
        ExclusiveGuard(self)
    }
}

pub(crate) struct ExclusiveGuard<'a>(&'a Exclusive);

impl Drop for ExclusiveGuard<'_> {
    fn drop(&mut self) {
        self.0.0.store(false, Relaxed);
    }
}
