//! A synchronization primitive for task wakeup.
//!
//! This crate provides [`SpmcWaker`], a single-producer, multiple-consumer (SPMC)
//! atomic waker.
//!
//! # Features
//!
//! - `portable-atomic`: use `portable-atomic` crate to provide functionality to
//!   targets without atomics.
#![no_std]
#[cfg(doc)]
extern crate std;
use core::{hint::assert_unchecked, mem::ManuallyDrop, task::Waker};

use crate::{
    loom::{
        AtomicUsizeExt,
        sync::atomic::{
            AtomicUsize,
            Ordering::{Relaxed, SeqCst},
        },
    },
    waker_cell::WakerCell,
};

#[cfg(all(debug_assertions, not(loom)))]
mod exclusive;
mod loom;
mod waker_cell;

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
/// Here is a simple example providing a `Flag` that can be signaled manually
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
    /// calling `wake`. This will result in the `register` caller's current
    /// task being notified once. A concurrent `wake` may prevent `register`
    /// to succeed, in which case it will return `false`. If despite the
    /// concurrent `wake`, the wakeup condition is still not fulfilled, then
    /// `Waker::wake` might be called to reschedule the task and give it
    /// another opportunity to register is waker — this would be equivalent
    /// to [`std::thread::yield_now`]. It is also possible to call `register`
    /// in small [spin-loop](std::hint::spin_loop), before falling back to
    /// calling `Waker::wake`.
    ///
    /// # Safety
    ///
    /// `register` and `unregister` methods must not be called concurrently
    /// from multiple threads.
    #[inline]
    pub unsafe fn register(&self, waker: &Waker) -> bool {
        #[cfg(all(debug_assertions, not(loom)))]
        let _guard = self.exclusive.check();
        // State is loaded and expected to be EMPTY. Otherwise, it means
        // there already is a registered waker that needs to be overwritten.
        let state = self.state.load(SeqCst);
        // The case `CACHED && state == EMPTY | 1` is handled in `overwrite`.
        if state == EMPTY {
            // SAFETY: SeqCst protect against outdated read, and `register`
            // cannot be called concurrently. It means that reading EMPTY
            // ensures there cannot be any registered waker at this point.
            // A concurrent `wake` will thus not attempt any read, so it's
            // safe to access both cells mutably.
            unsafe {
                if !CACHED {
                    self.wakers[0].set(waker.clone());
                } else if !self.wakers[0].will_wake(waker) {
                    return self.overwrite(waker, state);
                }
            }
            // SYNC=true uses swap, as `wake` must synchronize with `register`
            if SYNC {
                self.state.swap(0, SeqCst);
            } else {
                self.state.store(0, SeqCst);
            }
            true
        } else {
            self.overwrite(waker, state)
        }
    }

    // Overwriting a registered waker is expected to be rare, hence the `#[cold]` attribute.
    #[cold]
    fn overwrite(&self, waker: &Waker, state: usize) -> bool {
        // A concurrent `wake` may be happening.
        if (SYNC && state & WAKING != 0) || (!SYNC && state == WAKING) {
            // A thread is currently waking the registered waker, so we can
            // assume we should not wait and return immediately.
            // If a waking thread is preempted before resetting the state,
            // the task could loop infinitely on this state. This
            // is caught by loom and requires `spin_loop` to escape the
            // infinite loop. In practice, `spin_loop` or `Waker::wake`
            // are already expected to be called in between.
            #[cfg(loom)]
            ::loom::hint::spin_loop();
            return false;
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
                    self.wakers[0].set(waker.clone());
                } else if self.wakers[1].will_wake(waker) {
                    // If the cached waker at index 1 matches, it is moved to
                    // index 0 to optimize future `register`.
                    self.wakers[0].set(ManuallyDrop::into_inner(self.wakers[1].get()));
                } else {
                    // Otherwise, overwrite the cached waker, writing the new
                    // one at index 0 to optimize future `register`.
                    self.wakers[1].drop();
                    self.wakers[0].set(waker.clone());
                }
            }
            // same as in `register`
            if SYNC {
                self.state.swap(0, SeqCst);
            } else {
                self.state.store(0, SeqCst);
            }
            return true;
        }
        let cur_idx = state;
        // SAFETY: state is not EMPTY nor WAKING, so it must be the cell index
        // of a registered waker.
        unsafe { assert_unchecked(cur_idx < 2) };
        // If the new waker wakes the same task, there is no need to replace it.
        // Crucially, no state update is needed even for `SYNC=true`: the `SeqCst`
        // load at the top of `register` already participates in the total SeqCst
        // order, so any release from a preceding `wake` is already visible to
        // the caller — the synchronization guarantee is satisfied regardless.
        // SAFETY: `overwrite` cannot be called concurrently, but `wake` could. However,
        // both access the cell immutably, so it is safe.
        if unsafe { self.wakers[cur_idx].will_wake(waker) } {
            return true;
        }
        let new_idx = (cur_idx + 1) % 2;
        // SAFETY: SeqCst protect against outdated read, and `overwrite` cannot be called
        // concurrently. It means that `wake` can only access the cell at `cur_idx`, so
        // the cell at `new_idx` is safe to access mutably.
        unsafe { self.wakers[new_idx].set(waker.clone()) };
        // The cell index is attempted to be swapped with the new one just initialized.
        if let Err(state) = (self.state).compare_exchange(cur_idx, new_idx, SeqCst, SeqCst) {
            // State update failed, which means a concurrent `wake` was happening.
            // The registered waker should be dropped.
            debug_assert!(state >= 2);
            // SAFETY: state has not been updated, so `new_idx` cell is still safe
            // to access, and the waker previously set can be taken back.
            unsafe { ManuallyDrop::drop(&mut self.wakers[new_idx].get()) }
            false
        } else {
            // SAFETY: cell index has been successfully swapped, so the cell
            // at `cur_idx` is now safe to access to drop its waker.
            unsafe { self.wakers[cur_idx].drop() };
            true
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
        let state = self.state.load(SeqCst);
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

    /// Returns `true` if a waker is currently registered.
    ///
    /// This provides a best-effort snapshot: a concurrent [`wake`] call may
    /// consume the waker right after this returns `true`, and a concurrent
    /// [`register`] call may store one right after this returns `false`.
    ///
    /// Calling `has_waker_registered` then `wake` if it is returned `true`
    /// is guaranteed to provide the same synchronization as calling `wake`
    /// alone.
    ///
    /// [`register`]: Self::register
    /// [`wake`]: Self::wake
    #[inline]
    pub fn has_waker_registered(&self) -> bool {
        if SYNC {
            // See `check_before_wake` about `fetch_add(0)`
            self.state.load(Relaxed) < 2 || self.state.fetch_add(0, SeqCst) < 2
        } else {
            self.state.load(SeqCst) < 2
        }
    }

    /// Calls `wake` on the last `Waker` passed to `register`.
    ///
    /// If `register` has not been called yet, then this does nothing.
    #[inline]
    pub fn wake(&self) {
        self.check_before_wake(false, Self::wake_waker);
    }

    /// Same as [`wake`](Self::wake), but with the waking path marked `#[cold]`.
    ///
    /// This allows the method to inline more effectively. Prefer this over
    /// `wake` when waking is the uncommon case.
    #[inline]
    pub fn wake_cold(&self) {
        self.check_before_wake(true, Self::wake_waker);
    }

    fn wake_waker(waker: Option<ManuallyDrop<Waker>>) {
        match waker {
            Some(w) if CACHED => w.wake_by_ref(),
            Some(w) if !CACHED => ManuallyDrop::into_inner(w).wake(),
            _ => {}
        }
    }

    #[inline(always)]
    fn check_before_wake<R>(
        &self,
        cold: bool,
        wake: impl FnOnce(Option<ManuallyDrop<Waker>>) -> R,
    ) -> R {
        if SYNC {
            if cold {
                // SYNC=true requires a Release write on the state, but we don't want to set
                // the WAKING bit if there is no waker, as it would require unsetting it.
                // So we attempt a `fetch_add(0)` and hope for no concurrent `register`.
                if self.state.load(Relaxed) >= 2 && self.state.fetch_add(0, SeqCst) >= 2 {
                    return wake(None);
                }
                self.wake_sync_cold(wake)
            } else {
                self.wake_sync(wake)
            }
        } else {
            // Load the state to check if there is a registered waker.
            let state = self.state.load(SeqCst);
            if state >= 2 {
                wake(None)
            } else if cold {
                self.wake_unsync_cold(state, wake)
            } else {
                self.wake_unsync(state, wake)
            }
        }
    }

    fn wake_sync<R>(&self, wake: impl FnOnce(Option<ManuallyDrop<Waker>>) -> R) -> R {
        // There might be a waker registered, set the WAKING bit.
        let state = self.state.fetch_or(WAKING, SeqCst);
        // A concurrent `wake` has won the race, just return.
        if state & WAKING != 0 {
            return wake(None);
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
            wake(Some(waker))
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
            wake(None)
        }
    }

    #[cold]
    #[inline(never)]
    fn wake_sync_cold<R>(&self, wake: impl FnOnce(Option<ManuallyDrop<Waker>>) -> R) -> R {
        self.wake_sync(wake)
    }

    fn wake_unsync<R>(
        &self,
        state: usize,
        wake: impl FnOnce(Option<ManuallyDrop<Waker>>) -> R,
    ) -> R {
        unsafe { assert_unchecked(state < 2) };
        // Try swapping the state with WAKING. If it fails, it means either:
        // - a concurrent `wake` has won the race, so we can return
        // - the waker was overwritten, so the registering thread is supposed
        //   to check again its wakeup condition, so we can just return
        if (self.state.compare_exchange(state, WAKING, SeqCst, Relaxed)).is_err() {
            return wake(None);
        };
        // SAFETY: the state has been swapped, so a concurrent `overwrite` CAS
        // will fail, and it is safe to access the cell to take its waker
        let waker = unsafe { self.wakers[state].get() };
        // The state can be reset to EMPTY with a simple store.
        // (loom doesn't support SeqCst and uses RMW operation instead)
        let empty = if CACHED { state | EMPTY } else { EMPTY };
        self.state.store(empty, SeqCst);
        wake(Some(waker))
    }

    #[cold]
    #[inline(never)]
    fn wake_unsync_cold<R>(
        &self,
        state: usize,
        wake: impl FnOnce(Option<ManuallyDrop<Waker>>) -> R,
    ) -> R {
        self.wake_unsync(state, wake)
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
        self.check_before_wake(false, Self::take_waker)
    }

    /// Same as [`take`](Self::take), but with the taking path marked `#[cold]`.
    ///
    /// This allows the method to inline more effectively. Prefer this over
    /// `take` when taking is the uncommon case.
    #[inline]
    pub fn take_cold(&self) -> Option<Waker> {
        self.check_before_wake(true, Self::take_waker)
    }

    fn take_waker(waker: Option<ManuallyDrop<Waker>>) -> Option<Waker> {
        waker.map(ManuallyDrop::into_inner)
    }
}

impl<const SYNC: bool, const CACHED: bool> Default for SpmcWaker<SYNC, CACHED> {
    fn default() -> Self {
        Self::new()
    }
}
