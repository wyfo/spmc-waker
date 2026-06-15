//! A synchronization primitive for task wakeup.
//!
//! This crate provides [`SpmcWaker`], a single-producer, multiple-consumer (SPMC)
//! atomic waker.
//!
//! # Features
//!
//! - `portable-atomic`: use `portable-atomic` crate to provide functionality to
//!   targets without atomics.
#![warn(clippy::undocumented_unsafe_blocks)]
#![no_std]
#[cfg(doc)]
extern crate std;

use core::{
    hint::assert_unchecked,
    mem::ManuallyDrop,
    panic::{RefUnwindSafe, UnwindSafe},
    pin::Pin,
    ptr,
    sync::atomic::Ordering,
    task::{Context, Poll, RawWakerVTable, Waker},
};

use crate::{
    loom::{
        AtomicPtrExt,
        cell::UnsafeCell,
        hint::spin_loop,
        sync::atomic::{AtomicPtr, Ordering::*},
    },
    state_machine::{NULL, READ_FALLBACK, REGISTERED, WAKING, is_fallback},
    utils::{NOOP_PTR, TaggedPointerExt, UnsafeCellExt, WakerExt, guard},
};

#[cfg(all(debug_assertions, not(loom)))]
mod exclusive;
mod loom;
mod state_machine;
mod utils;

/// Truncate the `loom.trace` file (no-op unless the `LOOM_TRACE` env var is
/// set); call at the start of every loom model iteration so the trace reflects
/// only the failing interleaving.
#[cfg(loom)]
pub use crate::loom::trace::clear as clear_loom_trace;

/// A synchronization primitive for task wakeup.
///
/// `SpmcWaker` allows registering a task's [`Waker`] and waking the task atomically and
/// concurrently from multiple threads. A single instance may be reused for any number of
/// waker registration/call.
///
/// `SpmcWaker` should be paired with a wake condition, met **before** waking the
/// task, and checked **after** registering the task's waker to not miss a concurrent
/// notification that happened before.
///
/// `SpmcWaker` also provides a high-level async [`wait_until`](Self::wait_until), backed
/// by [`poll_wait_until`](Self::poll_wait_until), which is often more optimized than
/// manual registration through [`register`](Self::register).
///
/// Another optimization is to use [`wake_cold`](Self::wake_cold) instead of
/// [`wake`](Self::wake) in hot path, especially when a waker is rarely expected
/// to be registered, as it is more inlinable.
///
/// # Single-producer, multiple-consumer (SPMC)
///
/// `SpmcWaker` algorithm assumes a single thread registering a waker at a time.
/// It is enforced by the methods' safety condition.
///
/// This assumption allows significant optimizations compared to an MPMC algorithm
/// like [`AtomicWaker`].
///
/// # Synchronization
///
/// `SpmcWaker` has a generic `SYNC` parameter which determines the synchronization guarantees.
/// It impacts how the wake condition should be accessed.
///
/// ### `SYNC=true` (the default)
///
/// Calling `register` "acquires" all memory "released" by calls to `wake` before
/// the call to `register`. Later calls to `wake` will wake the registered waker.
///
/// As a consequence, if wake condition is atomic, it can accessed with [`Relaxed`] ordering.
///
/// ### `SYNC=false` (aliased to [`UnsynchronizedSpmcWaker`])
///
/// This is a more advanced configuration, where there is no acquire-release
/// synchronization between `register` and `wake`.
///
/// `register` and `wake` both use [`SeqCst`] ordering, and they rely on the `SeqCst`
/// pattern `store X; load Y || store Y; load X` to not miss any notification, where
/// X is the wake condition and Y the internal `SpmcWaker` state; `load Y` is `wake`
/// while `store Y` is `register`. That's why the wake condition should use `SeqCst`
/// for `store` and `load`.
///
/// It allows optimizing the algorithm even more, especially in the case where `wake`
/// is called with no waker registered, as it becomes a single atomic load (instead
/// of an atomic RMW operation for `SYNC=true`). In fact, `UnsynchronizedSpmcWaker` is
/// read-only as long as there is no waker registered. That makes it suitable to be
/// placed alongside other read-only data.
/// (As a side effect of a compiler optimization, `wake` with no waker registered
/// is also read-only on x86 platforms with `SYNC=true`, but not on aarch64)
///
/// `UnsynchronizedSpmcWaker` is particularly suited when the wake condition is already
/// updated through an atomic RMW operation. In that case, the cost of adding
/// `SeqCst` ordering to it is small compared to the significant gain of replacing
/// an atomic RMW operation by an atomic load in `wake`.
///
/// # Waker caching
///
/// Most of the time, `SpmcWaker` is used in a single task, so the waker
/// registered is always the same. That's why it provides a second generic
/// parameter `CACHED`.
///
/// ### `CACHED=true` (the default)
///
/// The last waker registered is kept cached to avoid cloning it on the next
/// registration. As a consequence, waking is done with [`Waker::wake_by_ref`].
/// As wakers are often `Arc`s, caching avoids atomic RMW operations updating
/// the reference counter.
///
/// ### `CACHED=false`
///
/// Waker is always cloned on registration, and the tasks are woken with
/// [`Waker::wake`].
///
/// # Progress guarantee
///
/// Waker registration is wait-free, while task waking is lock-free (without taking
/// in account waker clone/wake/drop operations).
///
/// When waker registration is only done through `try_register`, `wake` becomes
/// wait-free, but registration then requires spinning until it succeeds.
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
///         // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
///         unsafe { waker.poll_wait_until(cx, || self.0.notified.load(Relaxed)) }
///     }
/// }
///
/// fn event() -> (Notifier, Waiter) {
///     let waiter = Waiter::default();
///     (waiter.notifier(), waiter)
/// }
/// ```
///
/// An example with `SYNC=false` is presented in [`UnsynchronizedSpmcWaker`] documentation.
///
/// [`AtomicWaker`]: https://docs.rs/futures/latest/futures/task/struct.AtomicWaker.html
// The struct can contains two wakers: a "main" one and a "fallback" one, split into a vtable and
// a data fields. Main waker vtable pointer is tagged with the state machine state
#[derive(Debug)]
pub struct SpmcWaker<const SYNC: bool = true, const CACHED: bool = true> {
    vtable: AtomicPtr<RawWakerVTable>,
    data: UnsafeCell<*const ()>,
    fallback_data: AtomicPtr<()>,
    fallback_vtable: AtomicPtr<RawWakerVTable>,
    #[cfg(all(debug_assertions, not(loom)))]
    exclusive: exclusive::Exclusive,
}

