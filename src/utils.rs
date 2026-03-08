/*
 * File: utils.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Utilities to be used in the crate.

use bevy::color::{Alpha as _, Color, LinearRgba};

/// Extension of [`Color`] to add additional functionality.
pub(crate) trait ColorExt {
    /// Return the color as a linear RGBA color scaled by `intensity` and with an alpha of `1.`.
    fn to_scaled_linear(self, intensity: f32) -> LinearRgba;
}
impl ColorExt for Color {
    fn to_scaled_linear(self, intensity: f32) -> LinearRgba {
        (self.to_linear() * intensity).with_alpha(1.)
    }
}
