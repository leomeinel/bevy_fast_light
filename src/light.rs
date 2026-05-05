//! Different light types and modules for rendering.
//!
//! This first renders a light map to a scalable texture and then composes this to the screen texture.
//!
//! This is the third and fourth render stage of [`FastLightPlugin`](crate::prelude::FastLightPlugin).

mod extract;
mod node;
mod phase;
mod pipeline;
mod plugin;
mod prepare;

pub(super) mod prelude {
    pub(super) use super::extract::{ExtractedAmbientLight2d, ExtractedMeshLight2d};
    pub(super) use super::node::{Light2dCompositeNode, Light2dNode};
    pub(super) use super::phase::{DrawLight2d, Light2dPhase};
    pub(super) use super::pipeline::{Light2dCompositePipeline, Light2dPipeline};
    pub(crate) use super::plugin::{Light2dCompositeLabel, Light2dLabel, Light2dPlugin};
    pub(super) use super::prepare::{Light2dTextures, Light2dUniformBuffers};
    pub(crate) use super::{AmbientLight2d, MeshLight2d};
    pub(super) use super::{Light2dFragmentBindGroups, SetLight2dFragmentBindGroup};
}

use bevy::{
    color::Color,
    ecs::{
        component::Component,
        entity::Entity,
        query::ROQueryItem,
        resource::Resource,
        system::{SystemParamItem, lifetimeless::SRes},
    },
    platform::collections::HashMap,
    reflect::Reflect,
    render::{
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::BindGroup,
        sync_world::SyncToRenderWorld,
        view::ExtractedView,
    },
};

/// Ambient light for fullscreen lighting in a 2D environment.
///
/// This is meant to be added to a [`Camera2d`](bevy::camera::Camera2d).
///
/// ## Formula
///
/// color = src_color * ([`AmbientLight2d::color`] * [`AmbientLight2d::intensity`] + light_color).
///
/// ## Note
///
/// Only a single [`AmbientLight2d`] should ever exist, otherwise extraction will be skipped.
#[derive(Component, Reflect, Clone, Copy)]
#[require(SyncToRenderWorld)]
pub struct AmbientLight2d {
    /// The [`Color`] of the light.
    pub color: Color,
    /// The intensity of the light.
    pub intensity: f32,
}
impl Default for AmbientLight2d {
    /// This is visually the same as without [`AmbientLight2d`].
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.,
        }
    }
}

/// Mesh light for area lighting in a 2D environment.
///
/// This is meant to be added to a [`Mesh2d`](bevy::mesh::Mesh2d) which will determine the lit shape.
///
/// The lit shape is an ellipse in the center that is influenced by the width and height of the [`Mesh2d`](bevy::mesh::Mesh2d). Therefore in most [`Mesh2d`](bevy::mesh::Mesh2d)s, the corners will not be lit.
///
/// ## Formula
///
/// color = src_color * (ambient_color + [`MeshLight2d::color`] * [`MeshLight2d::intensity`] * attenuation).
///
/// ## Note
///
/// attenuation decreases smoothly from the center outwards.
#[derive(Component, Reflect, Clone, Copy)]
#[require(SyncToRenderWorld)]
pub struct MeshLight2d {
    /// The [`Color`] of the light.
    pub color: Color,
    /// The intensity of the light.
    pub intensity: f32,
}
impl Default for MeshLight2d {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.,
        }
    }
}

/// [`BindGroup`]s mapped to [`MeshLight2d`] [`Entity`]s.
#[derive(Resource, Default)]
pub(super) struct Light2dFragmentBindGroups(pub(super) HashMap<Entity, BindGroup>);

/// Set [`BindGroup`]s from [`Light2dFragmentBindGroups`] for [`DrawLight2d`](crate::light::prelude::DrawLight2d).
pub(super) struct SetLight2dFragmentBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetLight2dFragmentBindGroup<I> {
    type Param = SRes<Light2dFragmentBindGroups>;
    type ViewQuery = &'static ExtractedView;
    type ItemQuery = ();

    fn render<'w>(
        item: &P,
        _view: ROQueryItem<'w, '_, Self::ViewQuery>,
        _entity: Option<()>,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let bind_groups = bind_groups.into_inner();
        let Some(bind_group) = bind_groups.0.get(&item.entity()) else {
            return RenderCommandResult::Skip;
        };

        pass.set_bind_group(I, &bind_group, &[]);
        RenderCommandResult::Success
    }
}
