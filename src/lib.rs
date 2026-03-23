//! A synchronization primitive for task wakeup.
//!
//! This crate provides [`SpmcWaker`], a single-producer, multiple-consumer (SPMC)
//! atomic waker.
//!
//! # Features
//!
//! - `portable-atomic`: use `portable-atomic` crate to provides functionality to
//!   targets without atomics.
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

/// Either a [`Waker`] or a `&Waker`.
pub trait WakerRef {
    /// Returns a reference to the waker.
    fn as_waker(&self) -> &Waker;
    /// Returns an owned waker.
    fn into_waker(self) -> Waker;
    /// Wakes up the task associated with this `Waker`.
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

/// A synchronization primitive for task wakeup.
///
/// Sometimes the task interested in a given event will change over time.
/// A `SpmcWaker` can coordinate concurrent notifications with the consumer
/// potentially "updating" the underlying task to wake up. This is useful in
/// scenarios where a computation completes in another thread and wants to
/// notify the consumer, but the consumer is in the process of being migrated to
/// a new logical task.
///
/// Consumers should call `register` before checking the result of a computation
/// and producers should call `wake` after producing the computation (this
/// differs from the usual `thread::park` pattern). It is also permitted for
/// `wake` to be called **before** `register`. This results in a no-op.
///
/// A single `SpmcWaker` may be reused for any number of calls to `register` or
/// `wake`.
///
/// # Single-producer, multiple-consumer (SPMC)
///
/// `SpmcWaker` algorithm assumes a single thread calling `register`/`unregister`
/// at a time. It is enforced by the methods' safety condition.
///
/// This assumption allows significant optimizations compared to a MPMC algorithm
/// like [`AtomicWaker`].
///
/// # Memory ordering
///
/// `SpmcWaker` has a generic `SYNC` parameter which determines the
/// synchronization guarantees.
///
/// ## `SYNC=true` (the default)
///
/// Calling `register` "acquires" all memory "released" by calls to `wake`
/// before the call to `register`. Later calls to `wake` will wake the
/// registered waker (on contention this wake might be triggered in `register`).
///
/// ## `SYNC=false`
///
/// This is a more advanced configuration, where there is no acquire-release
/// synchronization between `register` and `wake`. A `wake` call may not see
/// the waker registered by a concurrent `register`.
///
/// For this reason, `SpmcWaker<false>` should be paired with a total order,
/// for example atomic `SeqCst` or RMW operations. It ensures that checking
/// the waking condition after `register` succeeds even when a concurrent
/// `wake` miss the registered waker.
///
/// It allows to optimize the algorithm even more, especially in the case
/// where `wake` is called with no waker registered, as it becomes a single
/// atomic load.
///
/// # Examples
///
/// Here is a simple example providing a `Flag` that can be signalled manually
/// when it is ready.
///
/// ```rust
/// use std::{
///     future::poll_fn,
///     sync::{
///         Arc,
///         atomic::{AtomicBool, Ordering::Relaxed},
///     },
///     task::Poll,
///     thread,
/// };
///
/// use spmc_waker::SpmcWaker;
///
/// #[derive(Default)]
/// struct Inner {
///     notified: AtomicBool,
///     waker: SpmcWaker,
/// }
///
/// #[derive(Clone)]
/// struct Notifier(Arc<Inner>);
///
/// impl Notifier {
///     fn notify(&self) {
///         self.0.notified.store(true, Relaxed);
///         self.0.waker.wake();
///     }
/// }
///
/// #[derive(Default)]
/// struct Waiter(Arc<Inner>);
///
/// impl Waiter {
///     async fn wait(&mut self) {
///         poll_fn(move |cx| {
///             // quick check to avoid registration if already done.
///             if self.0.notified.swap(false, Relaxed) {
///                 return Poll::Ready(());
///             }
///             // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
///             unsafe { self.0.waker.register(cx.waker()) };
///             // Need to check condition **after** `register` to avoid a race
///             // condition that would result in lost notifications.
///             if self.0.notified.swap(false, Relaxed) {
///                 // Unregister the waker to avoid spurious wakeups.
///                 // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
///                 unsafe { self.0.waker.unregister() };
///                 Poll::Ready(())
///             } else {
///                 Poll::Pending
///             }
///         })
///         .await;
///     }
///
///     fn notifier(&self) -> Notifier {
///         Notifier(self.0.clone())
///     }
/// }
///
/// fn event() -> (Notifier, Waiter) {
///     let waiter = Waiter::default();
///     (waiter.notifier(), waiter)
/// }
/// ```
///
/// The same example with `SYNC=false` requires a total order on `notified` accesses,
/// for example with `SeqCst` ordering.
///
/// ```rust
/// use std::{
///     future::poll_fn,
///     sync::{
///         Arc,
///         atomic::{
///             AtomicBool,
///             Ordering::{Relaxed, SeqCst},
///         },
///     },
///     task::Poll,
///     thread,
/// };
///
/// use spmc_waker::SpmcWaker;
///
/// #[derive(Default)]
/// struct Inner {
///     notified: AtomicBool,
///     waker: SpmcWaker,
/// }
///
/// #[derive(Clone)]
/// struct Notifier(Arc<Inner>);
///
/// impl Notifier {
///     fn notify(&self) {
///         // Use seqcst ordering to synchronize with the load after `register`
///         self.0.notified.store(true, SeqCst);
///         self.0.waker.wake();
///     }
/// }
///
/// #[derive(Default)]
/// struct Waiter(Arc<Inner>);
///
/// impl Waiter {
///     async fn wait(&mut self) {
///         poll_fn(move |cx| {
///             // quick check to avoid registration if already done.
///             if self.0.notified.swap(false, Relaxed) {
///                 return Poll::Ready(());
///             }
///             // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
///             unsafe { self.0.waker.register(cx.waker()) };
///             // Need to check condition **after** `register` to avoid a race
///             // condition that would result in lost notifications.
///             // Use seqcst ordering so it synchronizes with the store before wake.
///             if self.0.notified.swap(false, SeqCst) {
///                 // Unregister the waker to avoid spurious wakeups.
///                 // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
///                 unsafe { self.0.waker.unregister() };
///                 Poll::Ready(())
///             } else {
///                 Poll::Pending
///             }
///         })
///         .await;
///     }
///
///     fn notifier(&self) -> Notifier {
///         Notifier(self.0.clone())
///     }
/// }
///
/// fn event() -> (Notifier, Waiter) {
///     let waiter = Waiter::default();
///     (waiter.notifier(), waiter)
/// }
/// ```
///
/// [`AtomicWaker`]: https://docs.rs/futures/latest/futures/task/struct.AtomicWaker.html
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
    /// Creates a new `SpmcWaker`.
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

