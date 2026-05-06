use bevy::{
    camera::{Camera, Camera2d},
    ecs::{
        entity::Entity,
        query::With,
        system::{Local, Query, ResMut},
    },
    platform::collections::HashSet,
    render::{
        Extract,
        batching::gpu_preprocessing::GpuPreprocessingMode,
        render_phase::{ViewBinnedRenderPhases, ViewSortedRenderPhases},
        view::RetainedViewEntity,
    },
};

use crate::{light::prelude::*, occluder::prelude::*, sprite_depth::prelude::*};

/// Extract [`RetainedViewEntity`]s to relevant render phases.
pub(super) fn extract_view_entities(
    mut sprite_depth_phases: ResMut<ViewSortedRenderPhases<SpriteDepthPhase>>,
    mut occluder_phases: ResMut<ViewBinnedRenderPhases<OccluderPhase>>,
    mut light_phases: ResMut<ViewBinnedRenderPhases<Light2dPhase>>,
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
        sprite_depth_phases.insert_or_clear(retained_view_entity);
        occluder_phases.prepare_for_new_frame(retained_view_entity, GpuPreprocessingMode::None);
        light_phases.prepare_for_new_frame(retained_view_entity, GpuPreprocessingMode::None);
        live_entities.insert(retained_view_entity);
    }

    sprite_depth_phases.retain(|camera_entity, _| live_entities.contains(camera_entity));
    occluder_phases.retain(|camera_entity, _| live_entities.contains(camera_entity));
    light_phases.retain(|camera_entity, _| live_entities.contains(camera_entity));
}
