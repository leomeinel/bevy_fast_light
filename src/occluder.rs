//! Different occluders and modules for rendering.
//!
//! This renders to a texture that uses the red channel for determining if an occluder exists and the green channel for its z-level.
//!
//! This is the second render stage of [`FastLightPlugin`](crate::prelude::FastLightPlugin).

mod phase;
mod pipeline;
mod plugin;
mod prepare;
mod system;

pub(super) mod prelude {
    pub(crate) use super::MeshOccluder;
    pub(crate) use super::phase::OccluderPhase;
    pub(super) use super::phase::{DrawOccluder, PendingOccluderQueues};
    pub(super) use super::pipeline::OccluderPipeline;
    pub(crate) use super::plugin::{OccluderPlugin, OccluderSet};
    pub(crate) use super::prepare::OccluderTextures;
}

use bevy::{ecs::component::Component, render::extract_component::ExtractComponent};

/// Light occluder for 2D environments.
///
/// This is meant to be added to a [`Mesh2d`](bevy::mesh::Mesh2d) which will determine the occluded shape.
///
/// Currently this will occlude all non-ambient light.
///
/// ## Special Case
///
/// This will render [`Sprite`](bevy::sprite::Sprite)s on top of the occluder if they are on the same or a higher z-level which allows the usage as a 2d shadow.
#[derive(Component, ExtractComponent, Clone, Copy, Default)]
pub struct MeshOccluder;