// SAFETY: Cell accesses are synchronized through atomic vtable accesses
unsafe impl<const SYNC: bool, const CACHED: bool> Send for SpmcWaker<SYNC, CACHED> {}
// SAFETY: Cell accesses are synchronized through atomic vtable accesses
unsafe impl<const SYNC: bool, const CACHED: bool> Sync for SpmcWaker<SYNC, CACHED> {}
impl<const SYNC: bool, const CACHED: bool> UnwindSafe for SpmcWaker<SYNC, CACHED> {}
impl<const SYNC: bool, const CACHED: bool> RefUnwindSafe for SpmcWaker<SYNC, CACHED> {}

impl<const SYNC: bool, const CACHED: bool> Drop for SpmcWaker<SYNC, CACHED> {
    #[inline]
    fn drop(&mut self) {
        let vtable = self.vtable.load_mut();
        if CACHED || vtable.has(REGISTERED) {
            // SAFETY: there is a waker registered or cached, with no concurrent access
            // as per mutable reference
            drop(unsafe { self.waker(vtable.unset(REGISTERED)) });
        }
    }
}

impl<const SYNC: bool, const CACHED: bool> SpmcWaker<SYNC, CACHED> {
    /// Creates a new `SpmcWaker`.
    #[cfg_attr(loom, const_fn::const_fn(cfg(false)))]
    #[inline]
    pub const fn new() -> Self {
        Self {
            vtable: AtomicPtr::new(NOOP_PTR),
            data: UnsafeCell::new(ptr::null()),
            fallback_vtable: AtomicPtr::new(ptr::null_mut()),
            fallback_data: AtomicPtr::new(ptr::null_mut()),
            #[cfg(all(debug_assertions, not(loom)))]
            exclusive: exclusive::Exclusive::new(),
        }
    }

    /// # Safety
    ///
    /// `vtable` must point to a valid `&'static RawWakerVTable`.
    /// Waker data must be safe to read, i.e., no concurrent write shall happen.
    #[inline(always)]
    unsafe fn waker(&self, vtable: *mut RawWakerVTable) -> Waker {
        debug_assert!(!vtable.has(REGISTERED | WAKING));
        // SAFETY: as per function contract + data is always set together with vtable
        // so they form a valid waker.
        unsafe { Waker::new(self.data.read(), &*vtable) }
    }

    /// # Safety
    ///
    /// Fallback waker can be read at any moment, but must only be used
    /// after acquiring its ownership
    #[inline(always)]
    unsafe fn fallback_waker(&self) -> Waker {
        let data = self.fallback_data.load(Relaxed);
        let vtable = self.fallback_vtable.load(Relaxed);
        // SAFETY: as per function contract
        unsafe { Waker::new(data, &*vtable) }
    }

