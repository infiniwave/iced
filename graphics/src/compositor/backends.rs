//! Build a [`Compositor`] from a prioritized list of choices.
use crate::Shell;
use crate::compositor::{Compositor, Display, Window};
use crate::core::backend;
use crate::futures::{MaybeSend, MaybeSync};

use std::fmt;
use std::pin::Pin;
use std::sync::Arc;

/// A future that constructs a compositor `C`.
pub type Construction<'a, C> = Pin<Box<dyn Future<Output = Result<C, backend::Error>> + 'a>>;

/// A way of constructing a compositor `C`.
///
/// A [`Choice`] holds everything it needs to construct a compositor (e.g. its
/// options), except for the [`backend::Settings`] that every choice shares.
pub trait Choice<C>: fmt::Debug + MaybeSend + MaybeSync + 'static {
    /// Tries to construct the compositor.
    fn construct<'a>(
        &'a self,
        settings: &'a backend::Settings,
        display: Arc<dyn Display>,
        compatible_window: Arc<dyn Window>,
        shell: Shell,
    ) -> Construction<'a, C>;
}

/// A prioritized list of [`Choice`]s for constructing a compositor `C`.
///
/// Each [`Choice`] is tried in order until one of them succeeds.
pub struct Backends<C> {
    choices: Vec<Arc<dyn Choice<C>>>,
}

impl<C: 'static> Backends<C> {
    /// Creates an empty list of [`Backends`].
    pub fn new() -> Self {
        Self {
            choices: Vec::new(),
        }
    }

    /// Adds a [`Choice`] with the lowest priority so far.
    pub fn push(mut self, choice: impl Choice<C>) -> Self {
        self.choices.push(Arc::new(choice));
        self
    }

    /// Adds all the choices of the given [`Backends`], keeping their order,
    /// with the lowest priority so far.
    pub fn extend(mut self, backends: Self) -> Self {
        self.choices.extend(backends.choices);
        self
    }

    /// Returns true if there are no choices.
    pub fn is_empty(&self) -> bool {
        self.choices.is_empty()
    }

    /// Returns the amount of choices.
    pub fn len(&self) -> usize {
        self.choices.len()
    }

    /// Turns these [`Backends`] into [`Backends`] of a compositor `D` that can
    /// be created from `C`.
    pub fn map<D>(self) -> Backends<D>
    where
        C: 'static,
        D: From<C> + 'static,
    {
        Backends {
            choices: self
                .choices
                .into_iter()
                .map(|choice| Arc::new(Map(choice)) as Arc<dyn Choice<D>>)
                .collect(),
        }
    }

    /// Tries every choice in order and returns the first compositor that is
    /// successfully constructed.
    pub async fn construct(
        &self,
        settings: &backend::Settings,
        display: Arc<dyn Display>,
        compatible_window: Arc<dyn Window>,
        shell: Shell,
    ) -> Result<C, backend::Error> {
        let mut errors = Vec::new();

        for choice in &self.choices {
            match choice
                .construct(
                    settings,
                    display.clone(),
                    compatible_window.clone(),
                    shell.clone(),
                )
                .await
            {
                Ok(compositor) => return Ok(compositor),
                Err(error) => {
                    log::warn!("Backend choice {choice:?} failed: {error}");
                    errors.push(error);
                }
            }
        }

        Err(match errors.len() {
            0 => backend::Error::NoBackends,
            1 => errors.pop().expect("Error list has one element"),
            _ => backend::Error::List(errors),
        })
    }
}

impl<C: Compositor + 'static> Backends<C> {
    /// Returns the [`Backends`] that satisfy the given [`backend::Backend`]
    /// preference, as defined by [`Compositor::backends`].
    pub fn preferred(backend: &backend::Backend) -> Self {
        C::backends(backend)
    }
}

impl<C: 'static> Default for Backends<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> Clone for Backends<C> {
    fn clone(&self) -> Self {
        Self {
            choices: self.choices.clone(),
        }
    }
}

impl<C> fmt::Debug for Backends<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(&self.choices).finish()
    }
}

struct Map<C>(Arc<dyn Choice<C>>);

impl<C> fmt::Debug for Map<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<C, D> Choice<D> for Map<C>
where
    C: 'static,
    D: From<C> + 'static,
{
    fn construct<'a>(
        &'a self,
        settings: &'a backend::Settings,
        display: Arc<dyn Display>,
        compatible_window: Arc<dyn Window>,
        shell: Shell,
    ) -> Construction<'a, D> {
        let construction = self
            .0
            .construct(settings, display, compatible_window, shell);

        Box::pin(async move { construction.await.map(D::from) })
    }
}
