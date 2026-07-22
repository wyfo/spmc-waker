//! A wait-free synchronization primitive for task wakeup.
//!
//! This crate provides [`SpmcWaker`], a single-producer, multiple-consumer (SPMC)
//! atomic waker.
//!
//! # Features
//!
//! - `portable-atomic`: use `portable-atomic` crate to provide functionality to
//!   targets without atomics.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![warn(missing_docs)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![no_std]

#[cfg(miri)]
use core::cell::Cell;
#[cfg(not(loom))]
#[cfg(not(feature = "portable-atomic"))]
use core::sync::atomic;
use core::{
    marker::PhantomData,
    mem,
    mem::ManuallyDrop,
    panic::{RefUnwindSafe, UnwindSafe},
    ptr,
    sync::atomic::Ordering::*,
    task::{Context, Poll, RawWakerVTable, Waker},
};

#[cfg(loom)]
use ::loom::cell::Cell;
#[cfg(loom)]
use loom as atomic;
#[cfg(not(loom))]
#[cfg(feature = "portable-atomic")]
use portable_atomic as atomic;

use crate::{
    atomic::{AtomicPtr, AtomicUsize, fence},
    registration::{RegistrationMode, RegistrationPolicy, SafeRegistration, Strict, Unchecked},
    state_machine::{
        AtomicState, CACHED, FLAGS_MASK, REGISTERED, REGISTERING, REGISTRATION_INCR, State,
    },
    synchronization::{SyncMode, Synchronization, Synchronized},
    utils::{ConfirmedWaker, PendingWaker, TaggedExt},
    wait_until::{WaitUntil, WakeCondition},
};

#[cfg(loom)]
#[doc(hidden)]
pub mod loom;
pub mod registration;
mod state_machine;
pub mod synchronization;
mod utils;
pub mod wait_until;

/// A synchronization primitive for task wakeup.
///
/// `SpmcWaker` allows registering a task's [`Waker`] and waking the task atomically and
/// concurrently from multiple threads. A single instance may be reused for any number of waker
/// registrations/calls.
///
/// `SpmcWaker` should be paired with a wake condition, met **before** waking the task, and checked
/// **after** registering the task's waker to not miss a concurrent notification that happened
/// before.
///
/// `SpmcWaker` also provides a high-level async [`wait_until`](Self::wait_until), backed by
/// [`poll_wait_until`](Self::poll_wait_until), which is often more optimized than manual
/// registration through [`register`](Self::register).
///
/// Another optimization is to use [`wake_cold`](Self::wake_cold) instead of [`wake`](Self::wake)
/// in hot path, especially when a waker is rarely expected to be registered, as it is more
/// inlinable.
///
/// # Synchronization
///
/// `SpmcWaker` has a generic `S` parameter which determines the synchronization guarantees. See
/// [`Synchronization`] documentation for more details about its variants.
///
/// With the default [`Synchronized`], calling `register` "acquires" all memory "released" by calls
/// to `wake` before the call to `register`. Later calls to `wake` will wake the registered waker.
///
/// # Waker caching
///
/// Most of the time, `SpmcWaker` is used in a single task, so the waker registered is always the
/// same. That's why it provides a second generic parameter `CACHING`.
///
/// With `CACHING=true`, the latest waker registered is kept cached to avoid cloning on the next
/// registration. As a consequence, tasks are woken with [`Waker::wake_by_ref`] instead of
/// [`Waker::wake`].
///
/// As wakers are often `Arc`s, caching avoids atomic RMW operations updating the reference counter.
/// However, it adds an RMW operation to `SpmcWaker::wake`, so the benefit mostly concerns
/// `SpmcWaker::register`.
///
/// # Single-producer, multiple-consumer (SPMC)
///
/// `SpmcWaker` algorithm assumes a single thread registering a waker at a time. The behavior
/// in case of concurrent registration is determined by the generic parameter `R`, see
/// [`RegistrationPolicy`].
///
/// Notably, `R=Unchecked` makes registration methods unsafe, but removes one RMW from `register`.
///
/// # Progress guarantee
///
/// `SpmcWaker` algorithm is wait-free, so every operation is bounded by the underlying `Waker`
/// operation.
///
/// # Examples
///
/// Here is a simple example providing a `Notifier`/`Waiter` pair.
///
/// ```rust
/// use std::{
///     pin::Pin,
///     sync::{
///         Arc,
///         atomic::{AtomicBool, Ordering::Relaxed},
///     },
///     task::{Context, Poll},
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
///     pub fn new() -> Self {
///         Self(Arc::new(Inner {
///             waker: SpmcWaker::new(),
///             notified: AtomicBool::new(false),
///         }))
///     }
///
///     pub fn signal(&self) {
///         self.0.notified.store(true, Relaxed);
///         self.0.waker.wake();
///     }
/// }
///
/// #[derive(Default)]
/// struct Waiter(Arc<Inner>);
///
/// impl Waiter {
///     fn notifier(&self) -> Notifier {
///         Notifier(self.0.clone())
///     }
/// }
///
/// impl Future for Waiter {
///     type Output = ();
///
///     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
///         let waker = &self.0.waker;
///         waker.poll_wait_until(cx, |_| self.0.notified.load(Relaxed))
///     }
/// }
///
/// fn event() -> (Notifier, Waiter) {
///     let waiter = Waiter::default();
///     (waiter.notifier(), waiter)
/// }
/// ```
#[derive(Debug)]
pub struct SpmcWaker<
    S: Synchronization = Synchronized,
    const CACHING: bool = false,
    R: RegistrationPolicy = Strict,
