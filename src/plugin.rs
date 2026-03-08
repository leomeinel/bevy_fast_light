/*
 * File: plugin.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! [`FastLightPlugin`] and related.

use bevy::app::{App, Plugin};

use crate::render::Light2dRenderPlugin;

/// [`Plugin`] for fast 2D lighting.
///
/// You also need to add an [`AmbientLight2d`](crate::prelude::AmbientLight2d) to a [Camera2d](bevy::camera::Camera2d) for this to work.
///
/// Additionally you can spawn [`PointLight2d`](crate::prelude::PointLight2d)s to light up certain areas.
pub struct FastLightPlugin;
impl Plugin for FastLightPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Light2dRenderPlugin);
    }
}
