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
        pub(crate) struct AtomicPtr<T> {
            atomic: loom::sync::atomic::AtomicUsize,
            _phantom: core::marker::PhantomData<*mut T>,
        }

        #[cfg(loom)]
        impl<T> AtomicPtr<T> {
            pub(crate) fn new(x: *mut T) -> Self {
                Self {
                    atomic: loom::sync::atomic::AtomicUsize::new(x.expose_provenance()),
                    _phantom: core::marker::PhantomData,
                }
            }

            pub(crate) fn load(&self, order: Ordering) -> *mut T {
                seqcst_fence(order);
                core::ptr::with_exposed_provenance_mut(self.atomic.load(order))
            }

            pub(in crate::loom) fn load_mut(&mut self) -> *mut T {
                core::ptr::with_exposed_provenance_mut(self.atomic.with_mut(|x| *x))
            }

            pub(crate) fn store(&self, x: *mut T, order: Ordering) {
                self.atomic.store(x.expose_provenance(), order);
                seqcst_fence(order);
            }

            pub(crate) fn swap(&self, x: *mut T, order: Ordering) -> *mut T {
                seqcst_fence(order);
                let res = self.atomic.swap(x.expose_provenance(), order);
                seqcst_fence(order);
                core::ptr::with_exposed_provenance_mut(res)
            }

            pub(crate) fn compare_exchange(
                &self,
                current: *mut T,
                new: *mut T,
                success: Ordering,
                failure: Ordering,
            ) -> Result<*mut T, *mut T> {
                seqcst_fence(success);
                let res = self.atomic.compare_exchange(
                    current.expose_provenance(),
                    new.expose_provenance(),
                    success,
                    failure,
                );
                if res.is_ok() {
                    seqcst_fence(success);
                }
                res.map(core::ptr::with_exposed_provenance_mut)
                    .map_err(core::ptr::with_exposed_provenance_mut)
            }

            pub(crate) fn fetch_byte_add(&self, val: usize, order: Ordering) -> *mut T {
                seqcst_fence(order);
                let res = self.atomic.fetch_add(val, order);
                seqcst_fence(order);
                core::ptr::with_exposed_provenance_mut(res)
            }

            pub(crate) fn fetch_or(&self, val: usize, order: Ordering) -> *mut T {
                seqcst_fence(order);
                let res = self.atomic.fetch_or(val, order);
                seqcst_fence(order);
                core::ptr::with_exposed_provenance_mut(res)
            }
        }
    }
}

pub(crate) trait AtomicPtrExt<T> {
    fn load_mut(&mut self) -> *mut T;
}

impl<T> AtomicPtrExt<T> for sync::atomic::AtomicPtr<T> {
    #[cfg_attr(loom, track_caller)]
    fn load_mut(&mut self) -> *mut T {
        #[cfg(not(loom))]
        return *self.get_mut();
        #[cfg(loom)]
        return self.load_mut();
    }
}
