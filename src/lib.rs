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
pub mod small;

pub trait IntoWaker {
    fn as_waker(&self) -> &Waker;
    fn into_waker(self) -> Waker;
}

impl IntoWaker for Waker {
    fn as_waker(&self) -> &Waker {
        self
    }
    fn into_waker(self) -> Waker {
        self
    }
}

impl IntoWaker for &Waker {
    fn as_waker(&self) -> &Waker {
        self
    }
    fn into_waker(self) -> Waker {
        self.clone()
    }
}

const EMPTY: usize = 2;
const WAKING: usize = 4;
const EXCLUSIVE: usize = 8;

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
    fn drop(&mut self) {
        if let Some(waker) = self.wakers.get(self.state.load_mut()) {
            unsafe { waker.with_ref_mut(|w| w.assume_init_drop()) };
        }
    }
}

impl SpmcWaker {
    #[cfg_attr(loom, const_fn::const_fn(cfg(false)))]
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

    unsafe fn register_impl<const SAFE: bool>(&self, waker: impl IntoWaker) -> bool {
        #[cfg(debug_assertions)]
        let _guard = (!SAFE).then(|| self.exclusive.check());
        let mut state = self.load_state();
        if SAFE {
            if state & (WAKING | EXCLUSIVE) != 0 {
                return false;
            }
            state = self.state.fetch_or(EXCLUSIVE, SeqCst);
            if state & EXCLUSIVE != 0 {
                return false;
            } else if state & WAKING != 0 {
                self.state.fetch_and(!EXCLUSIVE, SeqCst);
                return false;
            }
        }
        if state == EMPTY {
            unsafe {
                self.wakers[0].with_ref_mut(|w| {
                    w.write(waker.into_waker());
                })
            };
            self.state.store(0, SeqCst);
            true
        } else if !SAFE && state == WAKING {
            false
        } else {
            self.overwrite::<SAFE>(waker, state)
        }
    }

    /// # Safety
    ///
    /// `register` and `unregister` methods must not be called concurrently from multiple threads.
    pub unsafe fn register<W: IntoWaker>(&self, waker: W) -> bool {
        unsafe { self.register_impl::<false>(waker) }
    }

    #[cold]
    #[inline(never)]
    fn overwrite<const SAFE: bool>(&self, waker: impl IntoWaker, cur_idx: usize) -> bool {
        unsafe { assert_unchecked(cur_idx < 2) };
        if unsafe {
            self.wakers[cur_idx].with_ref(|w| w.assume_init_ref().will_wake(waker.as_waker()))
        } {
            return true;
        }
        let new_idx = (cur_idx + 1) % 2;
        unsafe {
            self.wakers[new_idx].with_ref_mut(|w| {
                w.write(waker.into_waker());
            })
        };
        let exc_flag = EXCLUSIVE * SAFE as usize;
        if let Err(state) =
            (self.state).compare_exchange(cur_idx | exc_flag, new_idx, SeqCst, SeqCst)
        {
            debug_assert!(state >= 2 && state & exc_flag == exc_flag);
            unsafe { self.wakers[new_idx].with_ref_mut(|w| w.assume_init_drop()) };
            if SAFE {
                self.state.fetch_and(!EXCLUSIVE, SeqCst);
            }
            false
        } else {
            unsafe { self.wakers[cur_idx].with_ref_mut(|w| w.assume_init_drop()) };
            true
        }
    }

    unsafe fn unregister_impl<const SAFE: bool>(&self) -> bool {
        #[cfg(debug_assertions)]
        let _guard = (!SAFE).then(|| self.exclusive.check());
        let state = self.load_state();
        if let Some(waker_cell) = self.wakers.get(state) {
            match self.state.compare_exchange(state, EMPTY, SeqCst, Relaxed) {
                Ok(_) => {
                    unsafe { waker_cell.with_ref_mut(|w| w.assume_init_drop()) };
                    return true;
                }
                Err(s) => debug_assert!(s >= 2),
            }
        }
        false
    }

    /// # Safety
    ///
    /// `register` and `unregister` methods must not be called concurrently from multiple threads.
    pub unsafe fn unregister(&self) -> bool {
        unsafe { self.unregister_impl::<true>() }
    }

    fn take_impl<const SAFE: bool>(&self) -> Option<Waker> {
        let state = self.load_state();
        let exc_flag = EXCLUSIVE * SAFE as usize;
        let waker_cell = self.wakers.get(state & !exc_flag)?;
        if SAFE {
            let state = self.state.fetch_or(WAKING, SeqCst);
            if state & WAKING != 0 {
                return None;
            }
            let waker = self
                .wakers
                .get(state & !exc_flag)
                .map(|w| unsafe { w.with_ref(|w| w.assume_init_read()) });
            if state & EXCLUSIVE == 0
                || (self.state)
                    .compare_exchange(state | WAKING, EMPTY | EXCLUSIVE, SeqCst, Relaxed)
                    .is_err()
            {
                debug_assert!(self.load_state() & EXCLUSIVE == 0);
                self.state.store(EMPTY, SeqCst);
            }
            waker
        } else {
            (self.state.compare_exchange(state, WAKING, SeqCst, Relaxed)).ok()?;
            let waker = unsafe { waker_cell.with_ref(|w| w.assume_init_read()) };
            self.state.store(EMPTY, SeqCst);
            Some(waker)
        }
    }

    pub fn take(&self) -> Option<Waker> {
        self.take_impl::<false>()
    }

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

#[derive(Debug, Default)]
pub struct SafeSpmcWaker(SpmcWaker);

impl SafeSpmcWaker {
    #[cfg_attr(loom, const_fn::const_fn(cfg(false)))]
    pub const fn new() -> Self {
        Self(SpmcWaker::new())
    }

    pub fn register<W: IntoWaker>(&self, waker: W) -> bool {
        unsafe { self.0.register_impl::<true>(waker) }
    }

    pub fn unregister(&self) -> bool {
        unsafe { self.0.unregister_impl::<true>() }
    }

    pub fn take(&self) -> Option<Waker> {
        self.0.take_impl::<true>()
    }

    pub fn wake(&self) {
        if let Some(waker) = self.take() {
            waker.wake();
        }
    }
}
