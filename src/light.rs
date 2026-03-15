/*
 * File: light.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Different types of lights for 2D environments.

use bevy::{
    camera::{
        primitives::Aabb,
        visibility::{NoFrustumCulling, Visibility, VisibilityClass, add_visibility_class},
    },
    color::Color,
    ecs::{
        component::Component,
        entity::Entity,
        query::{Changed, Or, Without},
        system::{Commands, Query},
    },
    math::Vec3A,
    reflect::Reflect,
    render::sync_world::SyncToRenderWorld,
    transform::components::Transform,
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

/// Point light for area lighting in a 2D environment.
///
/// ## Formula
///
/// color = src_color * (ambient_color + [`PointLight2d::color`] * [`PointLight2d::intensity`] * attenuation).
///
/// NOTE: attenuation is influenced by [`PointLight2d::inner_radius`] and [`PointLight2d::outer_radius`].
#[derive(Component, Reflect, Clone, Copy)]
#[require(SyncToRenderWorld, Transform, Visibility, VisibilityClass)]
#[component(on_add = add_visibility_class::<Self>)]
pub struct PointLight2d {
    /// The [`Color`] of the light.
    pub color: Color,
    /// The intensity of the light.
    pub intensity: f32,
    /// The inner radius of the light.
    ///
    /// `attenuation` is always 1 (actually it is disregarded entirely).
    pub inner_radius: f32,
    /// The outer radius of the light.
    ///
    /// `attenuation` starts at 1 and then decreases smoothly if outside of [`PointLight2d::inner_radius`] until it reaches 0.
    pub outer_radius: f32,
}
impl PointLight2d {
    fn aabb(&self) -> Aabb {
        Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::new(self.outer_radius, self.outer_radius, 0.),
        }
    }
}
impl Default for PointLight2d {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.,
            inner_radius: 0.,
            outer_radius: 64.,
        }
    }
}

/// Update [`Aabb`] for [`PointLight2d`].
///
/// This allows [`PointLight2d`] to integrate with bevy native frustum culling.
pub(super) fn update_point_light_bounds(
    light_query: Query<
        (Entity, &PointLight2d),
        (
            Or<(Changed<PointLight2d>, Without<Aabb>)>,
            Without<NoFrustumCulling>,
        ),
    >,
    mut commands: Commands,
) {
    for (entity, light) in light_query {
        commands.entity(entity).insert(light.aabb());
    }
}
