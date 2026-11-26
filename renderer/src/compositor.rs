use crate::Renderer;
use crate::core::backend::Backend;
use crate::core::{Color, font, renderer};
use crate::graphics::{self, compositor};

use std::borrow::Cow;

/// The default graphics compositor for [`iced`].
///
/// It has one variant per enabled renderer.
///
/// [`iced`]: https://github.com/iced-rs/iced
pub enum Compositor {
    /// The `wgpu` compositor.
    #[cfg(feature = "wgpu-bare")]
    Wgpu(iced_wgpu::window::Compositor),
    /// The `vello-cpu` compositor.
    #[cfg(feature = "vello-cpu")]
    VelloCpu(iced_vello_cpu::Compositor),
}

/// The surface of the default [`Compositor`].
pub enum Surface {
    /// A `wgpu` surface.
    #[cfg(feature = "wgpu-bare")]
    Wgpu(<iced_wgpu::window::Compositor as graphics::Compositor>::Surface),
    /// A `vello-cpu` surface.
    #[cfg(feature = "vello-cpu")]
    VelloCpu(<iced_vello_cpu::Compositor as graphics::Compositor>::Surface),
}

#[cfg(feature = "wgpu-bare")]
impl From<iced_wgpu::window::Compositor> for Compositor {
    fn from(compositor: iced_wgpu::window::Compositor) -> Self {
        Self::Wgpu(compositor)
    }
}

#[cfg(feature = "vello-cpu")]
impl From<iced_vello_cpu::Compositor> for Compositor {
    fn from(compositor: iced_vello_cpu::Compositor) -> Self {
        Self::VelloCpu(compositor)
    }
}

impl graphics::Compositor for Compositor {
    type Renderer = Renderer;
    type Surface = Surface;

    fn backends(preference: &Backend) -> compositor::Backends<Self> {
        let backends = compositor::Backends::new();

        #[cfg(feature = "wgpu-bare")]
        let backends = backends.extend(
            <iced_wgpu::window::Compositor as graphics::Compositor>::backends(preference).map(),
        );

        #[cfg(feature = "vello-cpu")]
        let backends = backends.extend(
            <iced_vello_cpu::Compositor as graphics::Compositor>::backends(preference).map(),
        );

        backends
    }

    fn create_renderer(&self, settings: renderer::Settings) -> Renderer {
        match self {
            #[cfg(feature = "wgpu-bare")]
            Self::Wgpu(compositor) => Renderer::Wgpu(compositor.create_renderer(settings)),
            #[cfg(feature = "vello-cpu")]
            Self::VelloCpu(compositor) => Renderer::VelloCpu(compositor.create_renderer(settings)),
        }
    }

    fn create_surface(&mut self, window: impl compositor::Window + Clone) -> Surface {
        match self {
            #[cfg(feature = "wgpu-bare")]
            Self::Wgpu(compositor) => Surface::Wgpu(compositor.create_surface(window)),
            #[cfg(feature = "vello-cpu")]
            Self::VelloCpu(compositor) => Surface::VelloCpu(compositor.create_surface(window)),
        }
    }

    fn configure_surface(&mut self, surface: &mut Surface, width: u32, height: u32) {
        match (self, surface) {
            #[cfg(feature = "wgpu-bare")]
            (Self::Wgpu(compositor), Surface::Wgpu(surface)) => {
                compositor.configure_surface(surface, width, height);
            }
            #[cfg(feature = "vello-cpu")]
            (Self::VelloCpu(compositor), Surface::VelloCpu(surface)) => {
                compositor.configure_surface(surface, width, height);
            }
            #[allow(unreachable_patterns)]
            _ => unreachable!("Compositor and surface variants do not match"),
        }
    }

    fn load_font(&mut self, font: Cow<'static, [u8]>) -> Result<(), font::Error> {
        delegate!(self, compositor => compositor.load_font(font))
    }

    fn list_fonts(&mut self) -> Result<Vec<font::Family>, font::Error> {
        delegate!(self, compositor => compositor.list_fonts())
    }

    fn information(&self) -> compositor::Information {
        delegate!(self, compositor => compositor.information())
    }

    fn present(
        &mut self,
        renderer: &mut Renderer,
        surface: &mut Surface,
        viewport: &graphics::Viewport,
        background_color: Color,
        on_pre_present: impl FnOnce(),
    ) -> Result<(), compositor::SurfaceError> {
        match (self, renderer, surface) {
            #[cfg(feature = "wgpu-bare")]
            (Self::Wgpu(compositor), Renderer::Wgpu(renderer), Surface::Wgpu(surface)) => {
                compositor.present(
                    renderer,
                    surface,
                    viewport,
                    background_color,
                    on_pre_present,
                )
            }
            #[cfg(feature = "vello-cpu")]
            (
                Self::VelloCpu(compositor),
                Renderer::VelloCpu(renderer),
                Surface::VelloCpu(surface),
            ) => compositor.present(
                renderer,
                surface,
                viewport,
                background_color,
                on_pre_present,
            ),
            #[allow(unreachable_patterns)]
            _ => unreachable!("Compositor, renderer, and surface variants do not match"),
        }
    }

    fn screenshot(
        &mut self,
        renderer: &mut Renderer,
        viewport: &graphics::Viewport,
        background_color: Color,
    ) -> Vec<u8> {
        match (self, renderer) {
            #[cfg(feature = "wgpu-bare")]
            (Self::Wgpu(compositor), Renderer::Wgpu(renderer)) => {
                compositor.screenshot(renderer, viewport, background_color)
            }
            #[cfg(feature = "vello-cpu")]
            (Self::VelloCpu(compositor), Renderer::VelloCpu(renderer)) => {
                compositor.screenshot(renderer, viewport, background_color)
            }
            #[allow(unreachable_patterns)]
            _ => unreachable!("Compositor and renderer variants do not match"),
        }
    }
}
