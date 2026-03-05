/*
 * File: error.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Error messages.

/// Error message if [`AmbientLight2d`](crate::light::ambient_light::AmbientLight2d) has not been added to a [`Camera2d`](bevy::camera::Camera2d).
pub(crate) const ERR_AMBIENT_NO_CAMERA: &str =
    "AmbientLight2d has not been added to a Camera2d. Ensure that this is the case.";
