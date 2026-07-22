//! [`SpmcWaker::wait_until`] associated types.
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{RegistrationPolicy, SpmcWaker, Synchronization};

/// Future returned by [`SpmcWaker::wait_until`]
pub struct WaitUntil<'a, F, S: Synchronization, const CACHING: bool, R: RegistrationPolicy> {
    spmc_waker: &'a SpmcWaker<S, CACHING, R>,
    wake_condition: F,
}

impl<'a, F, S: Synchronization, const CACHING: bool, R: RegistrationPolicy>
    WaitUntil<'a, F, S, CACHING, R>
{
    pub(crate) fn new(spmc_waker: &'a SpmcWaker<S, CACHING, R>, wake_condition: F) -> Self {
        Self {
            spmc_waker,
            wake_condition,
        }
    }
}

impl<F, S: Synchronization, const CACHING: bool, R: RegistrationPolicy> Unpin
    for WaitUntil<'_, F, S, CACHING, R>
{
}

impl<
        F: FnMut(bool) -> W,
        W: WakeCondition,
        S: Synchronization,
        const CACHING: bool,
        R: RegistrationPolicy,
    > Future for WaitUntil<'_, F, S, CACHING, R>
{
    type Output = W::Output;
    #[inline(always)]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        (self.spmc_waker).poll_wait_until_impl(cx, |registered| (self.wake_condition)(registered))
    }
}

/// Wake condition returned by closure passed in [`SpmcWaker::wait_until`].
///
/// Typically implemented by `bool` and `Option<T>`. When met, it provides
/// an output that can be returned by `wait_until`.
pub trait WakeCondition {
    /// Wake condition output when met.
    type Output;
    /// Try getting the wake condition output, thereby checking if it is met.
    fn try_into_output(self) -> Option<Self::Output>;
}

impl WakeCondition for bool {
    type Output = ();
    fn try_into_output(self) -> Option<Self::Output> {
        self.then_some(())
    }
}

impl<T> WakeCondition for Option<T> {
    type Output = T;
    fn try_into_output(self) -> Option<Self::Output> {
        self
    }
}
