#[cfg(not(any(loom, miri)))]
use core::hint::assert_unchecked;
use core::task::{RawWakerVTable, Waker};

pub(crate) trait TaggedExt {
    fn has(self, tag: usize) -> bool;
    fn set(self, tag: usize) -> Self;
    fn unset(self, tag: usize) -> Self;
    fn clear(self, tag: usize) -> Self;
}

impl TaggedExt for usize {
    #[inline(always)]
    fn has(self, tag: usize) -> bool {
        self & tag != 0
    }

    #[inline(always)]
    fn set(self, tag: usize) -> Self {
        debug_assert!(!self.has(tag));
        self + tag
    }

    #[inline(always)]
    fn unset(self, tag: usize) -> Self {
        debug_assert!(self.has(tag));
        self - tag
    }

    #[inline(always)]
    fn clear(self, tag: usize) -> Self {
        self & !tag
    }
}

#[cfg(not(any(loom, miri)))]
pub(crate) type ConfirmedWaker = Waker;

/// A maybe-torn loaded waker, which must be claimed by updating `SpmcWaker` state.
pub(crate) struct PendingWaker {
    pub(crate) data: *const (),
    pub(crate) vtable: *const RawWakerVTable,
    #[cfg(any(loom, miri))]
    pub(crate) store_epoch: usize,
    #[cfg(any(loom, miri))]
    pub(crate) waker_cells: *const [crate::Cell<u8>],
}

impl PendingWaker {
    /// Compare the pointers like `Waker::will_wake` does. As it is called in `register`,
    /// it is not possible for the waker to be updated concurrently, so the comparison
    /// makes sense.
    #[inline(always)]
    pub(crate) fn will_wake(&self, waker: &Waker) -> bool {
        self.data == waker.data() && core::ptr::eq(self.vtable, waker.vtable())
    }

    /// `SpmcWaker` state has been updated and the waker claimed, the concrete `Waker`
    /// can be returned.
    #[cfg(not(any(loom, miri)))]
    #[inline(always)]
    pub(crate) fn confirm(self, _state: usize) -> ConfirmedWaker {
        debug_assert!(_state.has(crate::REGISTERED | crate::CACHED));
        // https://github.com/rust-lang/rust/issues/159335
        // SAFETY: the vtable pointer is always nonnull as derived from a static reference
        unsafe { assert_unchecked(!self.vtable.is_null()) };
        // SAFETY: pointers comes from a valid waker and their loading is correctly synchronized
        // (as checked with loom/miri)
        unsafe { Waker::new(self.data, &*self.vtable) }
    }

    #[cfg(any(loom, miri))]
    pub(crate) fn confirm(&self, state: usize) -> ConfirmedWaker {
        use crate::state_machine::*;
        debug_assert!(state.has(REGISTERED | CACHED));
        assert_eq!(self.store_epoch, store_epoch(state));
        ConfirmedWaker {
            // SAFETY: pointers comes from a valid waker and their loading is correctly synchronized
            waker: Some(unsafe { Waker::new(self.data, &*self.vtable) }),
            // SAFETY: the cell array is a valid reference (but uses pointer to avoid lifetime)
            cell: unsafe { &(*self.waker_cells)[self.store_epoch / REGISTRATION_INCR] },
        }
    }
}

#[cfg(any(loom, miri))]
pub(crate) struct ConfirmedWaker {
    waker: Option<Waker>,
    cell: *const crate::Cell<u8>,
}

#[cfg(any(loom, miri))]
impl ConfirmedWaker {
    pub(crate) fn get(mut self) -> Waker {
        self.waker.take().unwrap()
    }

    fn cell(&self) -> &crate::Cell<u8> {
        // SAFETY: the cell is a valid reference (but uses pointer to avoid lifetime)
        unsafe { &*self.cell }
    }

    pub(crate) fn wake(mut self) {
        #[cfg(loom)]
        crate::loom_trace!("Waker::wake");
        self.waker.take().unwrap().wake();
        self.cell().set(0);
        core::mem::forget(self);
    }

    pub(crate) fn wake_by_ref(&self) {
        #[cfg(loom)]
        crate::loom_trace!("Waker::wake_by_ref");
        self.waker.as_ref().unwrap().wake_by_ref();
        self.cell().get();
    }
}

#[cfg(any(loom, miri))]
impl Drop for ConfirmedWaker {
    fn drop(&mut self) {
        #[cfg(loom)]
        crate::loom_trace!("Waker::drop");
        self.cell().set(0);
    }
}