> {
    /// `SpmcWaker` state, as documented in [`state_machine`] module.
    state: AtomicState,
    /// Waker data
    data: AtomicPtr<()>,
    /// Waker vtable
    vtable: AtomicPtr<RawWakerVTable>,
    /// Ghost field used to check that a confirmed waker
    /// correctly matches the registration it is supposed to.
    #[cfg(any(loom, miri))]
    store_epoch: AtomicUsize,
    /// Ghost field used to check that `Waker::wake_by_ref` is
    /// correctly synchronized with `Waker::drop`.
    #[cfg(any(loom, miri))]
    waker_cells: [Cell<u8>; 256], // 256 should cover any loom/miri test.
    _sync: PhantomData<S>,
    _registration: PhantomData<R>,
}

// SAFETY: loom/miri catch data races
#[cfg(any(loom, miri))]
unsafe impl<S: Synchronization, const CACHING: bool, R: RegistrationPolicy> Sync
    for SpmcWaker<S, CACHING, R>
{
}

impl<S: Synchronization, const CACHING: bool, R: RegistrationPolicy> UnwindSafe
    for SpmcWaker<S, CACHING, R>
{
}
impl<S: Synchronization, const CACHING: bool, R: RegistrationPolicy> RefUnwindSafe
    for SpmcWaker<S, CACHING, R>
{
}

impl<S: Synchronization, const CACHING: bool, R: RegistrationPolicy> Drop
    for SpmcWaker<S, CACHING, R>
{
    #[inline]
    fn drop(&mut self) {
        let state = self.state.load(Relaxed);
        if state.has(REGISTERED) || (CACHING && state.has(CACHED)) {
            drop(self.load_waker().confirm(state));
        }
    }
}

impl<S: Synchronization, const CACHING: bool, R: RegistrationPolicy> SpmcWaker<S, CACHING, R> {
    /// Creates a new `SpmcWaker`.
    #[cfg_attr(loom, const_fn::const_fn(cfg(false)))]
    #[inline]
    pub const fn new() -> Self {
        Self {
            state: AtomicState::new(0),
            vtable: AtomicPtr::new(ptr::null_mut()),
            data: AtomicPtr::new(ptr::null_mut()),
            #[cfg(any(loom, miri))]
            store_epoch: AtomicUsize::new(0),
            #[cfg(loom)]
            waker_cells: core::array::from_fn(|_| Cell::new(0)),
            #[cfg(miri)]
            waker_cells: [const { Cell::new(0) }; _],
            _sync: PhantomData,
            _registration: PhantomData,
        }
    }

