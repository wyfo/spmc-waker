use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

#[derive(Debug)]
pub(crate) struct Exclusive(AtomicUsize);

impl Exclusive {
    pub(crate) const fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    pub(crate) fn check(&self) -> ExclusiveGuard<'_> {
        assert!(self.0.fetch_add(1, Relaxed) == 0, "concurrent access");
        ExclusiveGuard(self)
    }
}

pub(crate) struct ExclusiveGuard<'a>(&'a Exclusive);

impl Drop for ExclusiveGuard<'_> {
    fn drop(&mut self) {
        if self.0.0.compare_exchange(1, 0, Relaxed, Relaxed).is_err() {
            panic!("concurrent access");
        }
    }
}