    /// Wait until the given wake condition is met.
    ///
    /// Notifier threads should call [`wake`](Self::wake) (or [`wake_cold`](Self::wake))
    /// after wake condition is met.
    ///
    /// # Safety
    ///
    /// Polling the returned future calls [`poll_wail_until`](Self::poll_wait_until) and inherits
    /// its safety condition. Basically, only a single thread should await a wake condition at a
    /// time.
    #[inline]
    pub unsafe fn wait_until<C: FnMut() -> bool>(
        &self,
        wake_condition: C,
    ) -> WaitUntil<'_, C, SYNC, CACHED> {
        WaitUntil {
            wake: self,
            wake_condition,
        }
    }

    /// Returns `Poll::Ready` if the wake condition is met, or registers
    /// the task's waker to be notified.
    ///
    /// Notifier threads should call [`wake`](Self::wake) (or [`wake_cold`](Self::wake))
    /// after wake condition is met.
    ///
    /// It is equivalent to the following code (but more optimized):
    /// ```ignore
    /// // quick check to avoid registration if the wake condition is already met.
    /// if wake_condition() {
    ///     return Poll::Ready(());
    /// }
    /// // try registering the waker
    /// if !self.try_register(cx.waker()) {
    ///     // a concurrent wake is ongoing, the wake condition should be met
    ///     if wake_condition() {
    ///         return Poll::Ready(());
    ///     }
    ///     // a previous wake didn't terminate, pause before retrying
    ///     spin_loop();
    ///     // force waker registration
    ///     self.register(cx.waker());
    /// }
    /// // check the wake condition **after** registering the waker
    /// // to not miss any notification
    /// if self.wake_condition() {
    ///     // unregister the waker to avoid spurious wakeups
    ///     self.unregister();
    ///     return Poll::Ready(());
    /// }
    /// Poll::Pending
    /// ```
    ///
    /// # Safety
    ///
    /// All waker registration methods must not be called concurrently with each other from
    /// multiple threads.
    #[inline]
    pub unsafe fn poll_wait_until<P: FnMut() -> bool>(
        &self,
        cx: &mut Context,
        mut wake_condition: P,
    ) -> Poll<()> {
        #[cfg(all(debug_assertions, not(loom)))]
        let _guard = self.exclusive.check();
        // Quick check to avoid registration if the wake condition is already met.
        if wake_condition() {
            return Poll::Ready(());
        }
        // Try registering the waker with fast path.
        match self.register_inlined(cx.waker()) {
            // Check the wake condition **after** registering the waker
            // to not miss any notification.
            Ok(vtable) if wake_condition() => {
                // Unregister the waker to avoid spurious wakeups.
                self.unregister_inlined(vtable);
                Poll::Ready(())
            }
            Ok(_) => Poll::Pending,
            Err(vtable) => self.poll_wait_until_cold(cx.waker(), vtable, wake_condition),
        }
    }

    #[cold]
    fn poll_wait_until_cold<P: FnMut() -> bool>(
        &self,
        waker: &Waker,
        vtable: *mut RawWakerVTable,
        mut wake_condition: P,
    ) -> Poll<()> {
        // Try registering the waker.
        if !self.register_cold(waker, vtable, false, true) {
            // A concurrent wake is ongoing, the wake condition should be met.
            if wake_condition() {
                return Poll::Ready(());
            }
            // A previous wake didn't terminate, pause before retrying.
            spin_loop();
            // Force waker registration.
            self.register_cold(waker, vtable, true, true);
        }
        // Check the wake condition **after** registering the waker
        // to not miss any notification.
        if wake_condition() {
            // Unregister the waker to avoid spurious wakeups.
            if let Some(vtable) = self.unregister_vtable() {
                self.unregister_inlined(vtable);
            }
            return Poll::Ready(());
        }
        Poll::Pending
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
    /// task being notified once.
    ///
    /// # Safety
    ///
    /// All waker registration methods must not be called concurrently with each other from
    /// multiple threads.
    #[inline]
    pub unsafe fn register(&self, waker: &Waker) {
        #[cfg(all(debug_assertions, not(loom)))]
        let _guard = self.exclusive.check();
        if let Err(vtable) = self.register_inlined(waker) {
            self.register_cold(waker, vtable, true, true);
        }
    }

    /// Tries registering the waker, fails if a concurrent `wake` is currently executing.
    ///
    /// See [`register`](Self::register). Returns `true` if registration succeeded.
    ///
    /// When `wake` is executed concurrently, there is a great chance that the wake
    /// condition is met. In that case, forcing the registration of the waker like
    /// `register` does can be counter-productive. `register` should however be
    /// called if the wake condition is still not met — is it also possible
    /// to spin a bit to wait for `wake` to finish.
    ///
    /// # Safety
    ///
    /// All waker registration methods must not be called concurrently with each other from
    /// multiple threads.
    #[inline]
    pub unsafe fn try_register(&self, waker: &Waker) -> bool {
        #[cfg(all(debug_assertions, not(loom)))]
        let _guard = self.exclusive.check();
        match self.register_inlined(waker) {
            Ok(_) => true,
            Err(vtable) => self.register_cold(waker, vtable, false, true),
        }
    }

    #[inline(always)]
    fn register_inlined(&self, waker: &Waker) -> Result<*mut RawWakerVTable, *mut RawWakerVTable> {
        // Load vtable with Acquire ordering to synchronize data access with `wake`.
        // For SYNC=true, it also ensures the synchronization in case of early
        // return; otherwise, synchronization is done through the RMW setting
        // the vtable.
        let vtable = self.vtable.load(Acquire);
        if !CACHED && !vtable.has(REGISTERED | WAKING) {
            Ok(self.register_vtable(&ManuallyDrop::new(waker.clone()), true))
        // No need to check `is_empty(vtable)` as it is implied by vtable equality.
        } else if CACHED
            && waker.vtable_ptr() == vtable
            // SAFETY: vtable is unregistered, so its Acquire load cannot be stale
            // and synchronize with previous accesses
            && waker.data() == unsafe { self.data.read() }
        {
            Ok(self.register_vtable(waker, false))
        } else {
            Err(vtable)
        }
    }

    #[inline(always)]
    fn register_vtable(&self, waker: &Waker, set_data: bool) -> *mut RawWakerVTable {
        if set_data {
            // SAFETY: vtable is unregistered, so its Acquire load cannot be stale
            // and synchronize with previous accesses
            unsafe { self.data.write(waker.data()) };
        }
        let registered = waker.vtable_ptr().set(REGISTERED);
        // TRANSITION: V -> V|R
        if SYNC {
            // Acquire ordering is necessary to synchronize with `wake`, so swap
            // must be used. Release is necessary if waker data has been set.
            // Otherwise, the swap write can be relaxed: a `wake` claiming this
            // registration still acquires the data through the release sequence
            // of the previous registration of the same waker, as every vtable
            // updates are RMWs.
            let ordering = if set_data { AcqRel } else { Acquire };
            self.vtable.swap(registered, ordering);
        } else {
            // Storing the vtable with SeqCst is necessary for the pattern
            // `store X; load Y || store Y; load X` to not miss any
            // notification, where X is the wake condition and Y the vtable.
            self.vtable.store(registered, SeqCst);
        }
        registered
    }

    // Overwriting a registered/cached waker is expected to be rare, hence the `#[cold]` attribute.
    #[cold]
    fn register_cold(
        &self,
        waker: &Waker,
        vtable: *mut RawWakerVTable,
        force: bool,
        clone: bool,
    ) -> bool {
        // If the waker is already registered, there is no need to replace it,
        // and there is no `wake` to synchronize with: either the loaded vtable
        // is up to date, and a preceding `wake` would have consumed the
        // registration; or it is stale and a concurrent `wake` is consuming it,
        // but its pending `wake_by_ref` call targets this same waker, so the
        // task will be polled again — and that poll happens after the wake's
        // claim, so it cannot load the stale registration a second time.
        if clone
            && waker.vtable_ptr().set(REGISTERED) == vtable
            // SAFETY: vtable is registered, so its Acquire load cannot be stale
            // and synchronize with previous accesses
            && waker.data() == unsafe { self.data.read() }
        {
            return true;
        }
        // A concurrent `wake` may be happening.
        if vtable.has(WAKING) {
            return force && self.register_fallback(vtable, waker, clone);
        }
        let mut _waker_to_drop = None;
        // Unregister the registered waker if there is one.
        if vtable.has(REGISTERED) {
            if is_fallback(vtable) {
                return self.register_fallback(vtable, waker, clone);
            }
            // If CAS succeeds, then there is nothing to synchronize with, SYNC=true being
            // handled in `register_vtable`. On the other hand, CAS failure means a `wake`
            // in between and requires Acquire ordering to synchronize data access.
            // TRANSITION: V|R -> V
            match (self.vtable).compare_exchange(vtable, NOOP_PTR, Relaxed, Acquire) {
                // Unregistered waker must be dropped.
                // SAFETY: waker data access was already synchronized
                Ok(_) => _waker_to_drop = Some(unsafe { self.waker(vtable.unset(REGISTERED)) }),
                // A concurrent `wake` happened, return if registration is not forced,
                // as wake condition is surely met.
                Err(_) if !force => return false,
                // Otherwise, if `wake` is still ongoing, register a fallback waker.
                Err(vtable) if vtable.has(WAKING) => {
                    return self.register_fallback(vtable, waker, clone);
                }
                // `wake` is done, but the waker must still be overwritten, so the cached waker
                // must be dropped. It's not possible to reuse it as the first check of the
                // function would have return true in that case.
                // SAFETY: No concurrent `register` can happen, and there is no fallback waker
                // registered, so `wake` can't modify data
                Err(vtable) if CACHED => _waker_to_drop = Some(unsafe { self.waker(vtable) }),
                Err(vtable) => {
                    debug_assert!(!is_fallback(vtable) && !vtable.has(REGISTERED | WAKING));
                }
            }
        // Cached waker will be overridden and must be dropped
        } else if CACHED {
            // SAFETY: No concurrent `register` can happen, and there is no fallback waker
            // registered, so `wake` can't modify data
            _waker_to_drop = Some(unsafe { self.waker(vtable) });
        };
        let waker_clone;
        let waker = if clone {
            waker_clone = ManuallyDrop::new(waker.clone());
            &waker_clone
        } else {
            waker
        };
        // Register the waker clone.
        self.register_vtable(waker, true);
        true
    }

    fn register_fallback(
        &self,
        mut vtable: *mut RawWakerVTable,
        waker: &Waker,
        clone: bool,
    ) -> bool {
        let waker_clone;
        let waker = if clone {
            waker_clone = ManuallyDrop::new(waker.clone());
            &waker_clone
        } else {
            waker
        };
        let mut _waker_to_drop = None;
        // If there already is a fallback waker registered, vtable must be reset to NULL
        // to reset it and prevent the current one to be concurrently read.
        if is_fallback(vtable) {
            // This loop is not unbounded as per the state machine
            // bounded number of transitions until exit condition.
            loop {
                debug_assert!(vtable.has(REGISTERED) || vtable.has(REGISTERED | WAKING));
                // Relaxed is fine for success, as there is nothing to synchronize with.
                // Failure requires Acquire to synchronize waker data access before
                // overwriting it.
                // TRANSITION: N|R -> N / N|R|W -> N / RF|R -> N / RF|R|W -> N
                match (self.vtable).compare_exchange(vtable, NULL, Relaxed, Acquire) {
                    Ok(_) => break,
                    Err(v) if is_fallback(v) => vtable = v,
                    // CAS failed with a valid vtable pointer, so no fallback waker
                    // is registered passed this point. The main waker can be overwritten.
                    // The recursion is finite, because `register` is wait-free.
                    Err(v) => return self.register_cold(waker, v, true, false),
                }
            }
            vtable = NULL;
            // SAFETY: vtable has been reset to NULL, so fallback waker will not be
            // used by waking thread and can be safely dropped here
            _waker_to_drop = Some(unsafe { self.fallback_waker() });
        }
        debug_assert!((!is_fallback(vtable) && vtable.has(REGISTERED | WAKING)) || vtable == NULL);
        self.fallback_data.store(waker.data().cast_mut(), Relaxed);
        (self.fallback_vtable).store(waker.vtable_ptr(), Relaxed);
        // Register the fallback waker, using the same ordering as `register_vtable`.
        let ordering = if SYNC { AcqRel } else { SeqCst };
        // TRANSITION: V/R/W -> N|R / N -> N|R
        if let Err(vtable) =
            (self.vtable).compare_exchange(vtable, NULL.set(REGISTERED), ordering, Acquire)
        {
            debug_assert!(!is_fallback(vtable) && !vtable.has(REGISTERED | WAKING));
            // Registration fails, so main waker must be unregistered and will be
            // overwritten. There is no point to reuse cached waker as the waker
            // in argument has already been cloned.
            // The recursion is finite, because `register` is wait-free.
            return self.register_cold(waker, vtable, true, false);
        }
        true
    }

    /// Removes the registered waker if there is one, returning `true` in this case.
    ///
    /// It allows avoiding spurious wakeups when a waker has been registered,
    /// but the wake condition is already met.
    ///
    /// # Safety
    ///
    /// All waker registration methods must not be called concurrently with each other from
    /// multiple threads.
    #[inline]
    pub unsafe fn unregister(&self) -> bool {
        #[cfg(all(debug_assertions, not(loom)))]
        let _guard = self.exclusive.check();
        self.unregister_vtable()
            .is_some_and(|vtable| self.unregister_inlined(vtable))
    }

    fn unregister_vtable(&self) -> Option<*mut RawWakerVTable> {
        let vtable = self.vtable.load(Relaxed);
        (vtable.has(REGISTERED) && !vtable.has(WAKING) && !is_fallback(vtable)).then_some(vtable)
    }

    #[inline(always)]
    fn unregister_inlined(&self, vtable: *mut RawWakerVTable) -> bool {
        debug_assert!(vtable.has(REGISTERED) && !vtable.has(WAKING) && !is_fallback(vtable));
        let unregistered = vtable.unset(REGISTERED);
        // Relaxed/Acquire order is ok here, as `unregister` and `register` are called in
        // the same thread, i.e. sequenced-before, so there is no risk that this CAS make
        // possible a stale load of an unregistered vtable instead of a registered one.
        // It may provoke a stale load of registered vtable, but `wake` deals with it.
        // As `wake_fallback` may modify waker data, CACHED=false requires Acquire
        // ordering to drop the waker
        let ordering = if CACHED { Relaxed } else { Acquire };
        // TRANSITION: V|R -> V
        match (self.vtable).compare_exchange(vtable, unregistered, ordering, Relaxed) {
            Ok(_) if !CACHED => {
                // SAFETY: No concurrent `register` can happen, and there is no fallback waker
                // registered, so `wake` can't modify data
                drop(unsafe { self.waker(unregistered) });
                true
            }
            res => res.is_ok(),
        }
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
        self.has_waker_registered_impl().is_some()
    }

    #[inline(always)]
    fn has_waker_registered_impl(&self) -> Option<(*mut RawWakerVTable, bool)> {
        if SYNC {
            // SYNC=true requires a Release write on the state, but we don't want to set
            // the WAKING bit if there is no waker, as it would require unsetting it.
            // So we attempt a `fetch_add(0)` and hope for no concurrent `register`.
            // The next `register` synchronizes with this Release write even if other
            // RMWs intervene, through the release sequence it heads.
            let vtable = self.vtable.load(Relaxed);
            if vtable.has(REGISTERED) {
                return Some((vtable, false));
            }
            let vtable = self.vtable.fetch_byte_add(0, Release);
            if vtable.has(REGISTERED) {
                return Some((vtable, true));
            }
            None
        } else {
            // Loading the vtable with SeqCst is necessary for the pattern
            // `store X; load Y || store Y; load X` to not miss any
            // notification, where X is the wake condition and Y the vtable.
            let vtable = self.vtable.load(SeqCst);
            vtable.has(REGISTERED).then_some((vtable, false))
        }
    }

    /// Consumes the last `Waker` registered and wake its task.
    ///
    /// If `register` has not been called yet, then this does nothing.
    #[inline]
    pub fn wake(&self) {
        self.check_registered_and_wake(false, Self::wake_waker);
    }

    /// Same as [`wake`](Self::wake), but with the waking path marked `#[cold]`.
    ///
    /// This allows the method to inline more effectively. Prefer this over
    /// `wake` when waking is the uncommon case.
    #[inline]
    pub fn wake_cold(&self) {
        self.check_registered_and_wake(true, Self::wake_waker);
    }

    #[inline(always)]
    fn wake_waker(waker: Waker) {
        if CACHED {
            ManuallyDrop::new(waker).wake_by_ref();
        } else {
            waker.wake();
        }
    }

    #[inline(always)]
    fn check_registered_and_wake(&self, cold: bool, wake: impl FnMut(Waker)) {
        if let Some((vtable, released)) = self.has_waker_registered_impl() {
            if cold {
                self.wake_registered_cold(vtable, released, wake);
            } else {
                self.wake_registered(vtable, released, wake);
            }
        }
    }

    #[cfg(not(miri))]
    const RELEASE: Ordering = Release;
    #[cfg(miri)] // See https://github.com/rust-lang/miri/issues/5104
    const RELEASE: Ordering = if SYNC { Release } else { SeqCst }; // !ORDERING

    #[inline(always)]
    fn wake_registered(
        &self,
        mut vtable: *mut RawWakerVTable,
        mut released: bool,
        mut wake: impl FnMut(Waker),
    ) {
        // A waker is registered, try setting the WAKING flag.
        loop {
            if !vtable.has(WAKING) {
                // Acquire ordering is necessary to synchronize data access.
                // SYNC=true also needs Release to synchronize with `wake`.
                // It could be tempting to defer the Release to the last CAS,
                // but it doesn't work when a fallback waker is registered.
                // In fact, `wake` would happen before the registration, but
                // without synchronizing the wake condition.
                let ordering = if SYNC { AcqRel } else { Acquire };
                // TRANSITION: V|R -> V|R|W / N|R -> N|R|W / RF|R -> RF|R|W
                match (self.vtable).compare_exchange(vtable, vtable.set(WAKING), ordering, Relaxed)
                {
                    Ok(_) => break,
                    // Waking thread can update the vtable without consuming the waker
                    // when a fallback waker is registered, so the operation should be retried.
                    Err(v) if is_fallback(vtable) && v.has(REGISTERED) => {
                        vtable = v;
                        continue;
                    }
                    Err(_) => {}
                };
            }
            // Do the Release RMW if not done yet for SYNC=true and retry.
            if SYNC && !released {
                vtable = self.vtable.fetch_byte_add(0, Release);
                released = true;
                if vtable.has(REGISTERED) {
                    continue;
                }
            }
            // Setting the flag fails because the waker has already been consumed.
            // There is no interest in retrying, as the wake condition should already
            // been met, so it will be checked successfully after registering the
            // next waker.
            return;
        }
        // The fallback waker has been notified for the waking thread to call it, return.
        if is_fallback(vtable) {
            return;
        }
        // SAFETY: it has been tested before, but this assert helps compiler
        unsafe { assert_unchecked(vtable.has(REGISTERED)) };
        let unregistered = vtable.unset(REGISTERED);
        // SAFETY: The vtable WAKING flag has been set, so no waker can be registered.
        let waker = unsafe { self.waker(unregistered) };
        // With CACHED=true, as `wake_by_ref` uses a reference, the waker must remain
        // valid so it must be executed before resetting the state.
        let mut waker = ManuallyDrop::new(waker);
        if CACHED {
            guard(
                // SAFETY: with CACHED=true, wake function wraps the waker into
                // a ManuallyDrop, so it will not be duplicated.
                || wake(unsafe { ManuallyDrop::take(&mut waker) }),
                // If a new waker is registered in fallback, it will be leaked.
                // Leaking is better than risking an abort because of double panic.
                || self.vtable.swap(unregistered, SeqCst), // !ORDERING
            );
        }
        // Try resetting the vtable to unregistered. Use release ordering to
        // synchronize data access. Not using SeqCst with SYNC=false doesn't
        // break the lost-wakeup guarantee. Imagine the deadlock scenario:
        // the registering thread checks the wakeup condition after registration
        // and finds it unset, while this stale store is loaded by a `wake`
        // called after the condition was set. The unset condition load is
        // coherence-ordered before the condition store (it reads a value the
        // store overwrites), so with thread sequencing, the registration precedes
        // the stale vtable load in the SeqCst total order. But loading a vtable
        // older in modification order than the registration would coherence-order
        // the load *before* the registration — contradiction
        // TRANSITION: V|R|W -> V
        if let Err(vtable) =
            (self.vtable).compare_exchange(vtable.set(WAKING), unregistered, Self::RELEASE, Relaxed)
        {
            // Failing to reset the state means a concurrent `register` forced
            // a fallback waker registration.
            self.wake_fallback(vtable, unregistered, waker, wake);
        } else if !CACHED {
            wake(ManuallyDrop::into_inner(waker));
        }
    }

    #[cold]
    fn wake_fallback(
        &self,
        mut vtable: *mut RawWakerVTable,
        unregistered: *mut RawWakerVTable,
        waker: ManuallyDrop<Waker>,
        mut wake: impl FnMut(Waker),
    ) {
        // A fallback waker has been registered, and may be registered again or waken concurrently.
        loop {
            // `register` tries acquiring the fallback waker to overwrite it.
            // We don't have to wait for it to complete and can just reset the vtable,
            // as `register` will handle it and store a new waker normally.
            if vtable == NULL {
                // TRANSITION: N -> V
                if let Err(v) =
                    (self.vtable).compare_exchange(vtable, unregistered, Self::RELEASE, Relaxed)
                {
                    vtable = v;
                    continue;
                }
                // There is no point to wake the initial waker as register has been called
                // since; just drop it.
                if !CACHED {
                    drop(ManuallyDrop::into_inner(waker));
                }

            // A fallback waker has been registered, it should replace the main waker.
            } else if vtable == NULL.set(REGISTERED) {
                let read_fallback = READ_FALLBACK.set(REGISTERED);
                // First update the vtable to read the fallback waker
                // TRANSITION: N|R -> RF|R
                if let Err(v) =
                    (self.vtable).compare_exchange(vtable, read_fallback, Acquire, Relaxed)
                {
                    vtable = v;
                    continue;
                }
                // Try replacing the main waker by the fallback one.
                // SAFETY: Waker data is no longer accessed by registering thread after WAKING
                // flag has been set
                #[allow(unstable_name_collisions)]
                let data = unsafe { self.data.replace(self.fallback_data.load(Relaxed)) };
                let registered = self.fallback_vtable.load(Relaxed).set(REGISTERED);
                // TRANSITION: RF|R -> V|R
                if let Err(v) =
                    (self.vtable).compare_exchange(read_fallback, registered, Release, Relaxed)
                {
                    vtable = v;
                    // Don't forget to restore the initial data if replacement fails.
                    // SAFETY: Waker data is no longer accessed by registering thread after WAKING
                    // flag has been set
                    unsafe { self.data.write(data) };
                    continue;
                }
                drop(ManuallyDrop::into_inner(waker));
            // A registered fallback waker has been notified by another thread.
            } else {
                debug_assert!(vtable.has(REGISTERED) && vtable.has(WAKING));
                let read_fallback = READ_FALLBACK.set(REGISTERED | WAKING);
                // The vtable must be updated before reading the fallback waker.
                if vtable != read_fallback
                    // TRANSITION: N|R|W -> RF|R|W
                    && let Err(v) =
                        (self.vtable).compare_exchange(vtable, read_fallback, Acquire, Relaxed)
                {
                    vtable = v;
                    continue;
                }
                // SAFETY: fallback waker is used after resetting the vtable to unregistered
                // so they can't have been concurrent modification of fallback waker as
                // register would first try to reset the vtable to NULL.
                let mut fallback_waker = ManuallyDrop::new(unsafe { self.fallback_waker() });
                // Now reset the vtable to unregistered to wake the fallback waker.
                // TRANSITION: RF|R|W -> V
                if let Err(v) = (self.vtable).compare_exchange(
                    read_fallback,
                    unregistered,
                    Self::RELEASE,
                    Relaxed,
                ) {
                    vtable = v;
                    continue;
                }
                // SAFETY: with CACHED=true, wake function wraps the waker into
                // a ManuallyDrop, so it will not be duplicated.
                wake(unsafe { ManuallyDrop::take(&mut fallback_waker) });
                if CACHED {
                    drop(ManuallyDrop::into_inner(fallback_waker));
                } else {
                    drop(ManuallyDrop::into_inner(waker));
                }
            }
            return;
        }
    }

    #[cold]
    #[inline(never)]
    fn wake_registered_cold(
        &self,
        vtable: *mut RawWakerVTable,
        released: bool,
        wake: impl FnMut(Waker),
    ) {
        self.wake_registered(vtable, released, wake);
    }
}

