#[cfg(not(loom))]
pub(crate) use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(loom)]
pub(crate) use loom::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};

pub(crate) trait UnsafeCellExt<T> {
    unsafe fn with_ref<R, F: FnOnce(&T) -> R>(&self, f: F) -> R;
    unsafe fn with_ref_mut<R, F: FnOnce(&mut T) -> R>(&self, f: F) -> R;
}

#[cfg(not(loom))]
impl<T> UnsafeCellExt<T> for UnsafeCell<T> {
    unsafe fn with_ref<R, F: FnOnce(&T) -> R>(&self, f: F) -> R {
        f(unsafe { &*self.get() })
    }
    unsafe fn with_ref_mut<R, F: FnOnce(&mut T) -> R>(&self, f: F) -> R {
        f(unsafe { &mut *self.get() })
    }
}

#[cfg(loom)]
impl<T> UnsafeCellExt<T> for UnsafeCell<T> {
    unsafe fn with_ref<R, F: FnOnce(&T) -> R>(&self, f: F) -> R {
        self.with(|ptr| f(unsafe { &*ptr }))
    }
    unsafe fn with_ref_mut<R, F: FnOnce(&mut T) -> R>(&self, f: F) -> R {
        self.with_mut(|ptr| f(unsafe { &mut *ptr }))
    }
}

pub(crate) trait AtomicUsizeExt {
    fn load_mut(&mut self) -> usize;
}

#[cfg(not(loom))]
impl AtomicUsizeExt for AtomicUsize {
    fn load_mut(&mut self) -> usize {
        *self.get_mut()
    }
}

#[cfg(loom)]
impl AtomicUsizeExt for loom::sync::atomic::AtomicUsize {
    fn load_mut(&mut self) -> usize {
        self.with_mut(|v| *v)
    }
}
