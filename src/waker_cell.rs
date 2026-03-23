use core::{
    mem::{ManuallyDrop, MaybeUninit},
    task::{RawWakerVTable, Waker},
};

use crate::{
    WakerRef,
    loom::{UnsafeCell, UnsafeCellExt},
};

/// A sort of `UnsafeCell<MaybeUninit<Waker>>`. Because waker cell can be accessed to call
/// `will_wake` while a concurrent thread is waking the waker by value, storing `Waker`
/// directly would mean to rely on its internal behavior of being copy-safe.
///
/// Instead, we store `Waker` components, which are copy-safe, and use our own
/// `will_wake`. Even if miri was not complaining with `UnsafeCell<MaybeUninit<Waker>>`,
/// it still helps to simplify `SpmcWaker` code a bit.
#[derive(Debug)]
pub(super) struct WakerCell(UnsafeCell<MaybeUninit<(*const (), &'static RawWakerVTable)>>);

impl WakerCell {
    #[cfg_attr(loom, const_fn::const_fn(cfg(false)))]
    pub(super) const fn new() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }

    /// # Safety
    ///
    /// The cell must be safe to access mutably.
    pub(super) unsafe fn set(&self, waker: impl WakerRef) {
        let waker = ManuallyDrop::new(waker.into_waker());
        unsafe {
            self.0.with_ref_mut(|cell| {
                cell.write((waker.data(), waker.vtable()));
            });
        };
    }

    /// # Safety
    ///
    /// The cell must be safe to access immutably.
    pub(super) unsafe fn will_wake(&self, waker: &impl WakerRef) -> bool {
        let waker = waker.as_waker();
        unsafe {
            self.0.with_ref(|cell| {
                let (data, vtable) = cell.assume_init_read();
                waker.data() == data && waker.vtable() == vtable
            })
        }
    }

    /// # Safety
    ///
    /// The cell must be safe to access immutably, and the method
    /// must be called exactly once before a new `set` call.
    pub(super) unsafe fn take(&self) -> Waker {
        unsafe {
            self.0.with_ref(|cell| {
                let (data, vtable) = cell.assume_init_read();
                Waker::new(data, vtable)
            })
        }
    }
}
