//! Systems for extraction to the render world.

use bevy::{
    camera::{Camera, Camera2d},
    ecs::{
        entity::Entity,
        query::With,
        system::{Local, Query, ResMut},
    },
    platform::collections::HashSet,
    render::{Extract, render_phase::ViewSortedRenderPhases, view::RetainedViewEntity},
};

use crate::sprite_depth::prelude::*;

/// Extract [`RetainedViewEntity`]s to [`ViewSortedRenderPhases<SpriteDepthPhase>`] in render world.
pub(super) fn extract_occluder_view_entities(
    mut sprite_depth_phases: ResMut<ViewSortedRenderPhases<SpriteDepthPhase>>,
    cameras: Extract<Query<(Entity, &Camera), With<Camera2d>>>,
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
        live_entities.insert(retained_view_entity);
    }

    sprite_depth_phases.retain(|camera_entity, _| live_entities.contains(camera_entity));
}
