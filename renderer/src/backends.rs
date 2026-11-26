use crate::Compositor;
use crate::core::backend::{Api, Backend};
use crate::graphics::compositor::{self, Choice};

/// A prioritized list of backends for the default [`Compositor`].
///
/// Each choice is tried in order until one of them succeeds. There can be any
/// amount of them, and the same renderer can appear more than once with
/// different options.
#[derive(Debug, Clone)]
pub struct Backends(compositor::Backends<Compositor>);

impl Backends {
    /// Creates an empty list of [`Backends`].
    pub fn new() -> Self {
        Self(compositor::Backends::new())
    }

    /// Adds the `wgpu` renderer with the given [`Options`](crate::wgpu::window::Options).
    #[cfg(feature = "wgpu-bare")]
    pub fn wgpu(self, options: crate::wgpu::window::Options) -> Self {
        self.push(options)
    }

    /// Adds the `vello-cpu` renderer with the given [`Options`](crate::vello_cpu::Options).
    #[cfg(feature = "vello-cpu")]
    pub fn vello_cpu(self, options: crate::vello_cpu::Options) -> Self {
        self.push(options)
    }

    /// Adds the choices that satisfy the given [`Backend`] preference.
    pub fn backend(self, backend: impl Into<Backend>) -> Self {
        Self(
            self.0
                .extend(compositor::Backends::preferred(&backend.into())),
        )
    }

    /// Adds the choices that satisfy [`Backend::Best`].
    pub fn best(self) -> Self {
        self.backend(Backend::Best)
    }

    /// Adds the choices that satisfy [`Backend::Hardware`] with the given [`Api`].
    pub fn hardware(self, api: Api) -> Self {
        self.backend(Backend::Hardware(api))
    }

    /// Adds the choices that satisfy [`Backend::Software`].
    pub fn software(self) -> Self {
        self.backend(Backend::Software)
    }

    /// Adds a custom [`Choice`].
    pub fn push(self, choice: impl Choice<Compositor>) -> Self {
        Self(self.0.push(choice))
    }

    /// Returns true if there are no choices.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for Backends {
    fn default() -> Self {
        Self::new().backend(Backend::default())
    }
}

impl From<Backend> for Backends {
    fn from(backend: Backend) -> Self {
        Self::new().backend(backend)
    }
}

impl From<Backends> for compositor::Backends<Compositor> {
    fn from(backends: Backends) -> Self {
        backends.0
    }
}
