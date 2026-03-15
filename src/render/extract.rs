/*
 * File: extract.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/jgayfer/bevy_light_2d
 */

//! Extracted [`Component`]s and systems for extraction to the render world.

use bevy::{
    camera::{Camera2d, visibility::ViewVisibility},
    ecs::{
        component::Component,
        lifecycle::RemovedComponents,
        query::{Changed, Or, With},
        system::{Commands, Query, Res, Single},
    },
    math::{FloatPow as _, Vec2, Vec3, Vec3Swizzles as _},
    render::{Extract, render_resource::ShaderType, sync_world::RenderEntity},
    transform::components::GlobalTransform,
    utils::default,
};

use crate::{
    light::{AmbientLight2d, PointLight2d},
    plugin::FastLightSettings,
    utils::ColorExt as _,
};

/// [`ShaderType`] that gets extracted to the render world for [`AmbientLight2d`].
#[derive(Component, Default, Clone, Copy, ShaderType, Debug)]
pub(super) struct ExtractedAmbientLight2d {
    color: Vec3,
    pub(super) _padding: f32,
}
impl From<AmbientLight2d> for ExtractedAmbientLight2d {
    fn from(light: AmbientLight2d) -> Self {
        let color = light.color.to_scaled_vec3(light.intensity);
        Self { color, ..default() }
    }
}

/// [`ShaderType`] that gets extracted to the render world for [`PointLight2d`].
#[derive(Component, Default, Clone, Copy, ShaderType, Debug)]
pub(super) struct ExtractedPointLight2d {
    pub(super) color: Vec3,
    pub(super) inner_radius_sq: f32,
    pub(super) world_pos: Vec2,
    pub(super) outer_radius_sq: f32,
    pub(super) inv_radius_delta_sq: f32,
}
impl From<PointLight2d> for ExtractedPointLight2d {
    fn from(light: PointLight2d) -> Self {
        let color = light.color.to_scaled_vec3(light.intensity);
        let inner_radius_sq = light.inner_radius.squared();
        let outer_radius_sq = light.outer_radius.squared();
        let inv_radius_delta_sq = 1. / (outer_radius_sq - inner_radius_sq).max(1.);
        Self {
            color,
            inner_radius_sq,
            outer_radius_sq,
            inv_radius_delta_sq,
            ..default()
        }
    }
}
impl ExtractedPointLight2d {
    fn with_world_pos(self, world_pos: Vec2) -> Self {
        Self { world_pos, ..self }
    }
}

/// [`ShaderType`] that gets extracted to the render world with metadata related to lights.
#[derive(Component, Default, Clone, Copy, ShaderType, Debug)]
pub(super) struct ExtractedLight2dMeta {
    pub(super) cast_shadows: u32,
    pub(super) count: u32,
    pub(super) _padding: Vec2,
}
impl From<&FastLightSettings> for ExtractedLight2dMeta {
    fn from(settings: &FastLightSettings) -> Self {
        Self {
            cast_shadows: if settings.cast_shadows { 1 } else { 0 },
            ..default()
        }
    }
}
impl ExtractedLight2dMeta {
    fn with_count(self, count: u32) -> Self {
        Self { count, ..self }
    }
}

/// Extract [`AmbientLight2d`] as [`ExtractedAmbientLight2d`] to render world.
pub(super) fn extract_ambient(
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

/// Extract [`ExtractedLight2dMeta`] to render world.
pub(super) fn extract_light_meta(
    removed_lights: Extract<RemovedComponents<PointLight2d>>,
    ambient: Extract<Single<&RenderEntity, (With<AmbientLight2d>, With<Camera2d>)>>,
    light_changed_query: Extract<
        Query<
            (),
            (
                Or<(
                    Changed<PointLight2d>,
                    Changed<GlobalTransform>,
                    // NOTE: Even though the docs suggest that this would trigger every frame,
                    //       the actual implementation mostly avoids incorrect triggers.
                    Changed<ViewVisibility>,
                )>,
                With<PointLight2d>,
            ),
        >,
    >,
    light_query: Extract<Query<&ViewVisibility, With<PointLight2d>>>,
    mut commands: Commands,
    settings: Extract<Res<FastLightSettings>>,
) {
    if light_changed_query.is_empty() && removed_lights.is_empty() {
        return;
    }
    let render_entity = **ambient;
    let count = light_query.iter().filter(|v| v.get()).count() as u32;

    commands
        .entity(**render_entity)
        .insert(ExtractedLight2dMeta::from(&**settings).with_count(count));
}

/// Extract [`PointLight2d`] as [`ExtractedPointLight2d`] to render world.
pub(super) fn extract_point_lights(
    mut removed_lights: Extract<RemovedComponents<PointLight2d>>,
    light_query: Extract<
        Query<
            (
                &RenderEntity,
                &PointLight2d,
                &GlobalTransform,
                &ViewVisibility,
            ),
            Or<(
                Changed<PointLight2d>,
                Changed<GlobalTransform>,
                // NOTE: Even though the docs suggest that this would trigger every frame,
                //       the actual implementation mostly avoids incorrect triggers.
                Changed<ViewVisibility>,
            )>,
        >,
    >,
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
            .remove::<ExtractedPointLight2d>();
    }

    // Insert new extracted components
    for (render_entity, light, transform, visibility) in &light_query {
        if !visibility.get() {
            commands
                .entity(**render_entity)
                .remove::<ExtractedPointLight2d>();
            continue;
        }
        commands.entity(**render_entity).insert(
            ExtractedPointLight2d::from(*light).with_world_pos(transform.translation().xy()),
        );
    }
}
