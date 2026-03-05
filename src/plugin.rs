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
    app::{App, Plugin},
    shader::load_shader_library,
};

use crate::light::point_light::PointLight2dPlugin;

/// [`Plugin`] that configures fast 2D lighting from this crate.
pub struct FastLightPlugin;
impl Plugin for FastLightPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "types.wgsl");
        app.add_plugins(PointLight2dPlugin);
    }
}
