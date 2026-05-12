//! Light types and modules for rendering [`MeshLight`].
//!
//! This renders a light map to a texture.
//!
//! This is the third render stage of [`FastLightPlugin`](crate::prelude::FastLightPlugin).
//!
//! ## Note
//!
//! This also contains [`AmbientLight2d`] but modules for rendering that are located in [composite](crate::composite).

mod node;
mod phase;
mod pipeline;
mod plugin;
mod prepare;

pub(super) mod prelude {
    pub(super) use super::node::MeshLightNode;
    pub(super) use super::phase::DrawMeshLight;
    pub(crate) use super::phase::MeshLightPhase;
    pub(super) use super::pipeline::MeshLightPipeline;
    pub(crate) use super::plugin::{MeshLightLabel, MeshLightPlugin};
    pub(crate) use super::prepare::MeshLightTextures;
    pub(super) use super::prepare::MeshLightUniformBuffers;
    pub(crate) use super::{AmbientLight2d, MeshLight};
    pub(super) use super::{MeshLightFragmentBindGroups, SetMeshLightFragmentBindGroup};
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
        view::{ExtractedView, RetainedViewEntity},
    },
};

use crate::extract::prelude::*;

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
/// color = src_color * (ambient_color + [`MeshLight::color`] * [`MeshLight::intensity`] * attenuation).
///
/// ## Note
///
/// `attenuation` decreases smoothly from the center outwards.
#[derive(Component, Reflect, Clone, Copy)]
#[require(SyncToRenderWorld)]
pub struct MeshLight {
    /// The [`Color`] of the light.
    pub color: Color,
    /// The intensity of the light.
    pub intensity: f32,
}
impl Default for MeshLight {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.,
        }
    }
}

/// [`BindGroup`]s mapped to [`MeshLight`] [`Entity`]s.
#[derive(Resource, Default)]
pub(super) struct MeshLightFragmentBindGroups(
    pub(super) HashMap<(RetainedViewEntity, Entity), BindGroup>,
);

/// Set [`BindGroup`]s from [`MeshLightFragmentBindGroups`] for [`DrawMeshLight`](crate::light::prelude::DrawMeshLight).
pub(super) struct SetMeshLightFragmentBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetMeshLightFragmentBindGroup<I> {
    type Param = SRes<MeshLightFragmentBindGroups>;
    type ViewQuery = (&'static ExtractedView, &'static ExtractedAmbientLight2d);
    type ItemQuery = ();

    fn render<'w>(
        item: &P,
        (view, _): ROQueryItem<'w, '_, Self::ViewQuery>,
        _entity: Option<()>,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let bind_groups = bind_groups.into_inner();
        let Some(bind_group) = bind_groups
            .0
            .get(&(view.retained_view_entity, item.entity()))
        else {
            return RenderCommandResult::Skip;
        };

        pass.set_bind_group(I, &bind_group, &[]);
        RenderCommandResult::Success
    }
}
