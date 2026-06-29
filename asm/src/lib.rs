#![allow(unexpected_cfgs)]
use core::task::Waker;
use std::{
    sync::atomic::{AtomicBool, Ordering::Relaxed},
    task::{Context, Poll},
};

#[cfg(synchronized)]
type Mode = spmc_waker::Synchronized;
#[cfg(sequential)]
type Mode = spmc_waker::Sequential;
#[cfg(unsynchronized)]
type Mode = spmc_waker::Unsynchronized;

const CACHED: bool = cfg!(cached);
type SpmcWaker = spmc_waker::SpmcWaker<Mode, CACHED>;

#[unsafe(no_mangle)]
fn asm_wake_asm(spmc: &SpmcWaker) {
    spmc.wake();
}

#[unsafe(no_mangle)]
fn asm_wake_cold_asm(spmc: &SpmcWaker) {
    spmc.wake_cold();
}

#[unsafe(no_mangle)]
unsafe fn asm_poll_wait_until_asm(
    spmc: &SpmcWaker,
    cx: &mut Context,
    condition: &AtomicBool,
) -> Poll<()> {
    unsafe { spmc.poll_wait_until(cx, |_| condition.load(Relaxed)) }
}

#[unsafe(no_mangle)]
unsafe fn asm_try_register_asm(spmc: &SpmcWaker, waker: &Waker) -> bool {
    unsafe { spmc.try_register(waker) }
}

#[unsafe(no_mangle)]
unsafe fn asm_register_asm(spmc: &SpmcWaker, waker: &Waker) {
    unsafe { spmc.register(waker) }
}

#[unsafe(no_mangle)]
unsafe fn asm_unregister_asm(spmc: &SpmcWaker) -> bool {
    unsafe { spmc.unregister() }
}

#[unsafe(no_mangle)]
fn asm_has_waker_registered_asm(spmc: &SpmcWaker) -> bool {
    spmc.has_waker_registered()
}
