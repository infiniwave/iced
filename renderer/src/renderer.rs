use crate::core::image;
use crate::core::renderer;
use crate::core::{self, Background, Color, Point, Rectangle, Size, Transformation};
use crate::graphics;
use crate::graphics::compositor;
use crate::graphics::mesh;

#[cfg(feature = "geometry")]
pub use geometry::{Frame, Geometry};

/// The default graphics renderer for [`iced`].
///
/// It has one variant per enabled renderer.
///
/// [`iced`]: https://github.com/iced-rs/iced
pub enum Renderer {
    /// The `wgpu` renderer.
    #[cfg(feature = "wgpu-bare")]
    Wgpu(iced_wgpu::Renderer),
    /// The `vello-cpu` renderer.
    #[cfg(feature = "vello-cpu")]
    VelloCpu(iced_vello_cpu::Renderer),
}

/// The renderer that defines the shared associated types and constants.
#[cfg(feature = "wgpu-bare")]
type Primary = iced_wgpu::Renderer;

/// The renderer that defines the shared associated types and constants.
#[cfg(all(feature = "vello-cpu", not(feature = "wgpu-bare")))]
type Primary = iced_vello_cpu::Renderer;

impl core::Renderer for Renderer {
    fn fill_quad(&mut self, quad: renderer::Quad, background: impl Into<Background>) {
        delegate!(self, renderer => renderer.fill_quad(quad, background.into()));
    }

    fn start_layer(&mut self, bounds: Rectangle) {
        delegate!(self, renderer => renderer.start_layer(bounds));
    }

    fn end_layer(&mut self) {
        delegate!(self, renderer => renderer.end_layer());
    }

    fn start_transformation(&mut self, transformation: Transformation) {
        delegate!(self, renderer => renderer.start_transformation(transformation));
    }

    fn end_transformation(&mut self) {
        delegate!(self, renderer => renderer.end_transformation());
    }

    fn allocate_image(
        &self,
        handle: &image::Handle,
        callback: impl FnOnce(Result<image::Allocation, image::Error>) + Send + 'static,
    ) {
        delegate!(self, renderer => renderer.allocate_image(handle, callback));
    }

    fn hint(&mut self, scale: renderer::Scale) {
        delegate!(self, renderer => renderer.hint(scale));
    }

    fn scale(&self) -> Option<renderer::Scale> {
        delegate!(self, renderer => renderer.scale())
    }

    fn tick(&mut self) {
        delegate!(self, renderer => renderer.tick());
    }

    fn reset(&mut self, new_bounds: Rectangle) {
        delegate!(self, renderer => renderer.reset(new_bounds));
    }

    fn settings(&self) -> renderer::Settings {
        delegate!(self, renderer => renderer.settings())
    }
}

impl core::text::Renderer for Renderer {
    type Paragraph = <Primary as core::text::Renderer>::Paragraph;
    type Editor = <Primary as core::text::Renderer>::Editor;

    const ICON_FONT: core::Font = Primary::ICON_FONT;
    const CHECKMARK_ICON: char = Primary::CHECKMARK_ICON;
    const ARROW_DOWN_ICON: char = Primary::ARROW_DOWN_ICON;
    const SCROLL_UP_ICON: char = Primary::SCROLL_UP_ICON;
    const SCROLL_DOWN_ICON: char = Primary::SCROLL_DOWN_ICON;
    const SCROLL_LEFT_ICON: char = Primary::SCROLL_LEFT_ICON;
    const SCROLL_RIGHT_ICON: char = Primary::SCROLL_RIGHT_ICON;
    const ICED_LOGO: char = Primary::ICED_LOGO;

    fn fill_paragraph(
        &mut self,
        text: &Self::Paragraph,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        delegate!(self, renderer => renderer.fill_paragraph(text, position, color, clip_bounds));
    }

    fn fill_editor(
        &mut self,
        editor: &Self::Editor,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        delegate!(self, renderer => renderer.fill_editor(editor, position, color, clip_bounds));
    }

