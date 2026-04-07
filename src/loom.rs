#[cfg(not(loom))]
pub(crate) use core::*;

#[cfg(loom)]
pub(crate) use loom::*;
#[cfg(all(not(loom), feature = "portable-atomic"))]
pub(crate) mod sync {
    pub(crate) mod atomic {
        pub(crate) use portable_atomic::*;
    }
}

pub(crate) trait UnsafeCellExt<T> {
    /// # Safety
    ///
    /// Cell content must be safe to deref immutably.
    unsafe fn with_ref<R, F: FnOnce(&T) -> R>(&self, f: F) -> R;
    /// # Safety
    ///
    /// Cell content must be safe to deref mutably.
    unsafe fn with_ref_mut<R, F: FnOnce(&mut T) -> R>(&self, f: F) -> R;
}

impl<T> UnsafeCellExt<T> for cell::UnsafeCell<T> {
    #[cfg_attr(loom, track_caller)]
    unsafe fn with_ref<R, F: FnOnce(&T) -> R>(&self, f: F) -> R {
        #[cfg(not(loom))]
        return f(unsafe { &*self.get() });
        #[cfg(loom)]
        return self.with(|ptr| f(unsafe { &*ptr }));
    }
    #[cfg_attr(loom, track_caller)]
    unsafe fn with_ref_mut<R, F: FnOnce(&mut T) -> R>(&self, f: F) -> R {
        #[cfg(not(loom))]
        return f(unsafe { &mut *self.get() });
        #[cfg(loom)]
        return self.with_mut(|ptr| f(unsafe { &mut *ptr }));
    }
}

pub(crate) trait AtomicUsizeExt {
    fn load_mut(&mut self) -> usize;
}

impl AtomicUsizeExt for sync::atomic::AtomicUsize {
    #[cfg_attr(loom, track_caller)]
    fn load_mut(&mut self) -> usize {
        #[cfg(not(loom))]
        return *self.get_mut();
        #[cfg(loom)]
        return self.with_mut(|v| *v);
    }
}
