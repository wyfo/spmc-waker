//! Overwrite of `loom` atomics to emulate `SeqCst` support.
//!
//! `SpmcWaker<Sequential>` uses the pattern `store X; load Y || store Y; load X` with `SeqCst`
//! operations. This pattern also works by inserting a `SeqCst` fence between stores and loads.
//! This is done by tracking stores in a thread-local bool, so a fence can correctly be inserted
//! before the next load.
//!
//! Moreover, this module adds tracing to atomic operations: setting `LOOM_TRACE` environment
//! variable makes all atomic operations printed into `loom.trace` file.
//!
//! Last but not least, it allows downgrading an atomic operation ordering (or to remove a fence)
//! by setting the `LOOM_DOWNGRADE` environment variable; see [`Downgrade`] for the format. It
//! makes it possible to check if the atomic orderings are correctly chosen to be minimal.
extern crate std;

use core::{cell::Cell, panic::Location, sync::atomic::Ordering};
use std::{
    fs::{File, OpenOptions},
    io::{Seek, Write},
    sync::LazyLock,
};

const TRACE_PATH: &str = "loom.trace";
const TRACE_VAR: &str = "LOOM_TRACE";
const DOWNGRADE_VAR: &str = "LOOM_DOWNGRADE";

// ========== SeqCst emulation ==========

loom::thread_local! {
    static PENDING_SEQCST_STORE: Cell<bool> = Cell::new(false);
}

fn mark_store(order: Ordering) {
    if order == Ordering::SeqCst {
        PENDING_SEQCST_STORE.with(|s| s.set(true));
    }
}

fn fence_before_load(order: Ordering) {
    if order == Ordering::SeqCst && PENDING_SEQCST_STORE.with(|s| s.replace(false)) {
        loom::sync::atomic::fence(Ordering::SeqCst);
    }
}

// ========== Tracing ==========

static TRACE_FILE: LazyLock<Option<File>> = LazyLock::new(|| {
    std::env::var_os(TRACE_VAR)?;
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(TRACE_PATH)
        .unwrap();
    Some(file)
});

/// Write a string to the trace file.
#[track_caller]
pub fn write_trace<S: AsRef<str>>(s: impl FnOnce(&Location) -> S) {
    if let Some(mut file) = TRACE_FILE.as_ref() {
        file.write_all(s(Location::caller()).as_ref().as_bytes())
            .unwrap();
    }
}

/// Clears loom trace file.
///
/// It should be called at the beginning of each model iteration.
pub fn clear_trace() {
    if let Some(mut file) = TRACE_FILE.as_ref() {
        file.set_len(0).unwrap();
        file.rewind().unwrap();
    }
}

// ========== Ordering downgrade ==========

/// A single ordering downgrade (or fence removal) selected at runtime via the
/// `LOOM_DOWNGRADE` env var. Format: `"<line>:<from>:<to>"` for an ordering
/// (e.g.`"384:SeqCst:Release"`), or `"<line>:fence"` to drop a fence.
enum Downgrade {
    Ordering { from: Ordering, to: Ordering },
    Fence,
}

static DOWNGRADE: LazyLock<Option<(u32, Downgrade)>> = LazyLock::new(|| {
    fn parse_ordering(s: &str) -> Ordering {
        match s {
            "Relaxed" => Ordering::Relaxed,
            "Acquire" => Ordering::Acquire,
            "Release" => Ordering::Release,
            "AcqRel" => Ordering::AcqRel,
            "SeqCst" => Ordering::SeqCst,
            _ => panic!("invalid ordering in LOOM_DOWNGRADE: {s:?}"),
        }
    }
    let var = std::env::var_os(DOWNGRADE_VAR)?.into_string().unwrap();
    let parts = var.split(':').collect::<std::vec::Vec<_>>();
    Some(match parts.as_slice() {
        [line, from, to] => (
            line.parse().unwrap(),
            Downgrade::Ordering {
                from: parse_ordering(from),
                to: parse_ordering(to),
            },
        ),
        [line, "fence"] => (line.parse().unwrap(), Downgrade::Fence),
        _ => panic!("invalid {DOWNGRADE_VAR}: {parts:?}"),
    })
});