    #[inline(always)]
    fn load_waker(&self) -> PendingWaker {
        PendingWaker {
            data: self.data.load(Relaxed),
            vtable: self.vtable.load(Relaxed),
            #[cfg(any(loom, miri))]
            store_epoch: self.store_epoch.load(Relaxed),
            #[cfg(any(loom, miri))]
            waker_cells: &self.waker_cells,
        }
    }

    #[inline(always)]
    fn store_waker(&self, waker: Waker, _state: &mut State) {
        let waker = ManuallyDrop::new(waker);
        self.data.store(waker.data().cast_mut(), Relaxed);
        self.vtable.store(waker.vtable() as *const _ as _, Relaxed);
        #[cfg(any(loom, miri))]
        {
            let epoch = state_machine::set_store_epoch(_state);
            self.store_epoch.store(epoch, Relaxed);
            self.waker_cells[epoch / REGISTRATION_INCR].set(0);
        }
    }

    #[inline(always)]
    fn poll_wait_until_impl<F: FnMut(bool) -> W, W: WakeCondition>(
        &self,
        cx: &mut Context,
        mut wake_condition: F,
    ) -> Poll<W::Output> {
        // Quick check to avoid registration if the wake condition is already met.
        if let Some(out) = wake_condition(false).try_into_output() {
            return Poll::Ready(out);
        }
        // Register the waker
        let registered = self.register_impl(cx.waker());
        // Check the wake condition **after** registering the waker.
        if let Some(out) = wake_condition(true).try_into_output() {
            registered.unregister();
            return Poll::Ready(out);
        }
        Poll::Pending
    }

