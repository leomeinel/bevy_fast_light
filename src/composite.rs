//! Modules for rendering composition.
//!
//! This composes the light map from [`MeshLightTextures`](crate::light::prelude::MeshLightTextures) to the screen texture.
//!
//! This is the fourth render stage of [`FastLightPlugin`](crate::prelude::FastLightPlugin).

mod pipeline;
mod plugin;
mod system;

pub(super) mod prelude {
    pub(super) use super::pipeline::CompositePipeline;
    pub(crate) use super::plugin::CompositePlugin;
}
