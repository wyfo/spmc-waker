use core::{
    mem::ManuallyDrop,
    ptr,
    task::{RawWaker, RawWakerVTable, Waker},
};

use crate::loom::cell::UnsafeCell;

// A const NOOP_VTABLE as `Waker::noop` vtable cannot be accessed in const context.
const NOOP_VTABLE: &RawWakerVTable = &RawWakerVTable::new(
    |_| RawWaker::new(ptr::null(), NOOP_VTABLE),
    |_| (),
    |_| (),
    |_| (),
);
pub const NOOP_PTR: *mut RawWakerVTable = ptr::from_ref(NOOP_VTABLE).cast_mut();

pub(crate) trait UnsafeCellExt<T: Copy> {
    /// # Safety
    ///
    /// The cell must not be concurrently mutated.
    unsafe fn read(&self) -> T;
    /// # Safety
    ///
    /// The cell must not be concurrently accessed.
    unsafe fn write(&self, data: T);
    /// # Safety
    ///
    /// The cell must not be concurrently accessed.
    #[inline(always)]
    unsafe fn replace(&self, data: T) -> T {
        // SAFETY: as per function contract
        let prev = unsafe { self.read() };
        // SAFETY: as per function contract
        unsafe { self.write(data) };
        prev
    }
}

impl<T: Copy> UnsafeCellExt<T> for UnsafeCell<T> {
    #[inline(always)]
    unsafe fn read(&self) -> T {
        // SAFETY: as per function contract
        unsafe {
            #[cfg(not(loom))]
            return *self.get();
            #[cfg(loom)]
            return self.with(|d| *d);
        }
    }
    #[inline(always)]
    unsafe fn write(&self, data: T) {
        // SAFETY: as per function contract
        unsafe {
            #[cfg(not(loom))]
            self.get().write(data);
            #[cfg(loom)]
            self.with_mut(|d| *d = data);
        }
    }
}

pub(crate) trait TaggedPointerExt {
    fn set(self, tag: usize) -> Self;
    fn unset(self, tag: usize) -> Self;
    fn has(self, tag: usize) -> bool;
}

impl<T> TaggedPointerExt for *mut T {
    #[inline(always)]
    fn set(self, tag: usize) -> Self {
        self.map_addr(|addr| addr | tag)
    }
    #[inline(always)]
    fn unset(self, tag: usize) -> Self {
        self.map_addr(|addr| addr & !tag)
    }
    #[inline(always)]
    fn has(self, tag: usize) -> bool {
        self.addr() & tag != 0
    }
}

pub(crate) trait WakerExt {
    fn vtable_ptr(&self) -> *mut RawWakerVTable;
}

impl WakerExt for Waker {
    fn vtable_ptr(&self) -> *mut RawWakerVTable {
        ptr::from_ref(self.vtable()).cast_mut()
    }
}

pub(crate) struct Defer<F: FnOnce() -> T, T = ()>(ManuallyDrop<F>);

impl<F: FnOnce() -> T, T> Defer<F, T> {
    #[inline(always)]
    pub(crate) fn cancel(self) {
        let mut this = ManuallyDrop::new(self);
        // SAFETY: `ManuallyDrop` data is no longer accessed after this call
        unsafe { ManuallyDrop::drop(&mut this.0) }
    }
}

impl<F: FnOnce() -> T, T> Drop for Defer<F, T> {
    #[inline(always)]
    fn drop(&mut self) {
        // SAFETY: `ManuallyDrop` data is no longer accessed after this call
        unsafe { ManuallyDrop::take(&mut self.0)() };
    }
}

#[inline(always)]
pub(crate) fn defer<F: FnOnce() -> T, T>(f: F) -> Defer<F, T> {
    Defer(ManuallyDrop::new(f))
}

#[inline(always)]
pub(crate) fn guard<R, T>(f: impl FnOnce() -> R, reset: impl FnOnce() -> T) -> R {
    let guard = defer(move || {
        reset();
    });
    let res = f();
    guard.cancel();
    res
}
