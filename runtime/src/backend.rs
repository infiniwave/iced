//! Configure a [`Backend`](crate::core::Backend) at runtime.
use crate::Task;
use crate::core::backend;
use crate::futures::futures::channel::oneshot;
use crate::task;

use std::any::Any;
use std::fmt;

/// An backend operation.
#[derive(Debug)]
pub enum Action {
    /// Switches the [`backend::Settings`] of the current application.
    ///
    /// If [`AnyBackends`] are provided, they take precedence over
    /// [`backend::Settings::backend`].
    Configure(
        backend::Settings,
        Option<AnyBackends>,
        oneshot::Sender<Result<(), backend::Error>>,
    ),
}

/// The compositor backends of an application, with their type erased.
///
/// The runtime does not know the compositor of the application; so the
/// backends are carried around opaquely until the shell, which does know it,
/// recovers them.
pub struct AnyBackends(Box<dyn Any + Send>);

impl AnyBackends {
    /// Erases the type of the given backends.
    pub fn new<T: Any + Send>(backends: T) -> Self {
        Self(Box::new(backends))
    }

    /// Recovers the backends, if they have the given type.
    pub fn downcast<T: Any>(self) -> Result<T, Self> {
        self.0.downcast().map(|backends| *backends).map_err(Self)
    }
}

impl fmt::Debug for AnyBackends {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AnyBackends(..)")
    }
}

/// Returns a [`Task`] that switches the [`backend::Settings`] of the current application.
///
/// This can be leveraged to switch renderers at runtime.
pub fn configure(settings: backend::Settings) -> Task<Result<(), backend::Error>> {
    task::oneshot(|sender| crate::Action::Backend(Action::Configure(settings, None, sender)))
}

/// Returns a [`Task`] that switches the [`backend::Settings`] of the current
/// application, using the given [`AnyBackends`].
///
/// The [`AnyBackends`] must hold the compositor backends of the application, or
/// the task will fail.
pub fn configure_with(
    settings: backend::Settings,
    backends: AnyBackends,
) -> Task<Result<(), backend::Error>> {
    task::oneshot(|sender| {
        crate::Action::Backend(Action::Configure(settings, Some(backends), sender))
    })
}