    #[inline(always)]
    fn register_impl(&self, waker: &Waker) -> Registered<'_, S, CACHING, R> {
        // Load the state. If CACHING, the state must be loaded with Acquire ordering
        // to synchronize with the Release CAS setting the CACHED flag (which is supposed
        // to be the common case); if R::SAFE, this is already done by `set_registering`.
        // Otherwise, there is nothing to synchronize with, as `wake` takes the ownership
        // of the registered waker, hence Relaxed ordering.
        let mut state = self.state.load(if CACHING && !R::SAFE {
            Acquire
        } else {
            Relaxed
        });
        // Set the REGISTERING flag if R::SAFE to prevent concurrent registrations.
        if R::SAFE {
            match self.set_registering(state) {
                Ok(s) => state = s,
                Err(s) => return Registered::new(self, s),
            }
        }
        // If CACHING and the same waker is cached, then the state must just be
        // set back to REGISTERED, no need to touch the waker.
        let state = if CACHING && state.has(CACHED) && self.load_waker().will_wake(waker) {
            // The registration counter must still be incremented, even though the
            // waker is untouched: with a state value identical to the previous
            // registration, a waking thread could pair its acquire fence with the
            // previous registration's store, then successfully claim this one, and
            // use/drop the waker without synchronizing with the previous wake (ABA).
            // The stored epoch is untouched, as the waker storage doesn't change
            // (see state_machine module documentation).
            let unflagged = state.unset(CACHED).wrapping_add(REGISTRATION_INCR);
            self.register_waker(state, unflagged, None)
        // if not CACHING and no waker is registered, then store it and update
        // the state.
        } else if !CACHING && !state.has(REGISTERED) {
            self.register_waker(state, state, Some(waker))
        // Other cases fall in cold path
        } else {
            self.register_impl_cold(waker, state)
        };
        Registered::new(self, state)
    }

    #[inline]
    fn set_registering(&self, mut state: State) -> Result<State, State> {
        // Try setting the registering with a CAS loop (because it needs to
        // clear other mutually exclusive flags).
        while !state.has(REGISTERING) {
            let new_state = state.clear(FLAGS_MASK).set(REGISTERING);
            // Acquire ordering on success is mandatory because a waker may have been
            // stored by another thread, so the synchronization is needed as the waker
            // will be loaded right after.
            match (self.state).compare_exchange_weak(state, new_state, Acquire, Relaxed) {
                Ok(_) => return Ok(state),
                Err(s) => state = s,
            }
        }
        match R::MODE {
            RegistrationMode::Strict => panic!("concurrent registration"),
            // Lenient can't panic and must return a state to build a RegisteredWaker.
            // By returning a previous registration count, we ensure the RegisteredWaker
            // cannot be used for anything.
            RegistrationMode::Lenient => Err(state
                .unset(REGISTERING)
                .wrapping_sub(REGISTRATION_INCR)
                .set(REGISTERED)),
            RegistrationMode::Unchecked => unreachable!(),
        }
    }

    #[inline(always)]
    fn register_waker(&self, state: State, unflagged_state: State, waker: Option<&Waker>) -> State {
        let mut new_state = unflagged_state.set(REGISTERED);
        if let Some(waker) = waker {
            new_state = new_state.wrapping_add(REGISTRATION_INCR);
            // Don't forget to reset the REGISTERING flag, even if it's not necessary for
            // correctness, to not leave the SpmcWaker in a blocked state.
            struct ResetRegistering<'a>(&'a AtomicState, State);
            impl Drop for ResetRegistering<'_> {
                fn drop(&mut self) {
                    self.0.swap(self.1, SeqCst); // !ORDERING
                }
            }
            let guard = R::SAFE.then(|| ResetRegistering(&self.state, state));
            self.store_waker(waker.clone(), &mut new_state);
            mem::forget(guard);
        }
        match S::MODE {
            // Acquire ordering is necessary to synchronize with `wake`, so swap
            // must be used. Release is necessary if waker has been stored.
            // Otherwise, the swap write can be relaxed: a `wake` claiming this
            // registration still acquires the data through the release sequence
            // of the previous registration of the same waker, as every state
            // updates are RMWs.
            SyncMode::Synchronized => {
                (self.state).swap(new_state, if waker.is_some() { AcqRel } else { Acquire });
            }
            // Storing the state with SeqCst is necessary for the pattern
            // `store X; load Y || store Y; load X` to not miss any
            // notification, where X is the wake condition and Y the state.
            SyncMode::Sequential => self.state.store(new_state, SeqCst),
            // Even if synchronization is handled by the user and no data is
            // written, Release is still needed to synchronize with the initial
            // data write, which happens before (either on the same thread with
            // Unchecked, or in another thread with R::SAFE but with Acquire-Release
            // synchronization). The store is in fact breaking the release-sequence
            // headed by the initial store.
            SyncMode::Unsynchronized => self.state.store(new_state, Release),
        }
        new_state
    }

    // Overwriting a registered/cached waker is expected to be rare, hence the `#[cold]` attribute.
    #[cold]
    fn register_impl_cold(&self, waker: &Waker, state: State) -> State {
        if CACHING && !state.has(REGISTERED) {
            // If a waker is cached, but doesn't match the new one being registered (because in
            // cold path), it is simply overwritten and dropped.
            return if state.has(CACHED) {
                let old_waker = self.load_waker();
                debug_assert!(!old_waker.will_wake(waker));
                let registered = self.register_waker(state, state.unset(CACHED), Some(waker));
                drop(old_waker.confirm(state));
                registered
            // Otherwise, the waker is unregistered but the waking thread may try to update the
            // state back to CACHED. This is not possible with R::SAFE (because of REGISTERING
            // flag), so a new waker can simply be registered.
            } else if R::SAFE {
                self.register_waker(state, state, Some(waker))
            // However, concurrent update may happen with Unchecked and must be caught.
            // This is done by registering the new waker with a swap.
            } else {
                debug_assert!(!state.has(FLAGS_MASK));
                let old_waker = self.load_waker();
                let mut new_state = state.set(REGISTERED).wrapping_add(REGISTRATION_INCR);
                self.store_waker(waker.clone(), &mut new_state);
                // Ordering matches those of `register_waker` for the same reasons.
                // (The ordering could be factorized with a unique swap, but it would
                // mess with ordering downgrading parsing)
                let old_state = match S::MODE {
                    SyncMode::Synchronized => self.state.swap(new_state, AcqRel),
                    SyncMode::Sequential => self.state.swap(new_state, SeqCst),
                    SyncMode::Unsynchronized => self.state.swap(new_state, Release),
                };
                // If the old waker was concurrently cached, drop it, but emit an Acquire fence
                // first (if not already handled by the swap ordering) to ensure `wake_by_ref`
                // happens before the drop.
                if old_state.has(CACHED) {
                    if matches!(S::MODE, SyncMode::Unsynchronized) {
                        fence(Acquire);
                    }
                    drop(old_waker.confirm(old_state));
                }
                new_state
            };
        }
        // A waker is registered, check first if it can be reused.
        debug_assert!(state.has(REGISTERED));
        let old_waker = self.load_waker();
        if old_waker.will_wake(waker) {
            // REGISTERING flag must be unset.
            if R::SAFE {
                self.register_waker(state, state.unset(REGISTERED), None);
            }
            state
        // Otherwise, the waker must be replaced. With R::SAFE, REGISTERING flag protects against
        // concurrent `wake`, so the new waker can be overwritten.
        } else if R::SAFE {
            let registered = self.register_waker(state, state.unset(REGISTERED), Some(waker));
            drop(old_waker.confirm(state));
            registered
        // In case where concurrent `wake` can happen and read the old waker, it is not possible to
        // overwrite this one; it must be unregistered first, using a swap to catch if it has
        // already been unregistered (or cached).
        } else {
            let new_state = state.unset(REGISTERED);
            // Relaxed ordering is fine to set the state to unregistered. A concurrent `wake` may
            // load a stale REGISTERED, but its CAS would fail right after.
            let old_state = self.state.swap(new_state, Relaxed);
            debug_assert!(old_state & !FLAGS_MASK == state & !FLAGS_MASK);
            // If the old waker is still registered, it is dropped. It must be dropped before
            // cloning the new waker, as clone may panic and the old waker would then be leaked.
            if old_state.has(REGISTERED) {
                drop(old_waker.confirm(old_state));
            // If the waker has been cached after a concurrent `wake`, it is dropped after an
            // Acquire fence to ensure `wake_by_ref` happens before the drop.
            } else if CACHING && old_state.has(CACHED) {
                fence(Acquire);
                drop(old_waker.confirm(old_state));
            // Otherwise, if waker ownership has been acquired but there is still a chance that it
            // might be concurrently cached, it must be cached first using a CAS. Registering the
            // new waker with a swap (as done above) is tempting, but as explained before, dropping
            // the old waker must be done before cloning the new one.
            } else if CACHING {
                debug_assert!(!old_state.has(FLAGS_MASK));
                let cached = old_state.set(CACHED);
                // If the CAS succeeds, then `wake` CAS will fail and the waker will be dropped
                // subsequently. Otherwise, waker was already cached and the ownership has been
                // given back, so it must be dropped here.
                if let Err(old_state) =
                    (self.state).compare_exchange(old_state, cached, Relaxed, Acquire)
                {
                    debug_assert_eq!(old_state, cached);
                    drop(old_waker.confirm(cached));
                }
            } else {
                debug_assert!(!old_state.has(FLAGS_MASK));
            }
            // Then the new waker can be registered.
            self.register_waker(new_state, new_state, Some(waker))
        }
    }

    fn registered_impl(&self) -> Option<(State, S::Released)> {
        // Load the state with ordering depending on the synchronization:
        // - Sequential requires a SeqCst load
        // - Synchronized requires a RMW release on the state, but this one can be done to take
        //   the ownership of the registered waker; the initial load before the RMW can thus be
        //   Relaxed
        // - Unsynchronized has it synchronization handled by the wake condition, the load can be
        //   Relaxed
        let mut state = self.state.load(match S::MODE {
            SyncMode::Sequential => SeqCst,
            _ => Relaxed,
        });
        let mut released = false;
        // If S::SYNC and there is no waker registered, a Release RMW must still be executed.
        // As the state doesn't need to be modified, `fetch_add(0)` can be used, as it has the
        // advantage to be optimized on x86 architectures.
        if S::SYNC && !state.has(REGISTERED) {
            state = self.state.fetch_add(0, Release);
            released = true;
        }
        // No waker registered, return None.
        if !state.has(REGISTERED) {
            return None;
        }
        // The waker will be loaded to be wakened, so an Acquire fence must be emitted to synchronize
        // with the Release store/swap in `register`, ensuring the pointers loaded will be
        // up-to-date (they might even be too recent, but ownership claim would fail in that case).
        // It would be possible to replace the fence by an Acquire ordering on the previous load,
        // but the fence has multiple advantages: it is conditional, and the compilation of
        // `ldr + dmb ishld` can have better performance than `ldar` on some aarch64 architectures,
        // especially if the `ldar` is following a `stlr`, which it should in the common case.
        // (Recent aarch64 compiles Acquire load to `ldapr`, which doesn't suffer from the
        // combination with `stlr`, but the code is kept simpler this way. The optimization might
        // still be implemented in the future).
        // Also, for `wake_cold`, executing the fence before calling the cold function enables to
        // have it done in parallel to the function load, reducing latency.
        if !matches!(S::MODE, SyncMode::Sequential) {
            fence(Acquire);
        }
        Some((state, released.into()))
    }

    /// Consumes the latest `Waker` registered and returns it.
    #[inline]
    pub fn take(&self) -> Option<Waker> {
        let (state, released) = self.registered_impl()?;
        let (waker, _) = self.take_impl(state, released)?;
        #[cfg(any(loom, miri))]
        let waker = waker.get();
        Some(waker)
    }

    #[inline(always)]
    fn take_impl(
        &self,
        mut state: State,
        mut released: S::Released,
    ) -> Option<(ConfirmedWaker, State)> {
        // Claim the waker ownership with a CAS.
        debug_assert!(state.has(REGISTERED));
        loop {
            // Load the waker before updating the state. If the update succeeds, then
            // the waker is ensured to be valid.
            let waker = self.load_waker();
            let new_state = state.unset(REGISTERED);
            if ((self.state).compare_exchange(state, new_state, Release, Relaxed)).is_ok() {
                return Some((waker.confirm(state), new_state));
            }
            // Same as registered_impl. With S::SYNC, if the update failed, a Release RMW must
            // still be executed if it has not been done before.
            if S::SYNC && !released.into() {
                state = self.state.fetch_add(0, Release);
                released = true.into();
                // Try claiming the new waker if there is one.
                if state.has(REGISTERED) {
                    // Same as registered_impl, the fence is necessary to load the waker.
                    if !matches!(S::MODE, SyncMode::Sequential) {
                        fence(Acquire);
                    }
                    continue;
                }
            }
            return None;
        }
    }

    /// Consumes the latest `Waker` registered and wakes its task.
    #[inline]
    pub fn wake(&self) {
        if let Some((state, released)) = self.registered_impl() {
            self.wake_impl(state, released);
        }
    }

    /// Same as [`wake`](Self::wake), but with the waking path marked `#[cold]`.
    ///
    /// This allows the method to inline more effectively. Prefer this over
    /// `wake` when waking is the uncommon case.
    #[inline]
    pub fn wake_cold(&self) {
        if let Some((state, released)) = self.registered_impl() {
            self.wake_impl_cold(state, released);
        }
    }

    #[inline(always)]
    fn wake_impl(&self, state: State, released: S::Released) {
        if let Some((waker, state)) = self.take_impl(state, released) {
            // If CACHING is enabled, use `Waker::wake_by_ref` and try to update the state back
            // with the CACHED state, giving back the ownership of the waker. If it fails, drop it.
            if CACHING {
                waker.wake_by_ref();
                let cached = state.set(CACHED);
                // Use Release ordering so `wake_by_ref` can happen before a further drop.
                if ((self.state).compare_exchange(state, cached, Release, Relaxed)).is_ok() {
                    mem::forget(waker);
                }
            } else {
                waker.wake();
            }
        }
    }

    #[cold]
    fn wake_impl_cold(&self, state: State, released: S::Released) {
        self.wake_impl(state, released);
    }
}

