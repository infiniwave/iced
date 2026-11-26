//! A compositor is responsible for initializing a renderer and managing window
//! surfaces.
pub mod backends;

pub use backends::{Backends, Choice};

use crate::Viewport;
use crate::core::Color;
use crate::core::backend::{self, Backend};
use crate::core::font;
use crate::core::renderer;
use crate::futures::{MaybeSend, MaybeSync};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use thiserror::Error;

use std::borrow::Cow;
use std::fmt::Debug;

/// A graphics compositor that can draw to windows.
pub trait Compositor: Sized {
    /// The iced renderer of the backend.
    type Renderer;

    /// The surface of the backend.
    type Surface;

    /// Returns the [`Backends`] that satisfy the given [`Backend`] preference.
    ///
    /// This is used when no explicit [`Backends`] are provided. The result
    /// may be empty if the [`Compositor`] cannot satisfy the preference.
    fn backends(preference: &Backend) -> Backends<Self>;

    /// Creates a [`Self::Renderer`] for the [`Compositor`].
    fn create_renderer(&self, settings: renderer::Settings) -> Self::Renderer;

    /// Crates a new [`Surface`] for the given window.
    ///
    /// [`Surface`]: Self::Surface
    fn create_surface(&mut self, window: impl Window + Clone) -> Self::Surface;

    /// Configures a new [`Surface`] with the given dimensions.
    ///
    /// [`Surface`]: Self::Surface
    fn configure_surface(&mut self, surface: &mut Self::Surface, width: u32, height: u32);

    /// Returns [`Information`] used by this [`Compositor`].
    fn information(&self) -> Information;

    /// Loads a font from its bytes.
    fn load_font(&mut self, font: Cow<'static, [u8]>) -> Result<(), font::Error> {
        crate::text::font_system()
            .write()
            .expect("Write to font system")
            .load_font(font);

        // TODO: Error handling
        Ok(())
    }

    /// Lists all the available font families.
    fn list_fonts(&mut self) -> Result<Vec<font::Family>, font::Error> {
        use std::collections::BTreeSet;

        let font_system = crate::text::font_system()
            .read()
            .expect("Read from font system");

        let families = BTreeSet::from_iter(font_system.families());

        Ok(families.into_iter().map(font::Family::name).collect())
    }

    /// Presents the [`Renderer`] primitives to the next frame of the given [`Surface`].
    ///
    /// [`Renderer`]: Self::Renderer
    /// [`Surface`]: Self::Surface
    fn present(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &Viewport,
        background_color: Color,
        on_pre_present: impl FnOnce(),
    ) -> Result<(), SurfaceError>;

    /// Screenshots the current [`Renderer`] primitives to an offscreen texture, and returns the bytes of
    /// the texture ordered as `RGBA` in the `sRGB` color space.
    ///
    /// [`Renderer`]: Self::Renderer
    fn screenshot(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &Viewport,
        background_color: Color,
    ) -> Vec<u8>;
}

/// A window that can be used in a [`Compositor`].
///
/// This is just a convenient super trait of the `raw-window-handle`
/// traits.
pub trait Window: HasWindowHandle + Debug + MaybeSend + MaybeSync + 'static {}

impl<T> Window for T where T: HasWindowHandle + Debug + MaybeSend + MaybeSync + 'static {}

/// An owned display handle that can be used in a [`Compositor`].
///
/// This is just a convenient super trait of the `raw-window-handle`
/// trait.
pub trait Display: HasDisplayHandle + Debug + Send + Sync + 'static {}

impl<T> Display for T where T: HasDisplayHandle + Debug + Send + Sync + 'static {}

/// Defines the default compositor of a renderer.
pub trait Default {
    /// The compositor of the renderer.
    type Compositor: Compositor<Renderer = Self>;
}

/// Result of an unsuccessful call to [`Compositor::present`].
#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum SurfaceError {
    /// A timeout was encountered while trying to acquire the next frame.
    #[error("A timeout was encountered while trying to acquire the next frame")]
    Timeout,
    /// The underlying surface has changed, and therefore the surface must be updated.
    #[error("The underlying surface has changed, and therefore the surface must be updated.")]
    Outdated,
    /// The swap chain has been lost and needs to be recreated.
    #[error("The surface has been lost and needs to be recreated")]
    Lost,
    /// There is no more memory left to allocate a new frame.
    #[error("There is no more memory left to allocate a new frame")]
    OutOfMemory,
    /// The surface is occluded and must not be drawn to.
    #[error("The surface is occluded and must not be drawn to")]
    Occluded,
    /// Acquiring a texture failed with a generic error.
    #[error("Acquiring a texture failed with a generic error")]
    Other,
}

/// Contains information about the graphics (e.g. graphics adapter, graphics backend).
#[derive(Debug)]
pub struct Information {
    /// Contains the graphics adapter.
    pub adapter: String,
    /// Contains the graphics backend.
    pub backend: String,
}

#[cfg(debug_assertions)]
impl Compositor for () {
    type Renderer = ();
    type Surface = ();

    fn backends(_preference: &Backend) -> Backends<Self> {
        Backends::new().push(Null)
    }

    fn create_renderer(&self, _settings: renderer::Settings) -> Self::Renderer {}

    fn create_surface(&mut self, _window: impl Window + Clone) -> Self::Surface {}

    fn configure_surface(&mut self, _surface: &mut Self::Surface, _width: u32, _height: u32) {}

    fn load_font(&mut self, _font: Cow<'static, [u8]>) -> Result<(), font::Error> {
        Ok(())
    }

    fn list_fonts(&mut self) -> Result<Vec<font::Family>, font::Error> {
        Ok(Vec::new())
    }

    fn information(&self) -> Information {
        Information {
            adapter: String::from("Null Renderer"),
            backend: String::from("Null"),
        }
    }

    fn present(
        &mut self,
        _renderer: &mut Self::Renderer,
        _surface: &mut Self::Surface,
        _viewport: &Viewport,
        _background_color: Color,
        _on_pre_present: impl FnOnce(),
    ) -> Result<(), SurfaceError> {
        Ok(())
    }

    fn screenshot(
        &mut self,
        _renderer: &mut Self::Renderer,
        _viewport: &Viewport,
        _background_color: Color,
    ) -> Vec<u8> {
        vec![]
    }
}

#[cfg(debug_assertions)]
impl Default for () {
    type Compositor = ();
}

/// The [`Choice`] of the null `()` compositor.
#[cfg(debug_assertions)]
#[derive(Debug)]
struct Null;

#[cfg(debug_assertions)]
impl Choice<()> for Null {
    fn construct<'a>(
        &'a self,
        _settings: &'a backend::Settings,
        _display: std::sync::Arc<dyn Display>,
        _compatible_window: std::sync::Arc<dyn Window>,
        _shell: crate::Shell,
    ) -> backends::Construction<'a, ()> {
        Box::pin(async { Ok(()) })
    }
}