impl<const SYNC: bool> SpmcWaker<SYNC, true> {
    /// Applies the given function to the last `Waker` passed to `register`.
    ///
    /// If a waker has not been registered, this returns `None`.
    #[inline]
    pub fn wake_with<W: FnMut(&Waker)>(&self, mut wake: W) {
        self.check_registered_and_wake(false, move |w| wake(&ManuallyDrop::new(w)));
    }

    /// Same as [`wake_with`](Self::wake_with), but with the waking path marked `#[cold]`.
    ///
    /// This allows the method to inline more effectively. Prefer this over
    /// `wake_with` when taking is the uncommon case.
    #[inline]
    pub fn wake_with_cold<W: FnMut(&Waker)>(&self, mut wake: W) {
        self.check_registered_and_wake(true, move |w| wake(&ManuallyDrop::new(w)));
    }
}

impl<const SYNC: bool> SpmcWaker<SYNC, false> {
    /// Applies the given function to the last `Waker` passed to `register`.
    ///
    /// If a waker has not been registered, this returns `None`.
    #[inline]
    pub fn wake_with<W: FnMut(Waker)>(&self, wake: W) {
        self.check_registered_and_wake(false, wake);
    }

    /// Same as [`wake_with`](Self::wake_with), but with the waking path marked `#[cold]`.
    ///
    /// This allows the method to inline more effectively. Prefer this over
    /// `wake_with` when taking is the uncommon case.
    #[inline]
    pub fn wake_with_cold<W: FnMut(Waker)>(&self, wake: W) {
        self.check_registered_and_wake(true, wake);
    }
}