    fn fill_text(
        &mut self,
        text: core::Text<String>,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        delegate!(self, renderer => renderer.fill_text(text, position, color, clip_bounds));
    }
}

#[cfg(feature = "wgpu-bare")]
impl graphics::text::Renderer for Renderer {
    fn fill_raw(&mut self, raw: graphics::text::Raw) {
        match self {
            Self::Wgpu(renderer) => renderer.fill_raw(raw),
            #[cfg(feature = "vello-cpu")]
            Self::VelloCpu(_) => {
                log::warn!("Raw text is not supported with this renderer.");
            }
        }
    }
}

#[cfg(feature = "image")]
impl image::Renderer for Renderer {
    type Handle = <Primary as image::Renderer>::Handle;

    fn load_image(&self, handle: &Self::Handle) -> Result<image::Allocation, image::Error> {
        delegate!(self, renderer => renderer.load_image(handle))
    }

    fn measure_image(&self, handle: &Self::Handle) -> Option<Size<u32>> {
        delegate!(self, renderer => renderer.measure_image(handle))
    }

    fn draw_image(
        &mut self,
        image: core::Image<Self::Handle>,
        bounds: Rectangle,
        clip_bounds: Rectangle,
    ) {
        delegate!(self, renderer => renderer.draw_image(image, bounds, clip_bounds));
    }
}

#[cfg(feature = "svg")]
impl core::svg::Renderer for Renderer {
    fn measure_svg(&self, handle: &core::svg::Handle) -> Size<u32> {
        delegate!(self, renderer => renderer.measure_svg(handle))
    }

    fn draw_svg(&mut self, svg: core::Svg, bounds: Rectangle, clip_bounds: Rectangle) {
        delegate!(self, renderer => renderer.draw_svg(svg, bounds, clip_bounds));
    }
}

impl mesh::Renderer for Renderer {
    fn draw_mesh(&mut self, mesh: graphics::Mesh) {
        delegate!(self, renderer => renderer.draw_mesh(mesh));
    }

    fn draw_mesh_cache(&mut self, cache: mesh::Cache) {
        delegate!(self, renderer => renderer.draw_mesh_cache(cache));
    }
}

#[cfg(feature = "wgpu-bare")]
impl iced_wgpu::primitive::Renderer for Renderer {
    fn draw_primitive(&mut self, bounds: Rectangle, primitive: impl iced_wgpu::Primitive) {
        match self {
            Self::Wgpu(renderer) => {
                renderer.draw_primitive(bounds, primitive);
            }
            #[cfg(feature = "vello-cpu")]
            Self::VelloCpu(_) => {
                log::warn!("Custom shader primitive is not supported with this renderer.");
            }
        }
    }
}

impl renderer::Headless for Renderer {
    async fn new(settings: renderer::Settings, backend: Option<&str>) -> Option<Self> {
        #[cfg(feature = "wgpu-bare")]
        if let Some(renderer) =
            <iced_wgpu::Renderer as renderer::Headless>::new(settings, backend).await
        {
            return Some(Self::Wgpu(renderer));
        }

        #[cfg(feature = "vello-cpu")]
        if let Some(renderer) =
            <iced_vello_cpu::Renderer as renderer::Headless>::new(settings, backend).await
        {
            return Some(Self::VelloCpu(renderer));
        }

        None
    }

    fn name(&self) -> String {
        delegate!(self, renderer => renderer.name())
    }

    fn screenshot(
        &mut self,
        size: Size<u32>,
        scale_factor: f32,
        background_color: Color,
    ) -> Vec<u8> {
        delegate!(self, renderer => renderer::Headless::screenshot(
            renderer,
            size,
            scale_factor,
            background_color
        ))
    }
}

impl compositor::Default for Renderer {
    type Compositor = crate::Compositor;
}

#[cfg(feature = "geometry")]
mod geometry {
    use super::Renderer;
    use crate::core::{Point, Radians, Rectangle, Size, Svg, Vector};
    use crate::graphics::cache::{self, Cached};
    use crate::graphics::geometry::{self, Fill, Image, Path, Stroke, Text};

    impl geometry::Renderer for Renderer {
        type Geometry = Geometry;
        type Frame = Frame;

