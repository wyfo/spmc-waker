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
    convert::identity,
    mem::ManuallyDrop,
    panic::{RefUnwindSafe, UnwindSafe},
    ptr,
    task::{RawWaker, RawWakerVTable, Waker},
};

use crate::loom::{
    AtomicPtrExt,
    cell::Cell,
    sync::atomic::{AtomicPtr, Ordering::*, fence},
};

#[cfg(all(debug_assertions, not(loom)))]
mod exclusive;
mod loom;

const _: () = assert!(core::mem::align_of::<RawWakerVTable>() >= 4);
const REGISTERED: usize = 1;
const WAKING: usize = 2;
#[inline(always)]
fn is_registered(vtable: *mut RawWakerVTable) -> bool {
    vtable.addr() & (REGISTERED | WAKING) == REGISTERED
}
#[inline(always)]
fn unregistered(vtable: *mut RawWakerVTable) -> *mut RawWakerVTable {
    vtable.map_addr(|addr| addr & !REGISTERED)
}
fn check_concurrent_wake(vtable: *mut RawWakerVTable) {
    debug_assert!(vtable.addr() & WAKING != 0 || vtable.addr() & (REGISTERED | WAKING) == 0);
}

static NOOP_VTABLE: &RawWakerVTable = &RawWakerVTable::new(
    |_| RawWaker::new(ptr::null(), NOOP_VTABLE),
    |_| (),
    |_| (),
    |_| (),
);

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
/// # Synchronization
///
/// `SpmcWaker` has a generic `SYNC` parameter which determines the
/// synchronization guarantees.
///
/// ### `SYNC=true` (the default)
///
/// Calling `register` "acquires" all memory "released" by calls to `wake`
/// before the call to `register`. Later calls to `wake` will wake the
/// registered waker.
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
/// of an atomic RMW operation for `SYNC=true`). In fact, `UnsyncSpmcWaker` is
/// read-only as long as there is no waker registered. That makes it suitable to be
/// placed alongside other read-only data.
/// (As a side effect of a compiler optimization, `wake` with no waker registered
/// is also read-only on x86 platforms with `SYNC=true`, but not on aarch64)
///
/// `UnsyncSpmcWaker` is particularly suited when the wake condition is already
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
/// The last waker registered is kept cached to avoid cloning it at the next
/// registration. As a consequence, waking is done with [`Waker::wake_by_ref`].
/// As wakers are often `Arc`s, caching avoids atomic RMW operations updating
/// the reference counter.
///
/// ### `CACHED=false`
///
/// Waker is always cloned on `register`, and the tasks are woken with
/// [`Waker::wake`].
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
///         // quick check to avoid registration if already done.
///         if self.0.notified.load(Relaxed) {
///             return Poll::Ready(());
///         }
///
///         // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
///         unsafe { self.0.waker.register(cx.waker()) };
///
///         // Need to check condition **after** `register` to avoid a race
///         // condition that would result in lost notifications.
///         if self.0.notified.load(Relaxed) {
///             // Unregister the waker to avoid spurious wakeups.
///             // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
///             unsafe { self.0.waker.unregister() };
///             Poll::Ready(())
///         } else {
///             Poll::Pending
///         }
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
#[derive(Debug)]
pub struct SpmcWaker<const SYNC: bool = true, const CACHED: bool = true> {
    vtable: AtomicPtr<RawWakerVTable>,
    data: Cell<*const ()>,
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
        if CACHED || vtable.addr() & REGISTERED != 0 {
            // SAFETY: there is a waker registered or cached, with no concurrent access
            // as per mutable reference
            drop(unsafe { self.waker(vtable.map_addr(|addr| addr & !REGISTERED)) });
        }
    }
}

impl<const SYNC: bool, const CACHED: bool> SpmcWaker<SYNC, CACHED> {
    /// Creates a new `SpmcWaker`.
    #[cfg_attr(loom, const_fn::const_fn(cfg(false)))]
    #[inline]
    pub const fn new() -> Self {
        Self {
            vtable: AtomicPtr::new(ptr::from_ref(NOOP_VTABLE).cast_mut()),
            data: Cell::new(ptr::null()),
            #[cfg(all(debug_assertions, not(loom)))]
            exclusive: exclusive::Exclusive::new(),
        }
    }

