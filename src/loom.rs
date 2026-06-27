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
        use crate::loom::loom_trace;

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

            #[track_caller]
            pub(crate) fn load(&self, order: Ordering) -> *mut T {
                seqcst_fence(order);
                let res: *mut T = core::ptr::with_exposed_provenance_mut(self.atomic.load(order));
                loom_trace!("load({order:?}) -> {res:p}");
                res
            }

            #[track_caller]
            pub(in crate::loom) fn load_mut(&mut self) -> *mut T {
                let res: *mut T =
                    core::ptr::with_exposed_provenance_mut(self.atomic.with_mut(|x| *x));
                loom_trace!("load_mut() -> {res:p}");
                res
            }

            #[track_caller]
            pub(crate) fn store(&self, x: *mut T, order: Ordering) {
                self.atomic.store(x.expose_provenance(), order);
                seqcst_fence(order);
                loom_trace!("store({x:p}, {order:?})");
            }

            #[track_caller]
            pub(crate) fn swap(&self, x: *mut T, order: Ordering) -> *mut T {
                seqcst_fence(order);
                let res = self.atomic.swap(x.expose_provenance(), order);
                seqcst_fence(order);
                let res: *mut T = core::ptr::with_exposed_provenance_mut(res);
                loom_trace!("swap({x:p}, {order:?}) -> {res:p}");
                res
            }

            #[track_caller]
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
                let res = res
                    .map(core::ptr::with_exposed_provenance_mut::<T>)
                    .map_err(core::ptr::with_exposed_provenance_mut::<T>);
                loom_trace!(
                    "compare_exchange(cur={current:p}, new={new:p}, {success:?}/{failure:?}) -> {res:?}"
                );
                res
            }

            #[track_caller]
            pub(crate) fn fetch_byte_add(&self, val: usize, order: Ordering) -> *mut T {
                seqcst_fence(order);
                let res = self.atomic.fetch_add(val, order);
                seqcst_fence(order);
                let res: *mut T = core::ptr::with_exposed_provenance_mut(res);
                loom_trace!("fetch_byte_add({val:#x}, {order:?}) -> {res:p}");
                res
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

#[cfg(loom)]
pub(crate) mod trace {
    extern crate std;
    use std::{
        fs::{File, OpenOptions},
        sync::LazyLock,
    };

    const PATH: &str = "loom.trace";

    /// Tracing is enabled at runtime by setting the `LOOM_TRACE` env var.
    pub(crate) static ENABLED: LazyLock<bool> =
        LazyLock::new(|| std::env::var_os("LOOM_TRACE").is_some());

    pub(crate) static FILE: LazyLock<File> = LazyLock::new(|| {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(PATH)
            .unwrap()
    });

    /// Clears loom trace file.
    pub fn clear() {
        if *ENABLED {
            FILE.set_len(0).unwrap();
        }
    }
}

#[cfg(loom)]
macro_rules! loom_trace {
    ($($t:tt)*) => {
        if *crate::loom::trace::ENABLED {
            extern crate std;
            use std::io::Write;
            // Format the whole line first and append it with a single
            // `write_all`: `O_APPEND` makes one small write atomic, so
            // concurrent threads' lines don't tear (no lock needed).
            let line = std::format!(
                "[{:?}] {}: {}\n",
                loom::thread::current().id(),
                core::panic::Location::caller(),
                format_args!($($t)*),
            );
            let _ = (&*crate::loom::trace::FILE).write_all(line.as_bytes());
        }
    };
}

#[cfg(loom)]
pub(crate) use loom_trace;