impl<S: Synchronization, const CACHING: bool, R: SafeRegistration> SpmcWaker<S, CACHING, R> {
    /// Wait until the given wake condition is met.
    ///
    /// The method accepts a closure which takes in parameter a boolean telling whether the waker
    /// is already registered when the closure is called. In fact, the closure is executed a first
    /// time before registering the waker (see [`poll_wait_until`](Self::poll_wait_until)
    /// documentation). This parameter can be used to relax the first wake condition's check when a
    /// non-default [`Synchronization`] is used.
    ///
    /// Notifier threads should call [`wake`](Self::wake) (or [`wake_cold`](Self::wake_cold))
    /// after wake condition is met.
    ///
    /// # Panics
    ///
    /// Polling the returned future calls `poll_wait_until` and inherits its panic condition.
    /// Basically, only a single thread should await a wake condition at a time.
    #[inline]
    pub fn wait_until<F: FnMut(bool) -> W, W: WakeCondition>(
        &self,
        wake_condition: F,
    ) -> WaitUntil<'_, F, S, CACHING, R> {
        WaitUntil::new(self, wake_condition)
    }

    /// Returns `Poll::Ready` if the wake condition is met, or registers
    /// the task's waker to be notified.
    ///
    /// The method accepts a closure which takes in parameter a boolean telling whether the waker
    /// is already registered when the closure is called. This parameter can be used to relax the
    /// first wake condition's check when a non-default [`Synchronization`] is used.
    ///
    /// Notifier threads should call [`wake`](Self::wake) (or [`wake_cold`](Self::wake_cold))
    /// after wake condition is met.
    ///
    /// It is equivalent to the following code:
    /// ```
    /// # use std::task::{Context, Poll};
    /// # use spmc_waker::SpmcWaker;
    /// # fn poll_wait_until(spmc_waker: &SpmcWaker, cx: &mut Context, wake_condition: impl Fn(bool) -> bool) -> Poll<()> {
    ///     // quick check to avoid registration if the wake condition is already met
    ///     if wake_condition(false) {
    ///         return Poll::Ready(());
    ///     }
    ///     // register the waker
    ///     let registered = spmc_waker.register(cx.waker());
    ///     // check the wake condition **after** registering the waker
    ///     if wake_condition(true) {
    ///         // unregister the waker to avoid spurious wakeups
    ///         registered.unregister();
    ///         return Poll::Ready(());
    ///     }
    ///     Poll::Pending
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// This method calls `register` and inherits its panic condition.
    #[inline]
    pub fn poll_wait_until<F: FnMut(bool) -> W, W: WakeCondition>(
        &self,
        cx: &mut Context,
        wake_condition: F,
    ) -> Poll<W::Output> {
        self.poll_wait_until_impl(cx, wake_condition)
    }

    /// Registers a [`Waker`] whose task will be woken by the next call to `wake`.
    ///
    /// The waker overwrites any previously registered one, and is consumed by `wake`.
    /// As a consequence, `register` should be called **each** time a task requires a
    /// wakeup.
    ///
    /// Returns a [`Registered`] token, which can be used to unregister the waker if
    /// the wake condition is met, avoiding spurious wakeup.
    ///
    /// See [`poll_wait_until`](Self::poll_wait_until) documentation about how to use
    /// `register`.
    ///
    /// # Panics
    ///
    /// Panics if this method is called concurrently from multiple threads unless `R=Lenient`,
    /// in which case registration might fail silently.
    #[inline]
    pub fn register(&self, waker: &Waker) -> Registered<'_, S, CACHING, R> {
        self.register_impl(waker)
    }
}

