/*
 * File: color.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Utilities related to [`Color`].

use bevy::{
    color::{Color, ColorToComponents},
    math::Vec3,
};

/// Extension of [`Color`] to add additional functionality.
pub(crate) trait ColorExt {
    /// Convert to a Vec3 scaled by `intensity`
    fn to_scaled_vec3(self, intensity: f32) -> Vec3;
}
impl ColorExt for Color {
    fn to_scaled_vec3(self, intensity: f32) -> Vec3 {
        self.to_linear().to_vec3() * intensity
    }
}
