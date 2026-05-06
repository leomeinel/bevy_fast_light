/*
 * Heavily inspired by:
 * - https://github.com/jgayfer/bevy_light_2d
 */

//! Extracted [`Component`]s and systems for extraction to the render world.

use bevy::{
    camera::Camera2d,
    color::{Alpha, LinearRgba},
    ecs::{
        component::Component,
        lifecycle::RemovedComponents,
        query::{Changed, With},
        system::{Commands, Query, Single},
    },
    mesh::Mesh2d,
    render::{Extract, render_resource::ShaderType, sync_world::RenderEntity},
    utils::default,
};
use bytemuck::{Pod, Zeroable};

use crate::light::prelude::*;

/// [`ShaderType`] that gets extracted to the render world for [`AmbientLight2d`].
#[derive(Component, Default, Clone, Copy, ShaderType, Debug)]
pub(crate) struct ExtractedAmbientLight2d {
    color: LinearRgba,
}
impl From<AmbientLight2d> for ExtractedAmbientLight2d {
    fn from(light: AmbientLight2d) -> Self {
        Self {
            color: (light.color.to_linear() * light.intensity).with_alpha(1.),
            ..default()
        }
    }
}

/// [`ShaderType`] that gets extracted to the render world for [`MeshLight2d`].
#[repr(C)]
#[derive(Component, Default, Clone, Copy, ShaderType, Debug, Pod, Zeroable)]
pub(crate) struct ExtractedMeshLight2d {
    pub(super) color: LinearRgba,
}
impl From<MeshLight2d> for ExtractedMeshLight2d {
    fn from(light: MeshLight2d) -> Self {
        Self {
            color: (light.color.to_linear() * light.intensity).with_alpha(1.),
            ..default()
        }
    }
}

/// Extract [`AmbientLight2d`] as [`ExtractedAmbientLight2d`] to render world.
pub(super) fn extract_ambient_light(
    mut removed_ambient: Extract<RemovedComponents<AmbientLight2d>>,
    ambient: Extract<
        Single<(&RenderEntity, &AmbientLight2d), (Changed<AmbientLight2d>, With<Camera2d>)>,
    >,
    render_entity_query: Extract<Query<&RenderEntity>>,
    mut commands: Commands,
) {
    // Remove old extracted components
    for entity in removed_ambient.read() {
        let Ok(render_entity) = render_entity_query.get(entity) else {
            continue;
        };
        commands
            .entity(**render_entity)
            .remove::<ExtractedAmbientLight2d>();
    }

    // Insert new extracted component
    let (render_entity, ambient) = **ambient;
    commands
        .entity(**render_entity)
        .insert(ExtractedAmbientLight2d::from(*ambient));
}

/// Extract [`MeshLight2d`] as [`ExtractedMeshLight2d`] to render world.
pub(super) fn extract_mesh_lights(
    mut removed_lights: Extract<RemovedComponents<MeshLight2d>>,
    light_query: Extract<Query<(&RenderEntity, &MeshLight2d), With<Mesh2d>>>,
    render_entity_query: Extract<Query<&RenderEntity>>,
    mut commands: Commands,
) {
    // Remove old extracted components
    for entity in removed_lights.read() {
        let Ok(render_entity) = render_entity_query.get(entity) else {
            continue;
        };
        commands
            .entity(**render_entity)
            .remove::<ExtractedMeshLight2d>();
    }

    // Insert new extracted components
    for (render_entity, light) in &light_query {
        commands
            .entity(**render_entity)
            .insert(ExtractedMeshLight2d::from(*light));
    }
}