impl<const SYNC: bool, const CACHED: bool> Default for SpmcWaker<SYNC, CACHED> {
    fn default() -> Self {
        Self::new()
    }
}

/// Advanced configuration of [`SpmcWaker`] providing no synchronization
/// between `register` and `wake`.
///
/// It should be paired with [`SeqCst`] ordering on the wakeup condition.
/// See [Synchronization section](SpmcWaker#syncfalse-aliased-to-unsynchronizedspmcwaker)
/// for more details.
///
/// # Examples
///
/// Here is the `SpmcWaker` example rewritten for `UnsynchronizedSpmcWaker` using `SeqCst` ordering:
///
/// ```rust
/// use std::{
///     pin::Pin,
///     sync::{
///         Arc,
///         atomic::{
///             AtomicBool,
///             Ordering::{Relaxed, SeqCst},
///         },
///     },
///     task::{Context, Poll},
/// };
///
/// use spmc_waker::UnsynchronizedSpmcWaker;
///
/// #[derive(Default)]
/// struct Inner {
///     notified: AtomicBool,
///     waker: UnsynchronizedSpmcWaker,
/// }
///
/// #[derive(Clone)]
/// struct Notifier(Arc<Inner>);
///
/// impl Notifier {
///     pub fn new() -> Self {
///         Self(Arc::new(Inner {
///             waker: UnsynchronizedSpmcWaker::new(),
///             notified: AtomicBool::new(false),
///         }))
///     }
///
///     pub fn signal(&self) {
///         // `UnsynchronizedSpmcWaker` requires `SeqCst` ordering.
///         self.0.notified.store(true, SeqCst);
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
///         // `UnsynchronizedSpmcWaker` requires `SeqCst` ordering.
///         // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
///         unsafe { waker.poll_wait_until(cx, || self.0.notified.load(SeqCst)) }
///     }
/// }
///
/// fn event() -> (Notifier, Waiter) {
///     let waiter = Waiter::default();
///     (waiter.notifier(), waiter)
/// }
/// ```
pub type UnsynchronizedSpmcWaker<const CACHED: bool = true> = SpmcWaker<false, CACHED>;

pub struct WaitUntil<'a, C, const SYNC: bool, const CACHED: bool> {
    wake: &'a SpmcWaker<SYNC, CACHED>,
    wake_condition: C,
}

impl<C, const SYNC: bool, const CACHED: bool> Unpin for WaitUntil<'_, C, SYNC, CACHED> {}

impl<C: FnMut() -> bool, const SYNC: bool, const CACHED: bool> Future
    for WaitUntil<'_, C, SYNC, CACHED>
{
    type Output = ();
    #[inline(always)]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: as per `SpmcWaker::wait_until` safety contract
        unsafe { self.wake.poll_wait_until(cx, || (self.wake_condition)()) }
    }
}
