#[allow(unused_imports)]
pub(crate) mod prelude {
    pub(crate) use super::BindGroupCache;
}

use bevy::render::render_resource::{BindGroup, TextureViewId};

/// [`BindGroup`]s cached and identified via [`TextureViewId`].
#[derive(Default)]
pub(crate) struct BindGroupCache(pub(crate) Option<(TextureViewId, BindGroup)>);
