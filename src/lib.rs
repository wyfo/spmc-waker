#![cfg_attr(not(loom), no_std)]
use core::{hint::assert_unchecked, mem::MaybeUninit, task::Waker};

use crate::loom::{
    AtomicUsize, AtomicUsizeExt,
    Ordering::{Relaxed, SeqCst},
    UnsafeCell, UnsafeCellExt,
};

#[cfg(all(debug_assertions, not(loom)))]
mod exclusive;
mod loom;

pub trait WakerRef {
    fn as_waker(&self) -> &Waker;
    fn into_waker(self) -> Waker;
    fn wake(self);
}

impl WakerRef for Waker {
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

impl WakerRef for &Waker {
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
pub struct SpmcWaker<const SYNC: bool = true> {
    wakers: [UnsafeCell<MaybeUninit<Waker>>; 2],
    state: AtomicUsize,
    #[cfg(all(debug_assertions, not(loom)))]
    exclusive: exclusive::Exclusive,
}

unsafe impl<const SYNC: bool> Send for SpmcWaker<SYNC> {}
unsafe impl<const SYNC: bool> Sync for SpmcWaker<SYNC> {}

impl<const SYNC: bool> Drop for SpmcWaker<SYNC> {
    #[inline]
    fn drop(&mut self) {
        if let Some(waker) = self.wakers.get(self.state.load_mut()) {
            unsafe { waker.with_ref_mut(|w| w.assume_init_drop()) };
        }
    }
}

impl<const SYNC: bool> SpmcWaker<SYNC> {
    #[cfg_attr(loom, const_fn::const_fn(cfg(false)))]
    #[inline]
    pub const fn new() -> Self {
        Self {
            wakers: [
                UnsafeCell::new(MaybeUninit::uninit()),
                UnsafeCell::new(MaybeUninit::uninit()),
            ],
            state: AtomicUsize::new(EMPTY),
            #[cfg(all(debug_assertions, not(loom)))]
            exclusive: exclusive::Exclusive::new(),
        }
    }

    /// # Safety
    ///
    /// `try_register`, `register` and `unregister` methods must not be called concurrently
    /// from multiple threads.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub unsafe fn register<W: WakerRef>(&self, waker: W) {
        if let Err(waker) = unsafe { self.try_register(waker) } {
            waker.wake();
            #[cfg(loom)]
            ::loom::hint::spin_loop();
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
    #[cfg_attr(debug_assertions, track_caller)]
    pub unsafe fn try_register<W: WakerRef>(&self, waker: W) -> Result<(), W> {
        #[cfg(all(debug_assertions, not(loom)))]
        let _guard = self.exclusive.check();
        match self.load_state() {
            EMPTY => {
                unsafe {
                    self.wakers[0].with_ref_mut(|w| {
                        w.write(waker.into_waker());
                    });
                }
                if SYNC || cfg!(loom) {
                    self.state.swap(0, SeqCst);
                } else {
                    self.state.store(0, SeqCst);
                }
            }
            s if (SYNC && s & WAKING != 0) || (!SYNC && s == WAKING) => return Err(waker),
            idx => self.overwrite(waker, idx),
        }
        Ok(())
    }

    #[cold]
    fn overwrite(&self, waker: impl WakerRef, cur_idx: usize) {
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
    #[cfg_attr(debug_assertions, track_caller)]
    pub unsafe fn unregister(&self) -> Option<Waker> {
        #[cfg(all(debug_assertions, not(loom)))]
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
        if SYNC {
            if self.state.load(Relaxed) >= 2 && self.state.fetch_add(0, SeqCst) >= 2 {
                return None;
            }
            let state = self.state.fetch_or(WAKING, SeqCst);
            if state & WAKING != 0 {
                return None;
            }
            if let Some(waker_cell) = self.wakers.get(state) {
                let waker = unsafe { waker_cell.with_ref(|w| w.assume_init_read()) };
                self.state.swap(EMPTY, SeqCst);
                Some(waker)
            } else {
                debug_assert_eq!(state, EMPTY);
                let _ = (self.state).compare_exchange(WAKING | EMPTY, EMPTY, SeqCst, Relaxed);
                None
            }
        } else {
            let state = self.load_state();
            let waker_cell = self.wakers.get(state)?;
            (self.state.compare_exchange(state, WAKING, SeqCst, Relaxed)).ok()?;
            let waker = unsafe { waker_cell.with_ref(|w| w.assume_init_read()) };
            if cfg!(loom) {
                self.state.swap(EMPTY, SeqCst);
            } else {
                self.state.store(EMPTY, SeqCst);
            }
            Some(waker)
        }
    }

    #[inline]
    pub fn wake(&self) {
        if let Some(waker) = self.take() {
            waker.wake();
        }
    }
}

impl<const SYNC: bool> Default for SpmcWaker<SYNC> {
    fn default() -> Self {
        Self::new()
    }
}
