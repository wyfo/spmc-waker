use core::fmt::Debug;

/// Generic parameter of [`SpmcWaker`] which determines its synchronization guarantees.
///
/// As a consequence, it impacts how the wake condition should be accessed.
///
/// `SpmcWaker` uses the `store X; load Y || store Y; load X` pattern, where `X` is the wake
/// condition, and `Y` the waker registration state (`load Y` is done in [`wake`] while `store Y`
/// corresponds to [`register`]). There are four main ways to make this pattern work, i.e., either
/// `load Y` sees a waker registered, or `load X` sees the wake condition met:
/// - every operation uses `SeqCst`
/// - insert `SeqCst` fences between stores and loads
/// - use RMW operations for `X` store + load, with `Acquire` ordering for store and `Release`
///   ordering for load
/// - use RMW operations for `Y` store + load, with `Acquire` ordering for store and `Release`
///   ordering for load
///
/// Among these four ways, two impact `Y` operations, and others only depend on `X` or fences,
/// which gives 3 different synchronization variants:
/// - [`Synchronized`] (the default), using RMW operations in `SpmcWaker` (`Y`)
/// - [`Sequential`], using `SeqCst` operations in `SpmcWaker`
/// - [`Unsynchronized`], relying on `SeqCst` fences or RMW operations with appropriate ordering
///   on the wake condition (`X`) to be used
///
/// While `Sequential` and `Unsynchronized` put requirements on the wake condition check, they only
/// concern the check after the registration. Checks executed before, as done by [`wait_until`],
/// can be relaxed. For example, with `Unsynchronized`, a first check can omit a `SeqCst` fence, or
/// replace an RMW by a load.
///
/// # Which variant to choose
///
/// In doubt, use the default one which will work in all cases. Otherwise, the choice depends
/// mainly on the existing constraints on the wake condition accesses, on the architecture, and on
/// the operation to optimize.
///
/// For example, if the wake condition is already accessed through RMW, and the appropriate
/// orderings are cheap to add (RMW ordering makes no difference on x86), `Unsynchronized`
/// would be the go-to.
///
/// `SpmcWaker` implementation (including its generic `Synchronization` parameter) was built around
/// optimizing [`wake_cold`] when no waker is registered. Its typical use case is a MPSC channel
/// using `SpmcWaker` for consumer notification, whose send operation calls `wake_cold`, while not
/// being empty (no consumer to notify) most of the time. The best optimization for `wake_cold` is
/// to be read-only, which is achieved by `Sequential` and `Unsynchronized` (and `Synchronized` on
/// x86, although it still adds the overhead of a `SeqCst` fence).
///
/// In any case, profiling and benchmarking the different variants will often give the best answer.
///
/// [`SpmcWaker`]: crate::SpmcWaker
/// [`wake`]: crate::SpmcWaker::wake
/// [`register`]: crate::SpmcWaker::register
/// [`wake_cold`]: crate::SpmcWaker::wake_cold
/// [`wait_until`]: crate::SpmcWaker::wait_until
#[allow(private_bounds)]
pub trait Synchronization: private::Synchronization + Send + Sync + Debug + 'static {}

pub(crate) enum SyncMode {
    Synchronized,
    Sequential,
    Unsynchronized,
}

/// [`wake`] synchronizes with [`register`].
///
/// This is the default and the simplest mode; it has no requirement on the wake condition access,
/// which can use `Relaxed` ordering.
///
/// As a consequence, `wake` (or [`wake_cold`]) always executes an RMW operation, even if there is no
/// waker registered. On x86 architecture, this RMW operation can however be optimized as a
/// `SeqCst` fence when no waker is registered, making it read-only with minimal contention on
/// `SpmcWaker` cache-line.
///
/// [`wake`]: crate::SpmcWaker::wake
/// [`register`]: crate::SpmcWaker::register
/// [`wake_cold`]: crate::SpmcWaker::wake_cold
#[derive(Debug)]
pub struct Synchronized;
impl Synchronization for Synchronized {}
impl private::Synchronization for Synchronized {
    const MODE: SyncMode = SyncMode::Synchronized;
}

/// `SpmcWaker` uses `SeqCst` ordering internally.
///
/// It requires the wake condition to be accessed using `SeqCst` ordering.
///
/// As a consequence, when there is no waker registered, [`wake`] becomes a simple `SeqCst` load,
/// thus a read-only operation with minimal contention on `SpmcWaker` cache-line.
///
/// [`wake`]: crate::SpmcWaker::wake
#[derive(Debug)]
pub struct Sequential;
impl Synchronization for Sequential {}
impl private::Synchronization for Sequential {
    const MODE: SyncMode = SyncMode::Sequential;
}

/// `SpmcWaker` relies on external synchronization between [`wake`] and [`register`]
///
/// As described in [`Synchronization`] documentation, it requires either:
/// - `SeqCst` fences to be inserted before `wake` and after `register`
/// - the wake condition to be stored with an `Acquire` RMW operation and to be loaded
///   with a `Release` RMW operation.
///
/// As a consequence, when there is no waker registered, `wake` becomes a simple `Relaxed` load,
/// thus a read-only operation with minimal contention on `SpmcWaker` cache-line.
///
/// [`wake`]: crate::SpmcWaker::wake
/// [`register`]: crate::SpmcWaker::register
#[derive(Debug)]
pub struct Unsynchronized;
impl Synchronization for Unsynchronized {}
impl private::Synchronization for Unsynchronized {
    const MODE: SyncMode = SyncMode::Unsynchronized;
}

mod private {
    use crate::sync::SyncMode;

    pub(crate) trait Synchronization {
        const MODE: SyncMode;
        const SYNC: bool = matches!(Self::MODE, SyncMode::Synchronized);
    }
}
