//! `SpmcWaker` uses an atomic `usize` state split in two parts:
//! - three mutually exclusive bit flags: REGISTERED, CACHED and REGISTERING
//! - a registration count, used as an ABA counter
//!
//! The registration count is incremented each time a waker is registered, i.e. the REGISTERED
//! flag is set. The REGISTERING flag is set with safe registration policies to catch concurrent
//! registrations.
//!
//! The ownership of a registered waker is claimed by zeroing the REGISTERED flag. After that, it
//! is possible to set the CACHED flag to give back the waker ownership, the waker staying in the
//! cache.
//!
//! `SpmcWaker` stores the waker parts in atomic pointers. Those are written before setting the
//! REGISTERED flag. To claim the registered waker ownership, the pointers are loaded and an
//! attempt to update the state is made. If the update is successful, it means the pointers form
//! a valid waker.
//!
//! This algorithm is somewhat similar to Seqlock: load the count, load the data, reload the count
//! and compare it to the previous loaded value, equality means that there was no concurrent data
//! update so it is safe to use. `SpmcWaker` is different in the sense that the waker ownership is
//! claimed by updating the state instead of just reloading it.
//!
//! # loom/miri refinement
//!
//! With `#[cfg(any(loom, miri))]`, the non-flags state bits are split in two halves:
//! - the registration count on MSBs, which is used as the "epoch";
//! - a store epoch on LSBs, which is the value of the registration count when a waker is stored.
//!
//! The registration count is in fact kept with the exact same semantic, just using fewer bits.
//!
//! This augmented state is used in combination with two ghost fields in `SpmcWaker`:
//! - `store_epoch`, an atomic which stores the epoch when a waker is stored;
//! - `waker_cells`, an array associating one `Cell` per waker stored.
//!
//! `store_epoch` is loaded with the waker pointers and kept in the temporary `PendingWaker`.
//! When the waker is confirmed (after successfully updating the state), the previously loaded
//! `store_epoch` is compared to the related part of the state (before confirmation update).
//! Having both equal in all executions ensures the waker pointers that have been loaded at the
//! same time are consistent and form a valid waker.
//!
//! The `ConfirmedWaker` returned by `PendingWaker::confirm` embeds a reference to the `Cell`
//! indexed in `waker_cells` by `store_epoch`. Each waker operation (`wake`/`wake_by_ref`/`drop`)
//! accesses the cell to ensure they are correctly synchronized, i.e., `wake_by_ref` always happens
//! before `drop`.

pub(crate) type State = usize;
pub(crate) type AtomicState = crate::AtomicUsize;
pub(crate) const REGISTERED: State = 0b001;
pub(crate) const CACHED: State = 0b010;
pub(crate) const REGISTERING: State = 0b100;
pub(crate) const FLAGS_MASK: State = REGISTERED | CACHED | REGISTERING;
#[cfg(not(any(loom, miri)))]
pub(crate) const REGISTRATION_INCR: State = FLAGS_MASK + 1;
#[cfg(any(loom, miri))]
const HALF_BITS: u32 = (State::BITS - 3) / 2;
#[cfg(any(loom, miri))]
pub(crate) const REGISTRATION_INCR: State = 1 << (State::BITS - HALF_BITS);

#[cfg(any(loom, miri))]
fn registration_count(state: State) -> usize {
    state & !(REGISTRATION_INCR - 1)
}

#[cfg(any(loom, miri))]
pub(crate) fn store_epoch(state: State) -> usize {
    registration_count(state << HALF_BITS)
}

#[cfg(any(loom, miri))]
pub(crate) fn set_store_epoch(state: &mut State) -> usize {
    let epoch = registration_count(*state);
    // Erase the current store epoch and write the new one.
    *state &= !(store_epoch(*state) >> HALF_BITS);
    *state |= epoch >> HALF_BITS;
    epoch
}