    /// # Safety
    ///
    /// `vtable` must point to a valid `&'static RawWakerVTable`.
    /// Waker data must be safe to read, i.e., no concurrent write shall happen.
    #[inline(always)]
    unsafe fn waker(&self, vtable: *const RawWakerVTable) -> Waker {
        // A debug_assert here is better than a segfault without stacktrace in test.
        debug_assert!(vtable.addr() & (REGISTERED | WAKING) == 0);
        // SAFETY: as per function contract + data is always set together with vtable
        // so they form a valid waker.
        unsafe { Waker::new(self.data.get(), &*vtable) }
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
    /// another opportunity to register its waker — this would be equivalent
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
        // Modifying the waker data requires an Acquire operation pairing
        // with the Release reset in `wake`, which is sequenced after its
        // read of the data. Data will surely be modified with CACHED=false
        // (except if the waker is already registered), so the acquire is
        // folded into the load. With CACHED=true, data is only modified
        // in `overwrite`, which emits its own fence.
        // Synchronization required by SYNC=true will be carried by
        // the RMW used to set the vtable.
        let ordering = if CACHED { Relaxed } else { Acquire };
        let mut vtable = self.vtable.load(ordering);
        if !CACHED && vtable.addr() & (REGISTERED | WAKING) == 0 {
            let waker = ManuallyDrop::new(waker.clone());
            self.data.set(waker.data());
            vtable = ptr::from_ref(waker.vtable()).cast_mut();
            self.register_vtable(vtable, true)
        } else if CACHED
            // No need to check vtable.addr() & (REGISTERED | WAKING) == 0
            // as it is implied by vtable equality.
            && ptr::from_ref(waker.vtable()) == vtable
            && waker.data() == self.data.get()
        {
            self.register_vtable(vtable, false)
        } else {
            self.overwrite(waker, vtable)
        }
    }

    #[inline(always)]
    fn register_vtable(&self, vtable: *const RawWakerVTable, data_set: bool) -> bool {
        let registered = vtable.map_addr(|addr| addr | REGISTERED).cast_mut();
        if SYNC {
            // Acquire ordering is necessary to synchronize with `wake`, so swap
            // must be used. Release is necessary if waker data has been set.
            // Otherwise, the swap write can be relaxed: a `wake` claiming this
            // registration still acquires the data through the release sequence
            // of the previous registration of the same waker, as every vtable
            // updates are RMWs.
            let ordering = if data_set { AcqRel } else { Acquire };
            self.vtable.swap(registered, ordering);
        } else {
            // Storing the vtable with SeqCst is necessary for the pattern
            // `store X; load Y || store Y; load X` to not miss any
            // notification, where X is the wake condition and Y the vtable.
            self.vtable.store(registered, SeqCst);
        }
        true
    }