impl<S: Synchronization, const CACHING: bool> SpmcWaker<S, CACHING, Unchecked> {
    /// Wait until the given wake condition is met.
    ///
    /// The method accepts a closure which takes in parameter a boolean telling whether the waker
    /// is already registered when the closure is called. In fact, the closure is executed a first
    /// time before registering the waker (see [`poll_wait_until`](Self::poll_wait_until)
    /// documentation). This parameter can be used to relax the first wake condition's check when a
    /// non-default [`Synchronization`] is used.
    ///
    /// Notifier threads should call [`wake`](Self::wake) (or [`wake_cold`](Self::wake_cold))
    /// after wake condition is met.
    ///
    /// # Safety
    ///
    /// Polling the returned future calls `poll_wait_until` and inherits its safety condition.
    /// Basically, only a single thread should await a wake condition at a time.
    #[inline]
    pub unsafe fn wait_until<F: FnMut(bool) -> W, W: WakeCondition>(
        &self,
        wake_condition: F,
    ) -> WaitUntil<'_, F, S, CACHING, Unchecked> {
        WaitUntil::new(self, wake_condition)
    }

    /// Returns `Poll::Ready` if the wake condition is met, or registers
    /// the task's waker to be notified.
    ///
    /// The method accepts a closure which takes in parameter a boolean telling whether the waker
    /// is already registered when the closure is called. This parameter can be used to relax the
    /// first wake condition's check when a non-default [`Synchronization`] is used.
    ///
    /// Notifier threads should call [`wake`](Self::wake) (or [`wake_cold`](Self::wake_cold))
    /// after wake condition is met.
    ///
    /// It is equivalent to the following code:
    /// ```
    /// # use std::task::{Context, Poll};
    /// # use spmc_waker::SpmcWaker;
    /// # fn poll_wait_until(spmc_waker: &SpmcWaker, cx: &mut Context, wake_condition: impl Fn(bool) -> bool) -> Poll<()> {
    ///     // quick check to avoid registration if the wake condition is already met.
    ///     if wake_condition(false) {
    ///         return Poll::Ready(());
    ///     }
    ///     // register the waker
    ///     let registered = spmc_waker.register(cx.waker());
    ///     // check the wake condition **after** registering the waker
    ///     if wake_condition(true) {
    ///         // unregister the waker to avoid spurious wakeups
    ///         registered.unregister();
    ///         return Poll::Ready(());
    ///     }
    ///     Poll::Pending
    /// # }
    /// ```
    ///
    /// # Safety
    ///
    /// This method calls `register` and inherits its safety condition.
    #[inline]
    pub unsafe fn poll_wait_until<F: FnMut(bool) -> W, W: WakeCondition>(
        &self,
        cx: &mut Context,
        wake_condition: F,
    ) -> Poll<W::Output> {
        self.poll_wait_until_impl(cx, wake_condition)
    }

    /// Registers a [`Waker`] whose task will be woken by the next call to `wake`.
    ///
    /// The waker overwrites any previously registered one, and is consumed by `wake`.
    /// As a consequence, `register` should be called **each** time a task requires a
    /// wakeup.
    ///
    /// Returns a [`Registered`] token, which can be used to unregister the waker if
    /// the wake condition is met, avoiding spurious wakeup.
    ///
    /// See [`poll_wait_until`](Self::poll_wait_until) documentation about how to use
    /// `register`.
    ///
    /// # Safety
    ///
    /// This method must not be called concurrently from multiple threads.
    #[inline]
    pub unsafe fn register(&self, waker: &Waker) -> Registered<'_, S, CACHING, Unchecked> {
        self.register_impl(waker)
    }
}

