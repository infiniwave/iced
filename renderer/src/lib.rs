//! The official renderer for iced.
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "wgpu-bare")]
pub use iced_wgpu as wgpu;

#[cfg(feature = "vello-cpu")]
pub use iced_vello_cpu as vello_cpu;

pub use iced_graphics as graphics;
pub use iced_graphics::core;

#[cfg(feature = "geometry")]
pub use iced_graphics::geometry;

mod backends;

pub use backends::Backends;

/// Runs the same expression on whatever renderer variant `$value` holds.
#[cfg(any(feature = "wgpu-bare", feature = "vello-cpu"))]
macro_rules! delegate {
    ($value:expr, $name:ident => $body:expr) => {
        match $value {
            #[cfg(feature = "wgpu-bare")]
            Self::Wgpu($name) => $body,
            #[cfg(feature = "vello-cpu")]
            Self::VelloCpu($name) => $body,
        }
    };
}

#[cfg(any(feature = "wgpu-bare", feature = "vello-cpu"))]
mod compositor;

#[cfg(any(feature = "wgpu-bare", feature = "vello-cpu"))]
mod renderer;

#[cfg(any(feature = "wgpu-bare", feature = "vello-cpu"))]
pub use compositor::{Compositor, Surface};

#[cfg(any(feature = "wgpu-bare", feature = "vello-cpu"))]
pub use renderer::Renderer;

#[cfg(all(
    feature = "geometry",
    any(feature = "wgpu-bare", feature = "vello-cpu")
))]
pub use renderer::{Frame, Geometry};

#[cfg(not(any(feature = "wgpu-bare", feature = "vello-cpu")))]
mod null {
    #[cfg(not(debug_assertions))]
    compile_error!(
        "Cannot compile `iced_renderer` in release mode \
        without a renderer feature enabled. \
        Enable either the `wgpu` or `vello-cpu` feature, or both."
    );

    /// The default graphics renderer for [`iced`].
    ///
    /// [`iced`]: https://github.com/iced-rs/iced
    pub type Renderer = ();

    /// The default graphics compositor for [`iced`].
    ///
    /// [`iced`]: https://github.com/iced-rs/iced
    pub type Compositor = ();
}

#[cfg(not(any(feature = "wgpu-bare", feature = "vello-cpu")))]
pub use null::{Compositor, Renderer};
