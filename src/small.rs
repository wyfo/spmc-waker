use core::{mem::MaybeUninit, task::Waker};

use crate::{
    loom::{
        AtomicUsize, AtomicUsizeExt,
        Ordering::{Relaxed, SeqCst},
        UnsafeCell, UnsafeCellExt,
    },
    IntoWaker,
};

const EMPTY: usize = 0;
const REGISTERED: usize = 1;
const WAKING: usize = 2;
const OVERWRITE: usize = 4;

#[derive(Debug)]
pub struct SmallSpmcWaker {
    state: AtomicUsize,
    waker: UnsafeCell<MaybeUninit<Waker>>,
    #[cfg(debug_assertions)]
    exclusive: crate::exclusive::Exclusive,
}

unsafe impl Send for SmallSpmcWaker {}
unsafe impl Sync for SmallSpmcWaker {}

impl Drop for SmallSpmcWaker {
    #[inline]
    fn drop(&mut self) {
        if self.state.load_mut() == REGISTERED {
            unsafe { self.waker.with_ref_mut(|w| w.assume_init_drop()) };
        }
    }
}

impl SmallSpmcWaker {
    #[cfg_attr(loom, const_fn::const_fn(cfg(false)))]
    pub const fn new() -> Self {
        Self {
            state: AtomicUsize::new(EMPTY),
            waker: UnsafeCell::new(MaybeUninit::uninit()),
            #[cfg(debug_assertions)]
            exclusive: crate::exclusive::Exclusive::new(),
        }
    }

    fn load_state(&self) -> usize {
        #[cfg(not(loom))]
        return self.state.load(SeqCst);
        #[cfg(loom)]
        return self.state.fetch_add(0, SeqCst);
    }

    #[inline]
    pub unsafe fn register<W: IntoWaker>(&self, waker: W) -> bool {
        #[cfg(debug_assertions)]
        let _guard = self.exclusive.check();
        let state = self.load_state();
        if state == EMPTY {
            unsafe {
                self.waker.with_ref_mut(|w| {
                    w.write(waker.into_waker());
                })
            };
            self.state.store(REGISTERED, SeqCst);
            true
        } else if state == REGISTERED {
            self.overwrite(waker)
        } else {
            debug_assert!(state & WAKING != 0 && state & OVERWRITE == 0);
            false
        }
    }

    #[cold]
    fn overwrite(&self, waker: impl IntoWaker) -> bool {
        if unsafe {
            self.waker
                .with_ref(|w| w.assume_init_ref().will_wake(waker.as_waker()))
        } {
            return true;
        }
        if let Err(state) = self
            .state
            .compare_exchange(REGISTERED, OVERWRITE, SeqCst, SeqCst)
        {
            debug_assert_eq!(state, WAKING);
            return false;
        }
        unsafe { self.waker.with_ref_mut(|w| w.assume_init_drop()) };
        unsafe {
            self.waker.with_ref_mut(|w| {
                w.write(waker.into_waker());
            })
        };
        if let Err(state) = self
            .state
            .compare_exchange(OVERWRITE, REGISTERED, SeqCst, SeqCst)
        {
            debug_assert_eq!(state, OVERWRITE | WAKING);
            unsafe { self.waker.with_ref_mut(|w| w.assume_init_drop()) };
            self.state.store(EMPTY, SeqCst);
            return false;
        }
        true
    }

    #[inline]
    pub unsafe fn unregister(&self) -> bool {
        #[cfg(debug_assertions)]
        let _guard = self.exclusive.check();
        if self.load_state() == REGISTERED {
            match (self.state).compare_exchange(REGISTERED, EMPTY, SeqCst, Relaxed) {
                Ok(_) => {
                    unsafe { self.waker.with_ref_mut(|w| w.assume_init_drop()) };
                    return true;
                }
                Err(s) => debug_assert!(s & WAKING != 0 || s == EMPTY),
            }
        }
        false
    }

    #[inline]
    pub fn take(&self) -> Option<Waker> {
        let state = self.load_state();
        if state == OVERWRITE {
            return self.take_overwritten();
        } else if state != REGISTERED {
            return None;
        }
        ((self.state).compare_exchange(REGISTERED, WAKING, SeqCst, Relaxed)).ok()?;
        let waker = unsafe { self.waker.with_ref(|w| w.assume_init_read()) };
        self.state.store(EMPTY, SeqCst);
        Some(waker)
    }

    #[cold]
    fn take_overwritten(&self) -> Option<Waker> {
        let state = self.state.fetch_or(WAKING, SeqCst);
        if state == REGISTERED {
            let waker = unsafe { self.waker.with_ref(|w| w.assume_init_read()) };
            self.state.store(EMPTY, SeqCst);
            return Some(waker);
        } else if state == EMPTY {
            self.state.store(EMPTY, SeqCst);
        }
        None
    }

    #[inline]
    pub fn wake(&self) {
        if let Some(waker) = self.take() {
            waker.wake();
        }
    }
}

impl Default for SmallSpmcWaker {
    fn default() -> Self {
        Self::new()
    }
}
