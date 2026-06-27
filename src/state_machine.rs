//! `SpmcWaker` state machine uses waker's vtable pointer tagging with two bit flags:
//! - REGISTERED (`R`) = `0b01`
//! - WAKING (`W`)     = `0b10`
//!
//! Vtable pointers (without tags) can have the following values:
//! - a vtable pointer (`V`)
//! - NULL (`N`)             = `0`
//! - READ_FALLBACK (`RF`)   = `usize::MAX & !REGISTERED & !WAKING`
//!
//! States can be modified by 3 different threads:
//! - `RT` => registering thread
//! - `WT` => waking thread
//! - `NT` => notifier thread, i.e., concurrent thread which notify the waking
//!   thread to wake the fallback waker
//!
//! # States
//!
//! | vtable     | explanation               |
//! |------------|---------------------------|
//! | `V`        | no waker registered       |
//! | `V\|R`     | main waker registered     |
//! | `V\|R\|W`  | waking main waker         |
//! | `N\|R`     | fallback waker registered |
//! | `N\|R\|W`  | fallback waker notified   |
//! | `N`        | reset fallback waker      |
//! | `RF\|R`    | replacing fallback waker  |
//! | `RF\|R\|W` | waking fallback waker     |
//!
//! | thread | initial states   | final states           |
//! |--------|------------------|------------------------|
//! | `RT`   | all but `N`      | `V\|R` / `N\|R`        |
//! | `WT`   | `V\|R`           | `V` / `V\|R`           |
//! | `NT`   | `N\|R` / `RF\|R` | `N\|R\|W` / `RF\|R\|W` |
//!
//! # Transitions
//!
//! | thread | transition            | explanation                            |
//! |--------|-----------------------|----------------------------------------|
//! | `RT`   | `V -> V\|R`           | register main waker                    |
//! | `RT`   | `V\|R -> V`           | unregister main waker                  |
//! | `WT`   | `V\|R -> V\|R\|W`     | acquire main waker                     |
//! | `RT`   | `V\|R\|W -> N\|R`     | register fallback waker                |
//! | `WT`   | `V\|R\|W -> V`        | wake main waker                        |
//! | `RT`   | `N\|R -> N`           | reset fallback waker                   |
//! | `WT`   | `N\|R -> RF\|R`       | read fallback waker                    |
//! | `NT`   | `N\|R -> N\|R\|W`     | notify fallback waker                  |
//! | `RT`   | `N\|R\|W -> N`        | reset fallback waker                   |
//! | `WT`   | `N\|R\|W -> RF\|R\|W` | read fallback waker                    |
//! | `RT`   | `N -> N\|R`           | register fallback waker                |
//! | `WT`   | `N -> V`              | reset main waker                       |
//! | `RT`   | `RF\|R -> N`          | reset fallback waker                   |
//! | `WT`   | `RF\|R -> V\|R`       | replace main waker with fallback waker |
//! | `NT`   | `RF\|R -> RF\|R\|W`   | notify fallback waker                  |
//! | `RT`   | `RF\|R\|W -> N`       | reset fallback waker                   |
//! | `WT`   | `RF\|R\|W -> V`       | wake fallback waker                    |
//!
//! # Diagrams
//!
//! ### Main workflow
//!
//! ```text
//! +--------+                  +--------+                 +--------+
//! |   V    |--[RT] register-->|  V|R   |--[WT] acquire-->| V|R|W  |
//! +--------+                  +--------+                 +--------+
//!   ^   ^                         |                        |    |
//!   |   +-----[RT] unregister-----+                        |    | [RT] register fallback
//!   +----------------------[WT] wake-----------------------+    v
//!                                                             (N|R)
//! ```
//!
//! ### Fallback workflow
//!
//!```text
//!                          +--------+
//!                          |   N    |--[WT] reset main-->(V)
//!                          +--------+
//!                            ^    |
//!                 [RT] reset |    | [RT] register
//!                            |    v
//!                          +--------+                +--------+
//! (V|R|W)--[RT] register-->|  N|R   |--[NT] notify-->| N|R|W  |--[RT] reset-->(N)
//!                          +--------+                +--------+
//!                              |                         |
//!                              | [WT] read               | [WT] read
//!                              v                         v
//!                         +--------+                +--------+
//!       (N)<--[RT] reset--|  RF|R  |--[NT] notify-->| RF|R|W |--[RT] reset-->(N)
//!                         +--------+                +--------+
//!                             |                         |
//!                             | [WT] replace main       | [WT] wake
//!                             v                         v
//!                           (V|R)                      (V)
//! ```
//!
//! # Progress guarantee
//!
//! ### register
//!
//! | initial state | transitions                                                              |
//! |---------------|--------------------------------------------------------------------------|
//! | `V`           | `V -> V\|R`                                                              |
//! | `V\|R\|W`     | `V\|R\|W -> N\|R`<br>`V\|R\|W -> V -> ...`                               |
//! | `V\|R`        | `V\|R -> V -> ...`<br>`V\|R -> V\|R\|W -> ...`                           |
//! | `... -> N`    | `... -> N -> N\|R`<br>`... -> N -> V -> ...`                             |
//! | `RF\|R\|W`    | `RF\|R\|W -> N -> ...`<br>`RF\|R\|W -> V -> ...`                         |
//! | `N\|R\|W`     | `N\|R\|W -> N -> ...`<br>`N\|R\|W -> RF\|R\|W -> ...`                    |
//! | `RF\|R`       | `RF\|R -> N -> ...`<br>`RF\|R -> V\|R -> ...`                            |
//! | `N\|R`        | `N\|R -> N -> ...`<br>`N\|R -> N\|R\|W -> ...`<br>`N\|R -> RF\|R -> ...` |
//!
//! Each initial state reach a final state in a bounded number of transitions,
//! so the operation is wait-free (without taking in account waker operations).
//!
//! ### wake
//!
//! There are obvious cycles like `N|R -> RF|R -> N -> N|R`, but a thread is always making
//! progress (as `register` is wait-free, so there can't be a livelock), so the operation
//! is lock-free.
use core::{ptr, task::RawWakerVTable};

// Ensure vtable pointer can be tagged on the two LSBs.
const _: () = assert!(align_of::<RawWakerVTable>() >= 4);

pub(crate) const REGISTERED: usize = 0b01;
pub(crate) const WAKING: usize = 0b10;

pub(crate) const NULL: *mut RawWakerVTable = ptr::without_provenance_mut(0);
pub(crate) const READ_FALLBACK: *mut RawWakerVTable =
    ptr::without_provenance_mut(usize::MAX & !REGISTERED & !WAKING);

pub(crate) fn is_fallback(vtable: *mut RawWakerVTable) -> bool {
    vtable.addr().wrapping_add(4) <= 7
}

#[cfg(test)]
mod tests {
    use crate::{
        state_machine::{NULL, READ_FALLBACK, REGISTERED, WAKING, is_fallback},
        utils::{NOOP_PTR, TaggedPointerExt},
    };

    #[test]
    fn test_is_fallback() {
        for (ptr, fallback) in [(NOOP_PTR, false), (NULL, true), (READ_FALLBACK, true)] {
            for flags in [0, REGISTERED, WAKING, REGISTERED | WAKING] {
                assert_eq!(is_fallback(ptr.set(flags)), fallback)
            }
        }
    }
}
