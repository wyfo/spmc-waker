#[cfg(not(loom))]
pub(crate) use core::*;

#[cfg(loom)]
pub(crate) use loom::*;

pub(crate) mod sync {
    pub(crate) mod atomic {
        #[cfg(all(not(loom), not(feature = "portable-atomic")))]
        pub(crate) use core::sync::atomic::*;

        #[cfg(loom)]
        pub(crate) use loom::sync::atomic::*;
        #[cfg(all(not(loom), feature = "portable-atomic"))]
        pub(crate) use portable_atomic::*;

        #[cfg(loom)]
        fn seqcst_fence(order: Ordering) {
            if order == loom::sync::atomic::Ordering::SeqCst {
                loom::sync::atomic::fence(order);
            }
        }

        #[cfg(loom)]
        #[derive(Debug)]
        pub(crate) struct AtomicUsize(loom::sync::atomic::AtomicUsize);

        #[cfg(loom)]
        impl AtomicUsize {
            pub(crate) fn new(x: usize) -> Self {
                Self(loom::sync::atomic::AtomicUsize::new(x))
            }

            pub(crate) fn with_mut<R>(&mut self, f: impl FnOnce(&mut usize) -> R) -> R {
                self.0.with_mut(|x| f(x))
            }

            pub(crate) fn load(&self, order: Ordering) -> usize {
                seqcst_fence(order);
                self.0.load(order)
            }

            pub(crate) fn store(&self, x: usize, order: Ordering) {
                self.0.store(x, order);
                seqcst_fence(order);
            }

            pub(crate) fn swap(&self, x: usize, order: Ordering) -> usize {
                seqcst_fence(order);
                let res = self.0.swap(x, order);
                seqcst_fence(order);
                res
            }

            pub(crate) fn compare_exchange(
                &self,
                current: usize,
                new: usize,
                success: Ordering,
                failure: Ordering,
            ) -> Result<usize, usize> {
                seqcst_fence(success);
                let res = self.0.compare_exchange(current, new, success, failure);
                if res.is_ok() {
                    seqcst_fence(success);
                }
                res
            }

            pub(crate) fn fetch_add(&self, val: usize, order: Ordering) -> usize {
                seqcst_fence(order);
                let res = self.0.fetch_add(val, order);
                seqcst_fence(order);
                res
            }

            pub(crate) fn fetch_or(&self, val: usize, order: Ordering) -> usize {
                seqcst_fence(order);
                let res = self.0.fetch_or(val, order);
                seqcst_fence(order);
                res
            }
        }
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