    fn load_state(&self) -> usize {
        #[cfg(not(loom))]
        return self.state.load(SeqCst);
        #[cfg(loom)]
        return self.state.fetch_add(0, SeqCst);
    }

    /// Registers the waker to be notified on calls to `wake`.
    ///
    /// The new task will take place of any previous tasks that were registered
    /// by previous calls to `register`. Any calls to `wake` that happen after
    /// a call to `register` (as defined by the memory ordering rules), will
    /// notify the `register` caller's task and deregister the waker from future
    /// notifications. Because of this, callers should ensure `register` gets
    /// invoked with a new `Waker` **each** time they require a wakeup.
    ///
    /// It is safe to call `register` with multiple other threads concurrently
    /// calling `wake`. If `SYNC=true`, this will result in the `register`
    /// caller's current task being notified once.
    ///
    /// # Safety
    ///
    /// `register` and `unregister` methods must not be called concurrently
    /// from multiple threads.
    #[inline]
    pub unsafe fn register<W: WakerRef>(&self, waker: W) {
        #[cfg(all(debug_assertions, not(loom)))]
        let _guard = self.exclusive.check();
        let state = self.load_state();
        if state == EMPTY {
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
        } else {
            self.overwrite(waker, state);
        }
    }

    #[cold]
    fn overwrite(&self, waker: impl WakerRef, state: usize) {
        if (SYNC && state & WAKING != 0) || (!SYNC && state == WAKING) {
            waker.wake();
            #[cfg(loom)]
            ::loom::hint::spin_loop();
            return;
        }
        let cur_idx = state;
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

    /// Removes the registered waker, returning it without waking it.
    ///
    /// It allows avoiding spurious wakeups when a waker has been registered,
    /// but the wake condition is already met.
    ///
    /// # Safety
    ///
    /// `register` and `unregister` methods must not be called concurrently
    /// from multiple threads.
    #[inline]
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

    /// Returns the last `Waker` passed to `register`, so that the caller can wake it.
    ///
    /// Sometimes, just waking the `SpmcWaker` is not fine grained enough. This allows the caller
    /// to take the waker and then wake it separately, rather than performing both steps in one
    /// atomic action.
    ///
    /// If a waker has not been registered, this returns `None`.
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

    /// Calls `wake` on the last `Waker` passed to `register`.
    ///
    /// If `register` has not been called yet, then this does nothing.
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
