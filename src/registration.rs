//! [`RegistrationPolicy`] and its variants.
use core::{
    fmt::Debug,
    task::{Context, Poll, Waker},
};

use crate::{
    RegisteredWaker, SpmcWaker,
    synchronization::Synchronization,
    wait_until::{WaitUntil, WakeCondition},
};

/// Generic parameter of [`SpmcWaker`] which determines the policy for enforcing single waker
/// registration.
///
/// `SpmcWaker` algorithm assumes a single thread registering a waker at a time. Depending on the
/// policy variant, a concurrent registration may:
/// - panic (the default)
/// - fail silently
/// - cause undefined behavior (and requires unsafe code)
#[allow(private_bounds)]
pub trait RegistrationPolicy:
    private::RegistrationPolicy + Send + Sync + Debug + Sized + 'static
{
    #[doc(hidden)]
    unsafe fn register<'a, S: Synchronization, const CACHING: bool>(
        spmc_waker: &'a SpmcWaker<S, CACHING, Self>,
        waker: &Waker,
    ) -> RegisteredWaker<'a, S, CACHING, Self> {
        RegisteredWaker::new(spmc_waker, spmc_waker.register_impl(waker), false)
    }

    #[doc(hidden)]
    unsafe fn wait_until<
        S: Synchronization,
        const CACHING: bool,
        F: FnMut(bool) -> W,
        W: WakeCondition,
    >(
        spmc_waker: &SpmcWaker<S, CACHING, Self>,
        wake_condition: F,
    ) -> WaitUntil<'_, F, S, CACHING, Self> {
        WaitUntil::new(spmc_waker, wake_condition)
    }

    #[doc(hidden)]
    unsafe fn poll_wait_until<
        S: Synchronization,
        const CACHING: bool,
        F: FnMut(bool) -> W,
        W: WakeCondition,
    >(
        spmc_waker: &SpmcWaker<S, CACHING, Self>,
        cx: &mut Context,
        wake_condition: F,
    ) -> Poll<W::Output> {
        spmc_waker.poll_wait_until_impl(cx, wake_condition)
    }
}

/// Registration policies which are safe to use.
pub trait SafeRegistration: RegistrationPolicy {}

pub(crate) enum RegistrationMode {
    Strict,
    Lenient,
    Unchecked,
}

/// Concurrent registrations may panic.
#[derive(Debug)]
pub struct Strict;
impl private::RegistrationPolicy for Strict {
    const MODE: RegistrationMode = RegistrationMode::Strict;
}
impl RegistrationPolicy for Strict {}
impl SafeRegistration for Strict {}

/// Concurrent registrations may fail silently.
#[derive(Debug)]
pub struct Lenient;
impl private::RegistrationPolicy for Lenient {
    const MODE: RegistrationMode = RegistrationMode::Lenient;
}
impl RegistrationPolicy for Lenient {}
impl SafeRegistration for Lenient {}

/// Concurrent registrations are unsound and may cause undefined behavior.
///
/// As a consequence, registration methods are unsafe.
///
/// This variant saves one RMW in `register` compared to the other safe policies.
#[derive(Debug)]
pub struct Unchecked;
impl private::RegistrationPolicy for Unchecked {
    const MODE: RegistrationMode = RegistrationMode::Unchecked;
}
impl RegistrationPolicy for Unchecked {}

mod private {
    use crate::registration::RegistrationMode;

    pub(crate) trait RegistrationPolicy {
        const MODE: RegistrationMode;
        const SAFE: bool = matches!(
            Self::MODE,
            RegistrationMode::Strict | RegistrationMode::Lenient
        );
    }
}
