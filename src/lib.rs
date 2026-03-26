//! A synchronization primitive for task wakeup.
//!
//! This crate provides [`SpmcWaker`], a single-producer, multiple-consumer (SPMC)
//! atomic waker.
//!
//! # Features
//!
//! - `portable-atomic`: use `portable-atomic` crate to provide functionality to
//!   targets without atomics.
#![cfg_attr(not(loom), no_std)]
use core::{hint::assert_unchecked, mem::ManuallyDrop, task::Waker};

use crate::{
    loom::{
        AtomicUsize, AtomicUsizeExt,
        Ordering::{Relaxed, SeqCst},
    },
    waker_cell::WakerCell,
};

#[cfg(all(debug_assertions, not(loom)))]
mod exclusive;
mod loom;
mod waker_cell;

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
/// This assumption allows significant optimizations compared to an MPMC algorithm
/// like [`AtomicWaker`].
///
/// # Memory ordering
///
/// `SpmcWaker` has a generic `SYNC` parameter which determines the
/// synchronization guarantees.
///
/// ### `SYNC=true` (the default)
///
/// Calling `register` "acquires" all memory "released" by calls to `wake`
/// before the call to `register`. Later calls to `wake` will wake the
/// registered waker (on contention this wake might be triggered in `register`).
///
/// ### `SYNC=false`
///
/// This is a more advanced configuration, where there is no acquire-release
/// synchronization between `register` and `wake`. A `wake` call may not see
/// the waker registered by a concurrent `register`.
///
/// For this reason, `SpmcWaker<false>` should be paired with a total order,
/// for example atomic `SeqCst` or RMW operations. It ensures that checking
/// the waking condition after `register` succeeds even when a concurrent
/// `wake` misses the registered waker.
///
/// It allows optimizing the algorithm even more, especially in the case
/// where `wake` is called with no waker registered, as it becomes a single
/// atomic load.
///
/// # Waker caching
///
/// Most of the time, `SpmcWaker` is used in a single task, so the waker
/// registered is always the same. That's why it provides a second generic
/// parameter `CACHED`.
///
/// ### `CACHED=true` (the default)
///
/// The last waker registered is kept cached to avoid cloning it at the next
/// registration. As a consequence, waking is done with [`Waker::wake_by_ref`].
/// As wakers are often `Arc`s, caching avoids atomic RMW operations updating
/// the reference counter.
///
/// ### `CACHED=false`
///
/// Waker is cloned when registered by reference, and the tasks are woken with
/// [`Waker::wake`].
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
/// };
///
/// use spmc_waker::SpmcWaker;
///
/// #[derive(Default)]
/// struct Inner {
///     notified: AtomicBool,
///     waker: SpmcWaker<false>,
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
pub struct SpmcWaker<const SYNC: bool = true, const CACHED: bool = true> {
    wakers: [WakerCell; 2],
    /// State possible values are:
    /// - 0 or 1: A waker is registered in `wakers[state]`
    /// - EMPTY: there is no waker registered
    ///   with CACHED=true, it becomes a bit-flag and the state's LSB gives
    ///   the cached waker index (cells are initialized with dummy wakers)
    /// - WAKING: a `wake` operation is ongoing;
    ///   with SYNC=true, it becomes a bit-flag
    state: AtomicUsize,
    #[cfg(all(debug_assertions, not(loom)))]
    exclusive: exclusive::Exclusive,
}

unsafe impl<const SYNC: bool, const CACHED: bool> Send for SpmcWaker<SYNC, CACHED> {}
unsafe impl<const SYNC: bool, const CACHED: bool> Sync for SpmcWaker<SYNC, CACHED> {}

impl<const SYNC: bool, const CACHED: bool> Drop for SpmcWaker<SYNC, CACHED> {
    #[inline]
    fn drop(&mut self) {
        let state = self.state.load_mut();
        if CACHED || state < 2 {
            // SAFETY: state is the index of a waker currently registered
            // that must be taken back, and access is safe in destructor
            unsafe { self.wakers[state % 2].drop() };
        }
    }
}

impl<const SYNC: bool, const CACHED: bool> SpmcWaker<SYNC, CACHED> {
    /// Creates a new `SpmcWaker`.
    #[cfg_attr(loom, const_fn::const_fn(cfg(false)))]
    #[inline]
    pub const fn new() -> Self {
        Self {
            wakers: [WakerCell::new(), WakerCell::new()],
            state: AtomicUsize::new(EMPTY),
            #[cfg(all(debug_assertions, not(loom)))]
            exclusive: exclusive::Exclusive::new(),
        }
    }