#[track_caller]
fn handle_downgrade(order: Ordering) -> Ordering {
    match DOWNGRADE.as_ref() {
        Some(&(line, Downgrade::Ordering { from, to }))
            if from == order && Location::caller().line() == line =>
        {
            to
        }
        _ => order,
    }
}

// ========== Atomics wrapping ==========

#[track_caller]
pub fn fence(order: Ordering) {
    match DOWNGRADE.as_ref() {
        Some(&(line, Downgrade::Fence)) if Location::caller().line() == line => return,
        _ => {}
    }
    loom::sync::atomic::fence(order);
    loom_trace!("fence({order:?})");
}

#[derive(Debug, Default)]
pub struct AtomicUsize(loom::sync::atomic::AtomicUsize);

impl AtomicUsize {
    pub fn new(x: usize) -> Self {
        Self(loom::sync::atomic::AtomicUsize::new(x))
    }

    #[track_caller]
    pub fn load(&self, order: Ordering) -> usize {
        let order = handle_downgrade(order);
        fence_before_load(order);
        let res: usize = self.0.load(order);
        loom_trace!("load({order:?}) -> {res}");
        res
    }

    #[track_caller]
    pub fn store(&self, x: usize, order: Ordering) {
        let order = handle_downgrade(order);
        self.0.store(x, order);
        mark_store(order);
        loom_trace!("store({x}, {order:?})");
    }

    #[track_caller]
    pub fn swap(&self, x: usize, order: Ordering) -> usize {
        let order = handle_downgrade(order);
        // An RMW is a load then a store: fence for the load part, record the
        // store part for the next load.
        fence_before_load(order);
        let res = self.0.swap(x, order);
        mark_store(order);
        loom_trace!("swap({x}, {order:?}) -> {res}");
        res
    }

    #[track_caller]
    pub fn compare_exchange(
        &self,
        current: usize,
        new: usize,
        success: Ordering,
        failure: Ordering,
    ) -> Result<usize, usize> {
        let (success, failure) = (handle_downgrade(success), handle_downgrade(failure));
        fence_before_load(success);
        let res = self.0.compare_exchange(current, new, success, failure);
        // The store only happens on success, so only then is a store pending.
        if res.is_ok() {
            mark_store(success);
        }
        loom_trace!(
            "compare_exchange(cur={current}, new={new}, {success:?}/{failure:?}) -> {res:?}"
        );
        res
    }

    #[track_caller]
    pub fn compare_exchange_weak(
        &self,
        current: usize,
        new: usize,
        success: Ordering,
        failure: Ordering,
    ) -> Result<usize, usize> {
        self.compare_exchange(current, new, success, failure)
    }

    #[track_caller]
    pub fn fetch_add(&self, val: usize, order: Ordering) -> usize {
        let order = handle_downgrade(order);
        fence_before_load(order);
        let res = self.0.fetch_add(val, order);
        mark_store(order);
        loom_trace!("fetch_add({val}, {order:?}) -> {res}");
        res
    }
}

#[derive(Debug, Default)]
pub struct AtomicPtr<T>(loom::sync::atomic::AtomicPtr<T>);

impl<T> AtomicPtr<T> {
    pub fn new(x: *mut T) -> Self {
        Self(loom::sync::atomic::AtomicPtr::new(x))
    }

    #[track_caller]
    pub fn load(&self, order: Ordering) -> *mut T {
        let order = handle_downgrade(order);
        fence_before_load(order);
        let res = self.0.load(order);
        loom_trace!("load({order:?}) -> {res:?}");
        res
    }

    #[track_caller]
    pub fn store(&self, x: *mut T, order: Ordering) {
        let order = handle_downgrade(order);
        self.0.store(x, order);
        mark_store(order);
        loom_trace!("store({x:?}, {order:?})");
    }
}

// ========== Macro ==========

#[doc(hidden)]
#[macro_export]
macro_rules! loom_trace {
    ($($t:tt)*) => {{
        extern crate std;
        $crate::loom::write_trace(|loc| {
            let thread_id = ::loom::thread::current().id();
            let args = format_args!($($t)*);
            std::format!("[{thread_id:?}] {loc}: {args}\n",)
        });
    }};
}
use loom_trace;