    // Overwriting a registered waker is expected to be rare, hence the `#[cold]` attribute.
    #[cold]
    fn overwrite(&self, waker: &Waker, vtable: *mut RawWakerVTable) -> bool {
        // If the waker is already registered, there is no need to replace it,
        // and there is no `wake` to synchronize with: either the loaded vtable
        // is up to date, and a preceding `wake` would have consumed the
        // registration; or it is stale and a concurrent `wake` is consuming it,
        // but its pending `wake_by_ref` call targets this same waker, so the
        // task will be polled again — and that poll happens after the wake's
        // claim, so it cannot load the stale registration a second time.
        if waker.data() == self.data.get()
            && ptr::from_ref(waker.vtable()) == vtable.map_addr(|addr| addr & !REGISTERED)
        {
            return true;
        }
        // A concurrent `wake` may be happening.
        // For some reason, checking the flag with SYNC=false compiles to better asm than
        // `(SYNC && vtable.addr() & WAKING != 0) || (!SYNC && vtable.addr() == WAKING)`
        if vtable.addr() & WAKING != 0 {
            // A thread is currently waking the registered waker, so we can
            // assume we should not wait and return immediately.
            // An Acquire fence is emitted with SYNC=true to synchronize with `wake`
            // only for CACHED=true, as CACHED=false already handle it in `register`.
            if CACHED && SYNC {
                fence(Acquire);
            }
            return false;
        }
        // The waker data will surely be modified after this point, so it
        // needs an acquire fence, which has already been emitted for
        // CACHED=false (see register top comment)
        if CACHED {
            fence(Acquire);
        }
        // `Waker::drop`/`Waker::clone` may panic, in which case the state is reset
        // to a noop waker. Because this is an edge-case, we don't care about choosing
        // the right operation depending on SYNC, swap(SeqCst) works for both.
        let guard = defer(|| {
            if CACHED {
                self.data.set(ptr::null());
                (self.vtable).swap(ptr::from_ref(NOOP_VTABLE).cast_mut(), SeqCst);
            }
        });
        if vtable.addr() & REGISTERED != 0 {
            // Relaxed ordering is fine as it will be followed by a swap/store.
            // An Acquire fence is emitted in case of failure with SYNC=true
            // to synchronize with `wake`.
            if let Err(vtable) =
                (self.vtable).compare_exchange(vtable, ptr::null_mut(), Relaxed, Relaxed)
            {
                if SYNC {
                    fence(Acquire);
                }
                check_concurrent_wake(vtable);
                guard.forget();
                return false;
            }
            // SAFETY: No concurrent `register` can happen, so waker data is safe to access
            drop(unsafe { self.waker(vtable.map_addr(|addr| addr & !REGISTERED)) });
        } else if CACHED {
            // SAFETY: No concurrent `register` can happen, so waker data is safe to access
            drop(unsafe { self.waker(vtable) });
        };
        let waker = ManuallyDrop::new(waker.clone());
        guard.forget();
        // Register the waker clone.
        self.data.set(waker.data());
        self.register_vtable(waker.vtable(), true)
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
        let vtable = self.vtable.load(Relaxed);
        if vtable.addr() & (REGISTERED | WAKING) != REGISTERED {
            return false;
        }
        let unregistered = unregistered(vtable);
        // Relaxed order is ok here, as `unregister` and `register` are called in the same
        // thread, i.e. sequenced-before, so there is no risk that this CAS make possible a
        // stale load of an unregistered vtable instead of a registered one.
        // It may provoke a stale load of registered vtable, but `wake` deals with it.
        match (self.vtable).compare_exchange(vtable, unregistered, Relaxed, Relaxed) {
            Ok(_) if !CACHED => {
                // SAFETY: No concurrent `register` can happen, waker data is safe to access
                drop(unsafe { self.waker(unregistered) });
                true
            }
            res => res.map_err(check_concurrent_wake).is_ok(),
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
        if SYNC {
            // SYNC=true requires a Release write on the state, but we don't want to set
            // the WAKING bit if there is no waker, as it would require unsetting it.
            // So we attempt a `fetch_add(0)` and hope for no concurrent `register`.
            // The next `register` synchronizes with this Release write even if other
            // RMWs intervene, through the release sequence it heads.
            is_registered(self.vtable.load(Relaxed))
                || is_registered(self.vtable.fetch_byte_add(0, Release))
        } else {
            self.vtable.load(SeqCst).addr() & REGISTERED != 0
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
    ///
    /// With `SYNC=true`, it introduces a small check to not pay the full wake
    /// cost when no waker is registered.
    #[inline]
    pub fn wake_cold(&self) {
        self.check_before_wake(true, Self::wake_waker);
    }

    fn wake_waker(waker: Option<Waker>) {
        debug_assert!(waker.is_none() || !CACHED);
        if let Some(waker) = waker {
            Waker::wake(waker);
        }
    }

    #[inline(always)]
    fn check_before_wake<R>(&self, cold: bool, wake: impl FnOnce(Option<Waker>) -> R) -> R {
        if SYNC {
            if cold {
                if !self.has_waker_registered() {
                    return wake(None);
                }
                self.wake_sync_cold(wake)
            } else {
                self.wake_sync(false, wake)
            }
        } else {
            // Loading the vtable with SeqCst is necessary for the pattern
            // `store X; load Y || store Y; load X` to not miss any
            // notification, where X is the wake condition and Y the vtable.
            let vtable = self.vtable.load(SeqCst);
            if vtable.addr() & REGISTERED == 0 {
                wake(None)
            } else if cold {
                self.wake_unsync_cold(vtable, wake)
            } else {
                self.wake_unsync(vtable, wake)
            }
        }
    }

    fn unregister_and_wake<R, U>(
        &self,
        vtable: *mut RawWakerVTable,
        unregister: impl Fn(*mut RawWakerVTable) -> U,
        wake: impl FnOnce(Option<Waker>) -> R,
    ) -> R {
        let unregistered = unregistered(vtable);
        // SAFETY: The vtable WAKING flag has been set, so no waker can
        // be registered. For SYNC=false, `register` set vtable with SeqCst
        // so it synchronizes with the Acquire CAS before this call.
        // For SYNC=true, vtable is only updated through RMWs, which form a
        // release sequence. It makes the previous Acquire RMW/fence
        // synchronize with the Release swap subsequent to data write.
        let waker = unsafe { self.waker(unregistered) };
        // With CACHED=true, there is no `take` operation, so we know
        // it's a `wake` and we need to call `wake_by_ref`. However,
        // because the waker is used by reference, it cannot be
        // returned after having unregistered it, as it may be
        // overwritten and invalidated after.
        if CACHED {
            // Use a guard to ensure `unregister` is called even if
            // wake_by_ref panics.
            let _guard = defer(|| unregister(unregistered));
            ManuallyDrop::new(waker).wake_by_ref();
            wake(None)
        } else {
            unregister(unregistered);
            wake(Some(waker))
        }
    }

    fn wake_sync<R>(&self, registered: bool, wake: impl FnOnce(Option<Waker>) -> R) -> R {
        // There might be a waker registered, set the WAKING bit with Release
        // ordering, so it can synchronize with the Acquire RMW in `register`.
        // The synchronization holds even when `register` reads a later value
        // (e.g. the reset), through the release sequence headed by this RMW,
        // all SYNC=true vtable writes being RMWs.
        // If there is a waker registered, Acquire ordering is also required to
        // access the data stored in `register`.
        let ordering = if registered { AcqRel } else { Release };
        let vtable = self.vtable.fetch_or(WAKING, ordering);
        if is_registered(vtable) {
            // There is a waker registered in the end, so emit the Acquire fence
            // not handled by previous `fetch_or`.
            if !registered {
                fence(Acquire);
            }
            // At this point the only concurrent operation will be:
            // - fetch_add(0), no issue
            // - fetch_or(WAKING), another `wake` is losing the race
            // - CAS(_, NULL), will fail because of WAKING flag
            // The state can thus be swapped to unregistered, with Release
            // ordering to synchronize waker data access.
            // It could be tempting to use a store instead, but it would not
            // work as it might overwrite a potential fetch_or and prevent
            // the synchronization of a racing wake with the next register.
            self.unregister_and_wake(vtable, |vt| self.vtable.swap(vt, Release), wake)
        } else if vtable.addr() & WAKING != 0 {
            // A concurrent `wake` has won the race, just return.
            wake(None)
        } else {
            // Too bad, no waker was registered. It means that a concurrent `register`
            // might be concurrently storing a waker and swap the vtable with a
            // registered one. We still need to unset the WAKING flag, but we don't care
            // if it fails, as it would mean the flag has been unset anyway.
            // It is theoretically possible that WAKING flag has been already unset and
            // that another thread has already set it back. In this case, either a waker
            // has been registered and this CAS will fail, or the vtable was unregistered
            // and the other thread doesn't care as much as us about its CAS succeeding.
            // This edge case makes it impossible to use swap to set the WAKING flag, as
            // it would not be possible to reset the correct vtable.
            // There is nothing to synchronize here, hence the Relaxed ordering.
            let waking = vtable.map_addr(|addr| addr | WAKING);
            let _ = (self.vtable).compare_exchange(waking, vtable, Relaxed, Relaxed);
            wake(None)
        }
    }

    #[cold]
    #[inline(never)]
    fn wake_sync_cold<R>(&self, wake: impl FnOnce(Option<Waker>) -> R) -> R {
        self.wake_sync(true, wake)
    }

    fn wake_unsync<R>(
        &self,
        vtable: *mut RawWakerVTable,
        wake: impl FnOnce(Option<Waker>) -> R,
    ) -> R {
        // Try swapping the vtable with WAKING. If it fails, it means either:
        // - a concurrent `wake` has won the race
        // - the waker was overwritten, so the registering thread is supposed
        //   to check again its wakeup condition
        // - the waker was unregistered
        // In all cases, there is no registered waker to wake.
        // Acquire ordering is necessary to access the waker data; it may have
        // changed between the CAS and the previous SeqCst load. Because it's
        // not SeqCst, it might provoke stale loads, but this is not an issue,
        // as the following CAS in `register`/`unregister`/`wake` will then fail.
        // (It is also not an issue if `register` early returns true, as it is
        // the same as if `wake` was called after `register`)
        let waking = ptr::without_provenance_mut(WAKING);
        if (self.vtable)
            .compare_exchange(vtable, waking, Acquire, Relaxed)
            .is_err()
        {
            return wake(None);
        };
        // The state can be reset to unregistered with a simple store.
        // Using Release is necessary for waker data access synchronization.
        // Not using SeqCst doesn't break the lost-wakeup guarantee. Imagine the
        // deadlock scenario: the registering thread checks the wakeup condition
        // after registration and finds it unset, while this stale store is loaded
        // by a `wake` called after the condition was set. The unset condition load
        // is coherence-ordered before the condition store (it reads a value the
        // store overwrites), so with thread sequencing, the registration precedes
        // the stale vtable load in the SeqCst total order. But loading a vtable
        // older in modification order than the registration would coherence-order
        // the load *before* the registration — contradiction.
        // See https://github.com/rust-lang/miri/issues/5104
        let ordering = if cfg!(miri) { SeqCst } else { Release };
        self.unregister_and_wake(vtable, |vt| self.vtable.store(vt, ordering), wake)
    }

    #[cold]
    #[inline(never)]
    fn wake_unsync_cold<R>(
        &self,
        vtable: *mut RawWakerVTable,
        wake: impl FnOnce(Option<Waker>) -> R,
    ) -> R {
        self.wake_unsync(vtable, wake)
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
        self.check_before_wake(false, identity)
    }

    /// Same as [`take`](Self::take), but with the taking path marked `#[cold]`.
    ///
    /// This allows the method to inline more effectively. Prefer this over
    /// `take` when taking is the uncommon case.
    #[inline]
    pub fn take_cold(&self) -> Option<Waker> {
        self.check_before_wake(true, identity)
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
/// Here is the `SpmcWaker` example rewritten for `UnsyncSpmcWaker` using `SeqCst` ordering:
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
///         // `UnsyncSpmcWaker` requires `SeqCst` ordering.
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
///         // quick check to avoid registration if already done.
///         if self.0.notified.load(Relaxed) {
///             return Poll::Ready(());
///         }
///
///         // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
///         unsafe { self.0.waker.register(cx.waker()) };
///
///         // Need to check condition **after** `register` to avoid a race
///         // condition that would result in lost notifications.
///         // `UnsyncSpmcWaker` requires `SeqCst` ordering.
///         if self.0.notified.load(SeqCst) {
///             // Unregister the waker to avoid spurious wakeups.
///             // SAFETY: mutable reference on non-cloneable `Waiter` ensures no concurrent call
///             unsafe { self.0.waker.unregister() };
///             Poll::Ready(())
///         } else {
///             Poll::Pending
///         }
///     }
/// }
///
/// fn event() -> (Notifier, Waiter) {
///     let waiter = Waiter::default();
///     (waiter.notifier(), waiter)
/// }
/// ```
pub type UnsynchronizedSpmcWaker<const CACHED: bool = true> = SpmcWaker<false, CACHED>;

struct Defer<F: FnOnce()>(ManuallyDrop<F>);

impl<F: FnOnce()> Defer<F> {
    #[inline(always)]
    fn forget(self) {
        let mut this = ManuallyDrop::new(self);
        // SAFETY: `ManuallyDrop` data is no longer accessed after this call
        unsafe { ManuallyDrop::drop(&mut this.0) }
    }
}

impl<F: FnOnce()> Drop for Defer<F> {
    #[inline(always)]
    fn drop(&mut self) {
        // SAFETY: `ManuallyDrop` data is no longer accessed after this call
        unsafe { ManuallyDrop::take(&mut self.0)() };
    }
}

#[inline(always)]
fn defer<R>(f: impl FnOnce() -> R) -> Defer<impl FnOnce()> {
    Defer(ManuallyDrop::new(|| {
        f();
    }))
}
