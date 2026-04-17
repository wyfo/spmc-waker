use core::{
    mem::ManuallyDrop,
    ptr,
    task::{RawWaker, RawWakerVTable, Waker},
};

use crate::loom::{UnsafeCellExt, cell::UnsafeCell};

static NOOP_VTABLE: &RawWakerVTable = &RawWakerVTable::new(
    |_| RawWaker::new(ptr::null(), NOOP_VTABLE),
    |_| (),
    |_| (),
    |_| (),
);

/// A sort of `UnsafeCell<Waker>`. Because waker cell can be accessed to call
/// `will_wake` while a concurrent thread is waking the waker by value, storing `Waker`
/// directly would mean to rely on its internal behavior of being copy-safe.
///
/// Instead, we store `Waker` components, which are copy-safe, and use our own
/// `will_wake`. Even if miri was not complaining with `UnsafeCell<Waker>`,
/// it still helps to simplify `SpmcWaker` code a bit.
#[derive(Debug)]
pub(super) struct WakerCell(UnsafeCell<(*const (), &'static RawWakerVTable)>);

impl WakerCell {
    #[cfg_attr(loom, const_fn::const_fn(cfg(false)))]
    pub(super) const fn new() -> Self {
        // `Waker::noop` cannot be used here because Waker accessors are not const
        Self(UnsafeCell::new((ptr::null(), NOOP_VTABLE)))
    }

    /// # Safety
    ///
    /// The cell must be safe to access mutably.
    pub(super) unsafe fn set(&self, waker: Waker) {
        let waker = ManuallyDrop::new(waker);
        // SAFETY: as per function contract
        unsafe {
            self.0
                .with_ref_mut(|cell| *cell = (waker.data(), waker.vtable()));
        }
    }

    /// # Safety
    ///
    /// The cell must be safe to access immutably.
    pub(super) unsafe fn will_wake(&self, waker: &Waker) -> bool {
        // SAFETY: as per function contract
        unsafe {
            self.0
                .with_ref(|&(data, vtable)| data == waker.data() && ptr::eq(waker.vtable(), vtable))
        }
    }

    /// # Safety
    ///
    /// The cell must be safe to access immutably.
    pub(super) unsafe fn get(&self) -> ManuallyDrop<Waker> {
        // SAFETY: as per function contract
        unsafe { ManuallyDrop::new(self.0.with_ref(|&(data, vtable)| Waker::new(data, vtable))) }
    }

    /// # Safety
    ///
    /// The cell must be safe to access mutably.
    pub(super) unsafe fn drop(&self) {
        // In fact, mutable access is not needed, but it is done as an additional check.
        // SAFETY: as per function contract
        unsafe { self.0.with_ref_mut(|_| ()) };
        // SAFETY: as per function contract
        drop(ManuallyDrop::into_inner(unsafe { self.get() }));
    }
}
