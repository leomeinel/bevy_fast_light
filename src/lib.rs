/*
 * File: lib.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Simple 2D lighting for Bevy focused on performance over features.

mod light;
mod occluder;
mod plugin;
mod settings;

pub mod prelude {
    pub use crate::light::point_light::PointLight2d;
    pub use crate::plugin::FastLightPlugin;
}
