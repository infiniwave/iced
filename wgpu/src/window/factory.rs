//! Inject a graphics adapter and/or device.
use std::fmt;
use std::sync::Arc;

/// A boxed future returned by a [`Factory`].
#[cfg(not(target_arch = "wasm32"))]
pub type Future<'a, T> = futures::future::BoxFuture<'a, T>;

/// A boxed future returned by a [`Factory`].
#[cfg(target_arch = "wasm32")]
pub type Future<'a, T> = futures::future::LocalBoxFuture<'a, T>;

/// An error produced by a [`Factory`].
pub type Error = Arc<dyn std::error::Error + Send + Sync>;

/// A strategy to choose the [`wgpu::Adapter`] and create the [`wgpu::Device`]
/// that a [`Compositor`](super::Compositor) will render with.
pub trait Factory: Send + Sync + 'static {
    /// Chooses the [`wgpu::Adapter`] to use.
    ///
    /// The adapter must come from the given [`wgpu::Instance`] and, if a
    /// surface is provided, it should be able to present to it.
    ///
    /// Returning `None` lets the [`Compositor`](super::Compositor) choose one as usual.
    fn request_adapter<'a>(
        &'a self,
        _instance: &'a wgpu::Instance,
        _compatible_surface: Option<&'a wgpu::Surface<'static>>,
    ) -> Future<'a, Option<wgpu::Adapter>> {
        Box::pin(async { None })
    }

    /// Creates the [`wgpu::Device`] and [`wgpu::Queue`] to use.
    ///
    /// The given [`wgpu::DeviceDescriptor`] contains what the [`Compositor`](super::Compositor)
    /// needs. The device may enable more features, or have higher limits, but
    /// never fewer.
    ///
    /// This may be called more than once with decreasing limits.
    fn request_device<'a>(
        &'a self,
        adapter: &'a wgpu::Adapter,
        descriptor: &'a wgpu::DeviceDescriptor<'a>,
    ) -> Future<'a, Result<(wgpu::Device, wgpu::Queue), Error>> {
        Box::pin(async move {
            adapter
                .request_device(descriptor)
                .await
                .map_err(|error| Arc::new(error) as Error)
        })
    }
}

/// A shared [`Factory`].
#[derive(Clone)]
pub struct Shared(pub(crate) Arc<dyn Factory>);

impl Shared {
    /// Creates a new [`Shared`] factory.
    pub fn new(factory: impl Factory) -> Self {
        Self(Arc::new(factory))
    }
}

impl fmt::Debug for Shared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Factory")
    }
}

impl PartialEq for Shared {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