        fn new_frame(&self, bounds: Rectangle) -> Frame {
            match self {
                #[cfg(feature = "wgpu-bare")]
                Self::Wgpu(renderer) => Frame::Wgpu(renderer.new_frame(bounds)),
                #[cfg(feature = "vello-cpu")]
                Self::VelloCpu(renderer) => Frame::VelloCpu(renderer.new_frame(bounds)),
            }
        }

        fn draw_geometry(&mut self, geometry: Geometry) {
            match (self, geometry) {
                #[cfg(feature = "wgpu-bare")]
                (Self::Wgpu(renderer), Geometry::Wgpu(geometry)) => {
                    renderer.draw_geometry(geometry);
                }
                #[cfg(feature = "vello-cpu")]
                (Self::VelloCpu(renderer), Geometry::VelloCpu(geometry)) => {
                    renderer.draw_geometry(geometry);
                }
                #[allow(unreachable_patterns)]
                _ => unreachable!("Renderer and geometry variants do not match"),
            }
        }
    }

    /// The geometry of the default [`Renderer`].
    #[derive(Debug)]
    pub enum Geometry {
        /// `wgpu` geometry.
        #[cfg(feature = "wgpu-bare")]
        Wgpu(<iced_wgpu::Renderer as geometry::Renderer>::Geometry),
        /// `vello-cpu` geometry.
        #[cfg(feature = "vello-cpu")]
        VelloCpu(<iced_vello_cpu::Renderer as geometry::Renderer>::Geometry),
    }

    /// The cache of some [`Geometry`].
    #[derive(Debug)]
    pub enum Cache {
        /// A `wgpu` geometry cache.
        #[cfg(feature = "wgpu-bare")]
        Wgpu(<<iced_wgpu::Renderer as geometry::Renderer>::Geometry as Cached>::Cache),
        /// A `vello-cpu` geometry cache.
        #[cfg(feature = "vello-cpu")]
        VelloCpu(<<iced_vello_cpu::Renderer as geometry::Renderer>::Geometry as Cached>::Cache),
    }

    impl Clone for Cache {
        fn clone(&self) -> Self {
            match self {
                #[cfg(feature = "wgpu-bare")]
                Self::Wgpu(cache) => Self::Wgpu(cache.clone()),
                #[cfg(feature = "vello-cpu")]
                Self::VelloCpu(cache) => Self::VelloCpu(cache.clone()),
            }
        }
    }

    impl Cached for Geometry {
        type Cache = Cache;

        fn load(cache: &Cache) -> Self {
            match cache {
                #[cfg(feature = "wgpu-bare")]
                Cache::Wgpu(cache) => Self::Wgpu(Cached::load(cache)),
                #[cfg(feature = "vello-cpu")]
                Cache::VelloCpu(cache) => Self::VelloCpu(Cached::load(cache)),
            }
        }

        fn cache(self, group: cache::Group, previous: Option<Cache>) -> Cache {
            match (self, previous) {
                #[cfg(feature = "wgpu-bare")]
                (Self::Wgpu(geometry), Some(Cache::Wgpu(previous))) => {
                    Cache::Wgpu(geometry.cache(group, Some(previous)))
                }
                #[cfg(feature = "wgpu-bare")]
                (Self::Wgpu(geometry), None) => Cache::Wgpu(geometry.cache(group, None)),
                #[cfg(feature = "vello-cpu")]
                (Self::VelloCpu(geometry), Some(Cache::VelloCpu(previous))) => {
                    Cache::VelloCpu(geometry.cache(group, Some(previous)))
                }
                #[cfg(feature = "vello-cpu")]
                (Self::VelloCpu(geometry), None) => Cache::VelloCpu(geometry.cache(group, None)),
                #[allow(unreachable_patterns)]
                _ => unreachable!("Geometry and cache variants do not match"),
            }
        }
    }

