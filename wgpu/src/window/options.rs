//! Configure the `wgpu` backend of a [`Compositor`].
use crate::core::backend;
use crate::graphics::Shell;
use crate::graphics::compositor::{self, Choice, backends::Construction};
use crate::window::compositor::{self as wgpu_compositor, Compositor, Settings};
use crate::window::factory::{self, Factory};

use std::sync::Arc;

/// The options of the `wgpu` backend.
///
/// [`Options`] are a [`Choice`] for any compositor that can be created from
/// a `wgpu` [`Compositor`].
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    /// The graphics APIs `wgpu` may use.
    ///
    /// The `WGPU_BACKEND` environment variable overrides it, if set.
    ///
    /// By default, it is [`wgpu::Backends::all`].
    pub backends: wgpu::Backends,

    /// The [`Factory`] used to choose the adapter and create the device, if any.
    ///
    /// By default, it is `None`.
    pub factory: Option<factory::Shared>,
}

impl Options {
    /// Creates the default [`Options`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the graphics APIs `wgpu` may use.
    pub fn backends(self, backends: wgpu::Backends) -> Self {
        Self { backends, ..self }
    }

    /// Sets the [`Factory`] used to choose the adapter and create the device.
    pub fn factory(self, factory: impl Factory) -> Self {
        Self {
            factory: Some(factory::Shared::new(factory)),
            ..self
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            backends: wgpu::Backends::all(),
            factory: None,
        }
    }
}

impl<C> Choice<C> for Options
where
    C: From<Compositor> + 'static,
{
    fn construct<'a>(
        &'a self,
        settings: &'a backend::Settings,
        display: Arc<dyn compositor::Display>,
        compatible_window: Arc<dyn compositor::Window>,
        shell: Shell,
    ) -> Construction<'a, C> {
        Box::pin(async move {
            let mut settings = Settings::new(settings, self);

            if let Some(backends) = wgpu::Backends::from_env() {
                settings.backends = backends;
            }

            if let Some(present_mode) = wgpu_compositor::present_mode_from_env() {
                settings.present_mode = present_mode;
            }

            let compositor =
                wgpu_compositor::new(settings, display, compatible_window, shell).await?;

            Ok(C::from(compositor))
        })
    }
}
