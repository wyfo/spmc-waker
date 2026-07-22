#![allow(unexpected_cfgs)]
use core::task::Waker;
use std::{
    sync::atomic::{AtomicBool, Ordering::Relaxed},
    task::{Context, Poll},
};

#[cfg(synchronized)]
type S = spmc_waker::synchronization::Synchronized;
#[cfg(sequential)]
type S = spmc_waker::synchronization::Sequential;
#[cfg(unsynchronized)]
type S = spmc_waker::synchronization::Unsynchronized;

const CACHED: bool = cfg!(cached);

#[cfg(strict)]
type R = spmc_waker::registration::Strict;
#[cfg(unchecked)]
type R = spmc_waker::registration::Unchecked;

type SpmcWaker = spmc_waker::SpmcWaker<S, CACHED, R>;

#[unsafe(no_mangle)]
fn asm_wake_asm(spmc: &SpmcWaker) {
    spmc.wake();
}

#[unsafe(no_mangle)]
fn asm_wake_cold_asm(spmc: &SpmcWaker) {
    spmc.wake_cold();
}

#[cfg_attr(not(unchecked), allow(unused_unsafe))]
#[unsafe(no_mangle)]
unsafe fn asm_poll_wait_until_asm(
    spmc: &SpmcWaker,
    cx: &mut Context,
    condition: &AtomicBool,
) -> Poll<()> {
    unsafe { spmc.poll_wait_until(cx, |_| condition.load(Relaxed)) }
}

#[cfg_attr(not(unchecked), allow(unused_unsafe))]
#[unsafe(no_mangle)]
unsafe fn asm_register_asm(spmc: &SpmcWaker, waker: &Waker) {
    unsafe { spmc.register(waker) };
}

#[unsafe(no_mangle)]
unsafe fn asm_unregister_asm(registered: spmc_waker::Registered<'_, S, CACHED, R>) {
    registered.unregister();
}

#[unsafe(no_mangle)]
fn asm_take_asm(spmc: &SpmcWaker) -> Option<Waker> {
    spmc.take()
}