    fn load_state(&self) -> usize {
        #[cfg(not(loom))]
        return self.state.load(SeqCst);
        // loom doesn't support SeqCst and uses RMW operation instead
        // to emulate the total order.
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
        // State is loaded and expected to be EMPTY. Otherwise, it means
        // there already is a registered waker that needs to be overwritten.
        let state = self.load_state();
        // The case `CACHED && state == EMPTY | 1` is handled in `overwrite`.
        if state == EMPTY {
            // SAFETY: SeqCst protect against outdated read, and `register`
            // cannot be called concurrently. It means that reading EMPTY
            // ensures there cannot be any registered waker at this point.
            // A concurrent `wake` will thus not attempt any read, so it's
            // safe to access both cells mutably.
            unsafe {
                if !CACHED {
                    self.wakers[0].set(waker);
                } else if !self.wakers[0].will_wake(&waker) {
                    return self.overwrite(waker, state);
                }
            }
            // SYNC=true uses swap, as `wake` must synchronize with `register`
            // (loom doesn't support SeqCst and uses RMW operation instead)
            if SYNC || cfg!(loom) {
                self.state.swap(0, SeqCst);
            } else {
                self.state.store(0, SeqCst);
            }
        } else {
            self.overwrite(waker, state);
        }
    }

    // Overwriting a registered waker is expected to be rare, hence the `#[cold]` attribute.
    #[cold]
    fn overwrite(&self, waker: impl WakerRef, state: usize) {
        // A concurrent `wake` may be happening.
        if (SYNC && state & WAKING != 0) || (!SYNC && state == WAKING) {
            // A thread is currently waking the registered waker, so we can
            // assume we should not wait and return immediately.
            // In case the wakeup condition is still not satisfied, calling
            // `wake` ensures the task will be scheduled again to have a
            // second chance of registering a waker.
            waker.wake();
            // Rescheduling means that the task could spin infinitely if a
            // waking thread is preempted before resetting the state. This
            // is caught by loom and requires `spin_loop` to escape the
            // infinite loop. In practice, calling `wake` is expected to
            // already do the job of `spin_loop`.
            #[cfg(loom)]
            ::loom::hint::spin_loop();
            return;
        }
        // We voluntarily don't handle `state & EMPTY != 0` in `register` and
        // only handle index 0 instead to avoid dependency on the state when
        // computing `self.wakers[0].will_wake(&waker)`, allowing speculative
        // execution.
        if CACHED && state & EMPTY != 0 {
            // SAFETY: same as in `register`
            unsafe {
                if state == EMPTY {
                    // State is `EMPTY | 0`, but the cached waker needs to be overwritten.
                    self.wakers[0].drop();
                    self.wakers[0].set(waker);
                } else if self.wakers[1].will_wake(&waker) {
                    // If the cached waker at index 1 matches, it is moved to
                    // index 0 to optimize future `register`.
                    self.wakers[0].set(ManuallyDrop::into_inner(self.wakers[1].get()));
                } else {
                    // Otherwise, overwrite the cached waker, writing the new
                    // one at index 0 to optimize future `register`.
                    self.wakers[1].drop();
                    self.wakers[0].set(waker);
                }
            }
            // same as in `register`
            if SYNC || cfg!(loom) {
                self.state.swap(0, SeqCst);
            } else {
                self.state.store(0, SeqCst);
            }
            return;
        }
        let cur_idx = state;
        // SAFETY: state is not EMPTY nor WAKING, so it must be the cell index
        // of a registered waker.
        unsafe { assert_unchecked(cur_idx < 2) };
        // SAFETY: `overwrite` cannot be called concurrently, but `wake` could. However,
        // both access the cell immutably, so it is safe.
        if unsafe { self.wakers[cur_idx].will_wake(&waker) } {
            return;
        }
        let new_idx = (cur_idx + 1) % 2;
        // SAFETY: SeqCst protect against outdated read, and `overwrite` cannot be called
        // concurrently. It means that `wake` can only access the cell at `cur_idx`, so
        // the cell at `new_idx` is safe to access mutably.
        unsafe { self.wakers[new_idx].set(waker) };
        // The cell index is attempted to be swapped with the new one just initialized.
        if let Err(state) = (self.state).compare_exchange(cur_idx, new_idx, SeqCst, SeqCst) {
            // State update failed, which means a concurrent `wake` was happening.
            // For the same reason as above, the task is rescheduled.
            debug_assert!(state >= 2);
            // SAFETY: state has not been updated, so `new_idx` cell is still safe
            // to access, and the waker previously set can be taken back.
            unsafe { ManuallyDrop::into_inner(self.wakers[new_idx].get()).wake() }
        } else {
            // SAFETY: cell index has been successfully swapped, so the cell
            // at `cur_idx` is now safe to access to drop its waker.
            unsafe { self.wakers[cur_idx].drop() };
        }
    }

