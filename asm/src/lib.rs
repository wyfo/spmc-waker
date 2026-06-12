//! Compilation targets for asm inspection via `check-asm.sh`.
//!
//! Each cfg flag compiles exactly one entry point.  The function name matches
//! the cfg flag so that `cargo asm --lib "$cfg"` finds it unambiguously.
//!
//! Variant naming:
//!   sc = SpmcWaker<SYNC=true,  CACHED=true>  (the default)
//!   su = SpmcWaker<SYNC=true,  CACHED=false>
//!   uc = SpmcWaker<SYNC=false, CACHED=true>   (UnsynchronizedSpmcWaker)
//!   uu = SpmcWaker<SYNC=false, CACHED=false>
#![allow(unexpected_cfgs, unused_imports)]

use core::task::Waker;
use spmc_waker::SpmcWaker;

// ── sc: SpmcWaker<true, true> ────────────────────────────────────────────────

#[cfg(sc_wake)]
#[unsafe(no_mangle)]
pub fn sc_wake(w: &SpmcWaker<true, true>) {
    w.wake()
}

#[cfg(sc_wake_cold)]
#[unsafe(no_mangle)]
pub fn sc_wake_cold(w: &SpmcWaker<true, true>) {
    w.wake_cold()
}

#[cfg(sc_register)]
#[unsafe(no_mangle)]
pub unsafe fn sc_register(w: &SpmcWaker<true, true>, waker: &Waker) -> bool {
    // SAFETY: caller must ensure no concurrent register/unregister
    unsafe { w.register(waker) }
}

#[cfg(sc_unregister)]
#[unsafe(no_mangle)]
pub unsafe fn sc_unregister(w: &SpmcWaker<true, true>) -> bool {
    // SAFETY: caller must ensure no concurrent register/unregister
    unsafe { w.unregister() }
}

#[cfg(sc_has_waker_registered)]
#[unsafe(no_mangle)]
pub fn sc_has_waker_registered(w: &SpmcWaker<true, true>) -> bool {
    w.has_waker_registered()
}

// ── su: SpmcWaker<true, false> ───────────────────────────────────────────────

#[cfg(su_wake)]
#[unsafe(no_mangle)]
pub fn su_wake(w: &SpmcWaker<true, false>) {
    w.wake()
}

#[cfg(su_wake_cold)]
#[unsafe(no_mangle)]
pub fn su_wake_cold(w: &SpmcWaker<true, false>) {
    w.wake_cold()
}

#[cfg(su_register)]
#[unsafe(no_mangle)]
pub unsafe fn su_register(w: &SpmcWaker<true, false>, waker: &Waker) -> bool {
    // SAFETY: caller must ensure no concurrent register/unregister
    unsafe { w.register(waker) }
}

#[cfg(su_unregister)]
#[unsafe(no_mangle)]
pub unsafe fn su_unregister(w: &SpmcWaker<true, false>) -> bool {
    // SAFETY: caller must ensure no concurrent register/unregister
    unsafe { w.unregister() }
}

#[cfg(su_has_waker_registered)]
#[unsafe(no_mangle)]
pub fn su_has_waker_registered(w: &SpmcWaker<true, false>) -> bool {
    w.has_waker_registered()
}

// ── uc: SpmcWaker<false, true> ───────────────────────────────────────────────

#[cfg(uc_wake)]
#[unsafe(no_mangle)]
pub fn uc_wake(w: &SpmcWaker<false, true>) {
    w.wake()
}

#[cfg(uc_wake_cold)]
#[unsafe(no_mangle)]
pub fn uc_wake_cold(w: &SpmcWaker<false, true>) {
    w.wake_cold()
}

#[cfg(uc_register)]
#[unsafe(no_mangle)]
pub unsafe fn uc_register(w: &SpmcWaker<false, true>, waker: &Waker) -> bool {
    // SAFETY: caller must ensure no concurrent register/unregister
    unsafe { w.register(waker) }
}

#[cfg(uc_unregister)]
#[unsafe(no_mangle)]
pub unsafe fn uc_unregister(w: &SpmcWaker<false, true>) -> bool {
    // SAFETY: caller must ensure no concurrent register/unregister
    unsafe { w.unregister() }
}

#[cfg(uc_has_waker_registered)]
#[unsafe(no_mangle)]
pub fn uc_has_waker_registered(w: &SpmcWaker<false, true>) -> bool {
    w.has_waker_registered()
}

// ── uu: SpmcWaker<false, false> ──────────────────────────────────────────────

#[cfg(uu_wake)]
#[unsafe(no_mangle)]
pub fn uu_wake(w: &SpmcWaker<false, false>) {
    w.wake()
}

#[cfg(uu_wake_cold)]
#[unsafe(no_mangle)]
pub fn uu_wake_cold(w: &SpmcWaker<false, false>) {
    w.wake_cold()
}

#[cfg(uu_register)]
#[unsafe(no_mangle)]
pub unsafe fn uu_register(w: &SpmcWaker<false, false>, waker: &Waker) -> bool {
    // SAFETY: caller must ensure no concurrent register/unregister
    unsafe { w.register(waker) }
}

#[cfg(uu_unregister)]
#[unsafe(no_mangle)]
pub unsafe fn uu_unregister(w: &SpmcWaker<false, false>) -> bool {
    // SAFETY: caller must ensure no concurrent register/unregister
    unsafe { w.unregister() }
}

#[cfg(uu_has_waker_registered)]
#[unsafe(no_mangle)]
pub fn uu_has_waker_registered(w: &SpmcWaker<false, false>) -> bool {
    w.has_waker_registered()
}