impl<S: Synchronization, const CACHING: bool, R: RegistrationPolicy> Default
    for SpmcWaker<S, CACHING, R>
{
    fn default() -> Self {
        Self::new()
    }
}

/// Token returned by [`SpmcWaker::register`] to unregister the waker if the wake condition is met.
pub struct Registered<
    'a,
    S: Synchronization = Synchronized,
    const CACHING: bool = false,
    R: RegistrationPolicy = Strict,
> {
    spmc_waker: &'a SpmcWaker<S, CACHING, R>,
    state: State,
}

impl<'a, S: Synchronization, const CACHING: bool, R: RegistrationPolicy>
    Registered<'a, S, CACHING, R>
{
    #[inline(always)]
    fn new(spmc_waker: &'a SpmcWaker<S, CACHING, R>, state: State) -> Self {
        debug_assert!(state.has(REGISTERED));
        Self { spmc_waker, state }
    }

    /// Unregisters the previously registered waker.
    ///
    /// It allows avoiding spurious wakeups if the wake condition is already met.
    #[inline]
    pub fn unregister(self) {
        let mut new_state = self.state.unset(REGISTERED);
        if CACHING {
            new_state = new_state.set(CACHED);
        }
        // If no caching, load the waker before claiming its ownership.
        let waker = (!CACHING).then(|| self.spmc_waker.load_waker());
        match (self.spmc_waker.state).compare_exchange(self.state, new_state, Relaxed, Relaxed) {
            Ok(_) if !CACHING => drop(waker.unwrap().confirm(self.state)),
            _ => {}
        }
    }
}