    /// The frame of the default [`Renderer`].
    pub enum Frame {
        /// A `wgpu` frame.
        #[cfg(feature = "wgpu-bare")]
        Wgpu(<iced_wgpu::Renderer as geometry::Renderer>::Frame),
        /// A `vello-cpu` frame.
        #[cfg(feature = "vello-cpu")]
        VelloCpu(<iced_vello_cpu::Renderer as geometry::Renderer>::Frame),
    }

    impl geometry::frame::Backend for Frame {
        type Geometry = Geometry;

        fn width(&self) -> f32 {
            delegate!(self, frame => frame.width())
        }

        fn height(&self) -> f32 {
            delegate!(self, frame => frame.height())
        }

        fn size(&self) -> Size {
            delegate!(self, frame => frame.size())
        }

        fn center(&self) -> Point {
            delegate!(self, frame => frame.center())
        }

        fn fill(&mut self, path: &Path, fill: impl Into<Fill>) {
            delegate!(self, frame => frame.fill(path, fill));
        }

        fn fill_rectangle(&mut self, top_left: Point, size: Size, fill: impl Into<Fill>) {
            delegate!(self, frame => frame.fill_rectangle(top_left, size, fill));
        }

        fn stroke<'a>(&mut self, path: &Path, stroke: impl Into<Stroke<'a>>) {
            delegate!(self, frame => frame.stroke(path, stroke));
        }

        fn stroke_rectangle<'a>(
            &mut self,
            top_left: Point,
            size: Size,
            stroke: impl Into<Stroke<'a>>,
        ) {
            delegate!(self, frame => frame.stroke_rectangle(top_left, size, stroke));
        }

        fn stroke_text<'a>(&mut self, text: impl Into<Text>, stroke: impl Into<Stroke<'a>>) {
            delegate!(self, frame => frame.stroke_text(text, stroke));
        }

        fn fill_text(&mut self, text: impl Into<Text>) {
            delegate!(self, frame => frame.fill_text(text));
        }

        fn draw_image(&mut self, bounds: Rectangle, image: impl Into<Image>) {
            delegate!(self, frame => frame.draw_image(bounds, image));
        }

        fn draw_svg(&mut self, bounds: Rectangle, svg: impl Into<Svg>) {
            delegate!(self, frame => frame.draw_svg(bounds, svg));
        }

        fn push_transform(&mut self) {
            delegate!(self, frame => frame.push_transform());
        }

        fn pop_transform(&mut self) {
            delegate!(self, frame => frame.pop_transform());
        }

        fn draft(&mut self, bounds: Rectangle) -> Self {
            match self {
                #[cfg(feature = "wgpu-bare")]
                Self::Wgpu(frame) => Self::Wgpu(frame.draft(bounds)),
                #[cfg(feature = "vello-cpu")]
                Self::VelloCpu(frame) => Self::VelloCpu(frame.draft(bounds)),
            }
        }

        fn paste(&mut self, frame: Self) {
            match (self, frame) {
                #[cfg(feature = "wgpu-bare")]
                (Self::Wgpu(target), Self::Wgpu(source)) => target.paste(source),
                #[cfg(feature = "vello-cpu")]
                (Self::VelloCpu(target), Self::VelloCpu(source)) => target.paste(source),
                #[allow(unreachable_patterns)]
                _ => unreachable!("Frame variants do not match"),
            }
        }

        fn translate(&mut self, translation: Vector) {
            delegate!(self, frame => frame.translate(translation));
        }

        fn rotate(&mut self, angle: impl Into<Radians>) {
            delegate!(self, frame => frame.rotate(angle));
        }

        fn scale(&mut self, scale: f32) {
            delegate!(self, frame => frame.scale(scale));
        }

        fn scale_nonuniform(&mut self, scale: impl Into<Vector>) {
            delegate!(self, frame => frame.scale_nonuniform(scale));
        }

        fn into_geometry(self) -> Geometry {
            match self {
                #[cfg(feature = "wgpu-bare")]
                Self::Wgpu(frame) => Geometry::Wgpu(frame.into_geometry()),
                #[cfg(feature = "vello-cpu")]
                Self::VelloCpu(frame) => Geometry::VelloCpu(frame.into_geometry()),
            }
        }
    }
}
