//! Extraction

pub(super) mod prelude {
    pub(crate) use super::{ExtractedAmbientLight2d, ExtractedMeshLight};
}

use bevy::{
    camera::{Camera, Camera2d},
    color::{Alpha as _, LinearRgba},
    ecs::{
        component::Component,
        entity::Entity,
        lifecycle::RemovedComponents,
        query::{Changed, With},
        system::{Commands, Local, Query, ResMut, Single},
    },
    mesh::Mesh2d,
    platform::collections::HashSet,
    render::{
        Extract,
        batching::gpu_preprocessing::GpuPreprocessingMode,
        render_phase::{ViewBinnedRenderPhases, ViewSortedRenderPhases},
        render_resource::ShaderType,
        sync_world::RenderEntity,
        view::RetainedViewEntity,
    },
};
use bytemuck::{Pod, Zeroable};

use crate::{light::prelude::*, occluder::prelude::*, sprite_depth::prelude::*};

/// [`ShaderType`] that gets extracted to the render world for [`AmbientLight2d`].
#[derive(Component, Default, Clone, Copy, ShaderType, Debug)]
pub(crate) struct ExtractedAmbientLight2d {
    color: LinearRgba,
}
impl From<AmbientLight2d> for ExtractedAmbientLight2d {
    fn from(light: AmbientLight2d) -> Self {
        Self {
            color: (light.color.to_linear() * light.intensity).with_alpha(1.),
        }
    }
}

/// [`ShaderType`] that gets extracted to the render world for [`MeshLight`].
#[repr(C)]
#[derive(Component, Default, Clone, Copy, ShaderType, Debug, Pod, Zeroable)]
pub(crate) struct ExtractedMeshLight {
    pub(super) color: LinearRgba,
}
impl From<MeshLight> for ExtractedMeshLight {
    fn from(light: MeshLight) -> Self {
        Self {
            color: (light.color.to_linear() * light.intensity).with_alpha(1.),
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

/// Extract [`MeshLight`] as [`ExtractedMeshLight`] to render world.
pub(super) fn extract_mesh_lights(
    mut removed_lights: Extract<RemovedComponents<MeshLight>>,
    light_query: Extract<Query<(&RenderEntity, &MeshLight), With<Mesh2d>>>,
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
            .remove::<ExtractedMeshLight>();
    }

    // Insert new extracted components
    for (render_entity, light) in &light_query {
        commands
            .entity(**render_entity)
            .insert(ExtractedMeshLight::from(*light));
    }
}

/// Extract [`RetainedViewEntity`]s to relevant render phases.
pub(super) fn extract_view_entities(
    mut sprite_depth_phases: ResMut<ViewSortedRenderPhases<SpriteDepthPhase>>,
    mut occluder_phases: ResMut<ViewBinnedRenderPhases<OccluderPhase>>,
    mut light_phases: ResMut<ViewBinnedRenderPhases<MeshLightPhase>>,
    cameras: Extract<Query<(Entity, &Camera), (With<Camera2d>, With<AmbientLight2d>)>>,
    mut live_entities: Local<HashSet<RetainedViewEntity>>,
) {
    live_entities.clear();
    for (main_entity, camera) in &cameras {
        if !camera.is_active {
            continue;
        }
        // NOTE: This is the main camera, so we use the first subview index (0)
        let retained_view_entity = RetainedViewEntity::new(main_entity.into(), None, 0);
        sprite_depth_phases.prepare_for_new_frame(retained_view_entity);
        occluder_phases.prepare_for_new_frame(retained_view_entity, GpuPreprocessingMode::None);
        light_phases.prepare_for_new_frame(retained_view_entity, GpuPreprocessingMode::None);
        live_entities.insert(retained_view_entity);
    }

    sprite_depth_phases.retain(|camera_entity, _| live_entities.contains(camera_entity));
    occluder_phases.retain(|camera_entity, _| live_entities.contains(camera_entity));
    light_phases.retain(|camera_entity, _| live_entities.contains(camera_entity));
}