    /// Removes the registered waker if there is one, returning `true` in this case.
    ///
    /// It allows avoiding spurious wakeups when a waker has been registered,
    /// but the wake condition is already met.
    ///
    /// # Safety
    ///
    /// `register` and `unregister` methods must not be called concurrently
    /// from multiple threads.
    #[inline]
    pub unsafe fn unregister(&self) -> bool {
        #[cfg(all(debug_assertions, not(loom)))]
        let _guard = self.exclusive.check();
        let state = self.load_state();
        if let Some(waker_cell) = self.wakers.get(state) {
            let empty = if CACHED { state | EMPTY } else { EMPTY };
            let res = self.state.compare_exchange(state, empty, SeqCst, Relaxed);
            match res {
                // SAFETY: state has been swapped to EMPTY, so the cell can
                // no longer be accessed by `wake`, and its waker can be taken
                Ok(_) if !CACHED => unsafe { waker_cell.drop() },
                Ok(_) => {}
                Err(s) => debug_assert!(s >= 2),
            }
            return res.is_ok();
        }
        false
    }

    fn wake_impl(&self) -> Option<ManuallyDrop<Waker>> {
        if SYNC {
            // SYNC=true requires a Release write on the state, but we don't want to set
            // the WAKING bit if there is no waker, as it would require unsetting it.
            // So we attempt a `fetch_add(0)` and hope for no concurrent `register`.
            if self.state.load(Relaxed) >= 2 && self.state.fetch_add(0, SeqCst) >= 2 {
                return None;
            }
            // There should be a waker registered, set the WAKING bit.
            let state = self.state.fetch_or(WAKING, SeqCst);
            // A concurrent `wake` has won the race, just return.
            if state & WAKING != 0 {
                return None;
            }
            if let Some(waker_cell) = self.wakers.get(state) {
                // SAFETY: the state is locked on WAKING, the cell can be concurrently
                // accessed with `will_wake`, but it can still be accessed immutably.
                // The waker is taken before resetting the state.
                let waker = unsafe { waker_cell.get() };
                // At this point the only concurrent operation will be:
                // - fetch_add(0), no issue
                // - fetch_or(WAKING), another `wake` is losing the race
                // - CAS(new_idx, cur_idx), will fail because of WAKING flag
                // The state can thus be swapped to EMPTY without issue.
                // It could be tempting to use a store instead, but it would not
                // work as it might overwrite a potential fetch_or and prevent
                // the synchronization of a racing wake with the next register.
                let empty = if CACHED { state | EMPTY } else { EMPTY };
                self.state.swap(empty, SeqCst);
                Some(waker)
            } else {
                // Too bad, no waker was registered. It means that a concurrent `register`
                // might be concurrently storing a waker in cell 0 and swap the state with
                // EMPTY. We still need to unset the WAKING flag, but we don't care if it
                // fails, as it would mean the flag has been unset anyway.
                // It is theoretically possible that WAKING flag has been already unset and
                // that another thread has already set it back. In this case, either the
                // state was not EMPTY and this CAS will fail, or the state was EMPTY and
                // the other thread doesn't care as much as us about its CAS succeeding.
                debug_assert!((CACHED && state & EMPTY != 0) || (!CACHED && state == EMPTY));
                let _ = (self.state).compare_exchange(state | WAKING, state, SeqCst, Relaxed);
                None
            }
        } else {
            // Load the state to check if there is a registered waker.
            let state = self.load_state();
            let waker_cell = self.wakers.get(state)?;
            // Try swapping the state with WAKING. If it fails, it means either:
            // - a concurrent `wake` has won the race, so we can return
            // - the waker was overwritten, so the registering thread is supposed
            //   to check again its wakeup condition, so we can just return
            (self.state.compare_exchange(state, WAKING, SeqCst, Relaxed)).ok()?;
            // SAFETY: the state has been swapped, so a concurrent `overwrite` CAS
            // will fail, and it is safe to access the cell to take its waker
            let waker = unsafe { waker_cell.get() };
            // The state can be reset to EMPTY with a simple store.
            // (loom doesn't support SeqCst and uses RMW operation instead)
            let empty = if CACHED { state | EMPTY } else { EMPTY };
            if cfg!(loom) {
                self.state.swap(empty, SeqCst);
            } else {
                self.state.store(empty, SeqCst);
            }
            Some(waker)
        }
    }

    /// Calls `wake` on the last `Waker` passed to `register`.
    ///
    /// If `register` has not been called yet, then this does nothing.
    #[inline]
    pub fn wake(&self) {
        if let Some(waker) = self.wake_impl() {
            if CACHED {
                waker.wake_by_ref();
            } else {
                ManuallyDrop::into_inner(waker).wake();
            }
        }
    }
}

impl<const SYNC: bool> SpmcWaker<SYNC, false> {
    /// Returns the last `Waker` passed to `register`, so that the caller can wake it.
    ///
    /// Sometimes, just waking the `SpmcWaker` is not fine-grained enough. This allows the caller
    /// to take the waker and then wake it separately, rather than performing both steps in one
    /// atomic action.
    ///
    /// If a waker has not been registered, this returns `None`.
    pub fn take(&self) -> Option<Waker> {
        self.wake_impl().map(ManuallyDrop::into_inner)
    }
}

impl<const SYNC: bool, const CACHED: bool> Default for SpmcWaker<SYNC, CACHED> {
    fn default() -> Self {
        Self::new()
    }
}
