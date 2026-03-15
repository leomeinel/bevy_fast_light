/*
 * File: plugin.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! [`FastLightPlugin`] and related.

use bevy::{
    app::{App, Plugin, PostUpdate},
    camera::visibility::VisibilitySystems,
    ecs::{resource::Resource, schedule::IntoScheduleConfigs},
    render::extract_resource::ExtractResource,
};

use crate::{light::update_point_light_bounds, render::plugin::FastLightRenderPlugin};

/// [`Plugin`] for fast 2D lighting.
///
/// You also need to add an [`AmbientLight2d`](crate::prelude::AmbientLight2d) to a [Camera2d](bevy::camera::Camera2d) for this to work.
///
/// Additionally you can spawn [`PointLight2d`](crate::prelude::PointLight2d)s to light up certain areas.
pub struct FastLightPlugin {
    // FIXME: Implement this!
    /// Whether non-ambient lights should cast shadows.
    ///
    /// NOTE: This has not been implemented yet.
    pub cast_shadows: bool,
    /// Texture scale for any non-ambient light.
    ///
    /// The screen texture resolution will be multiplied by this to get the light texture resolution.
    pub texture_scale: f32,
}
impl Default for FastLightPlugin {
    fn default() -> Self {
        Self {
            cast_shadows: false,
            texture_scale: 0.125,
        }
    }
}
impl Plugin for FastLightPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FastLightSettings::from(self));

        app.add_plugins(FastLightRenderPlugin);

        app.add_systems(
            PostUpdate,
            update_point_light_bounds.in_set(VisibilitySystems::CalculateBounds),
        );
    }
}

/// Settings from [`FastLightPlugin`] as a [`Resource`].
///
/// This cannot be changed independently and should always be derived from [`FastLightPlugin`].
#[derive(Resource, Clone, Copy, ExtractResource)]
pub(crate) struct FastLightSettings {
    pub(crate) cast_shadows: bool,
    pub(crate) texture_scale: f32,
}
impl From<&FastLightPlugin> for FastLightSettings {
    fn from(plugin: &FastLightPlugin) -> Self {
        Self {
            cast_shadows: plugin.cast_shadows,
            texture_scale: plugin.texture_scale,
        }
    }
}
