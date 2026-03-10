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

//! Light types that get extracted to the render world and related systems.
//!
//! These extracted light types are being used in [`Light2dRenderPlugin`](crate::render::Light2dRenderPlugin).

use bevy::{
    camera::{Camera2d, visibility::ViewVisibility},
    color::LinearRgba,
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Single},
    },
    math::{FloatPow as _, Vec2, Vec3, Vec3Swizzles as _},
    render::{Extract, render_resource::ShaderType, sync_world::RenderEntity},
    transform::components::GlobalTransform,
    utils::default,
};

use crate::{
    light::{AmbientLight2d, PointLight2d},
    utils::ColorExt as _,
};

/// [`ShaderType`] that gets extracted to the render world for [`AmbientLight2d`].
#[derive(Component, Default, Clone, Copy, ShaderType, Debug)]
pub(crate) struct ExtractedAmbientLight2d {
    color: LinearRgba,
    light_count: u32,
    _padding: Vec3,
}
impl From<AmbientLight2d> for ExtractedAmbientLight2d {
    fn from(light: AmbientLight2d) -> Self {
        let color = light.color.to_scaled_linear(light.intensity);
        Self { color, ..default() }
    }
}
impl ExtractedAmbientLight2d {
    fn with_light_count(self, light_count: u32) -> Self {
        Self {
            light_count,
            ..self
        }
    }
}

/// [`ShaderType`] that gets extracted to the render world for [`PointLight2d`].
#[derive(Component, Default, Clone, Copy, ShaderType, Debug)]
pub(super) struct ExtractedPointLight2d {
    pub(super) cast_shadows: u32,
    pub(super) color: LinearRgba,
    pub(super) inner_radius_sq: f32,
    pub(super) outer_radius_sq: f32,
    pub(super) inv_radius_delta_sq: f32,
    pub(super) world_pos: Vec2,
    pub(super) _padding: Vec2,
}
impl From<PointLight2d> for ExtractedPointLight2d {
    fn from(light: PointLight2d) -> Self {
        let color = light.color.to_scaled_linear(light.intensity);
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

/// Extract [`AmbientLight2d`] as [`ExtractedAmbientLight2d`] to render world.
pub(super) fn extract_ambient(
    ambient: Extract<Single<(&RenderEntity, &AmbientLight2d), With<Camera2d>>>,
    light_query: Extract<Query<&ViewVisibility, With<PointLight2d>>>,
    mut commands: Commands,
) {
    let (render_entity, ambient) = **ambient;
    let light_count = light_query.iter().filter(|v| v.get()).count() as u32;
    commands
        .entity(**render_entity)
        .insert(ExtractedAmbientLight2d::from(*ambient).with_light_count(light_count));
}

/// Extract [`PointLight2d`] as [`ExtractedPointLight2d`] to render world.
pub(super) fn extract_point_lights(
    light_query: Extract<
        Query<(
            &RenderEntity,
            &PointLight2d,
            &GlobalTransform,
            &ViewVisibility,
        )>,
    >,
    mut commands: Commands,
) {
    for (render_entity, light, transform, visibility) in &light_query {
        if !visibility.get() {
            continue;
        }
        commands.entity(**render_entity).insert(
            ExtractedPointLight2d::from(*light).with_world_pos(transform.translation().xy()),
        );
    }
}
