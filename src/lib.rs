#![cfg_attr(not(loom), no_std)]
use core::{hint::assert_unchecked, mem::MaybeUninit, task::Waker};

use crate::loom::{
    AtomicUsize, AtomicUsizeExt,
    Ordering::{Relaxed, SeqCst},
    UnsafeCell, UnsafeCellExt,
};

#[cfg(debug_assertions)]
mod exclusive;
mod loom;

pub trait IntoWaker {
    fn as_waker(&self) -> &Waker;
    fn into_waker(self) -> Waker;
    fn wake(self);
}

impl IntoWaker for Waker {
    fn as_waker(&self) -> &Waker {
        self
    }
    fn into_waker(self) -> Waker {
        self
    }
    fn wake(self) {
        self.wake();
    }
}

impl IntoWaker for &Waker {
    fn as_waker(&self) -> &Waker {
        self
    }
    fn into_waker(self) -> Waker {
        self.clone()
    }
    fn wake(self) {
        self.wake_by_ref();
    }
}

const EMPTY: usize = 2;
const WAKING: usize = 4;

#[derive(Debug)]
pub struct SpmcWaker {
    wakers: [UnsafeCell<MaybeUninit<Waker>>; 2],
    state: AtomicUsize,
    #[cfg(debug_assertions)]
    exclusive: exclusive::Exclusive,
}

unsafe impl Send for SpmcWaker {}
unsafe impl Sync for SpmcWaker {}

impl Drop for SpmcWaker {
    #[inline]
    fn drop(&mut self) {
        if let Some(waker) = self.wakers.get(self.state.load_mut()) {
            unsafe { waker.with_ref_mut(|w| w.assume_init_drop()) };
        }
    }
}

impl SpmcWaker {
    #[cfg_attr(loom, const_fn::const_fn(cfg(false)))]
    #[inline]
    pub const fn new() -> Self {
        Self {
            wakers: [
                UnsafeCell::new(MaybeUninit::uninit()),
                UnsafeCell::new(MaybeUninit::uninit()),
            ],
            state: AtomicUsize::new(EMPTY),
            #[cfg(debug_assertions)]
            exclusive: exclusive::Exclusive::new(),
        }
    }

    fn load_state(&self) -> usize {
        #[cfg(not(loom))]
        return self.state.load(SeqCst);
        #[cfg(loom)]
        return self.state.fetch_add(0, SeqCst);
    }

    /// # Safety
    ///
    /// `try_register`, `register` and `unregister` methods must not be called concurrently
    /// from multiple threads.
    #[inline]
    pub unsafe fn try_register<W: IntoWaker>(&self, waker: W) -> Result<(), W> {
        #[cfg(debug_assertions)]
        let _guard = self.exclusive.check();
        match self.load_state() {
            EMPTY => {
                unsafe {
                    self.wakers[0].with_ref_mut(|w| {
                        w.write(waker.into_waker());
                    });
                }
                self.state.store(0, SeqCst);
            }
            s if s & WAKING != 0 => return Err(waker),
            idx => self.overwrite(waker, idx),
        }
        Ok(())
    }

    /// # Safety
    ///
    /// `try_register`, `register` and `unregister` methods must not be called concurrently
    /// from multiple threads.
    #[inline]
    pub unsafe fn register<W: IntoWaker>(&self, waker: W) {
        if let Err(waker) = unsafe { self.try_register(waker) } {
            waker.wake();
        }
    }

    #[cold]
    fn overwrite(&self, waker: impl IntoWaker, cur_idx: usize) {
        unsafe { assert_unchecked(cur_idx < 2) };
        if unsafe {
            self.wakers[cur_idx].with_ref(|w| w.assume_init_ref().will_wake(waker.as_waker()))
        } {
            return;
        }
        let new_idx = (cur_idx + 1) % 2;
        unsafe {
            self.wakers[new_idx].with_ref_mut(|w| {
                w.write(waker.into_waker());
            });
        }
        if let Err(state) = (self.state).compare_exchange(cur_idx, new_idx, SeqCst, SeqCst) {
            debug_assert!(state >= 2);
            let waker = unsafe { self.wakers[new_idx].with_ref_mut(|w| w.assume_init_read()) };
            waker.wake();
        } else {
            unsafe { self.wakers[cur_idx].with_ref_mut(|w| w.assume_init_drop()) };
        }
    }

    /// # Safety
    ///
    /// `try_register`, `register` and `unregister` methods must not be called concurrently
    /// from multiple threads.
    #[inline]
    pub unsafe fn unregister(&self) -> Option<Waker> {
        #[cfg(debug_assertions)]
        let _guard = self.exclusive.check();
        let state = self.load_state();
        if let Some(waker_cell) = self.wakers.get(state) {
            match self.state.compare_exchange(state, EMPTY, SeqCst, Relaxed) {
                Ok(_) => return Some(unsafe { waker_cell.with_ref_mut(|w| w.assume_init_read()) }),
                Err(s) => debug_assert!(s >= 2),
            }
        }
        None
    }

    #[inline]
    pub fn take(&self) -> Option<Waker> {
        self.wakers.get(self.load_state())?;
        let waker_cell = self.wakers.get(self.state.fetch_or(WAKING, SeqCst))?;
        let waker = unsafe { waker_cell.with_ref(|w| w.assume_init_read()) };
        self.state.store(EMPTY, SeqCst);
        Some(waker)
    }

    #[inline]
    pub fn wake(&self) {
        if let Some(waker) = self.take() {
            waker.wake();
        }
    }
}

impl Default for SpmcWaker {
    fn default() -> Self {
        Self::new()
    }
}
