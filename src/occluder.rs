/*
 * File: occluder.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Types of occluders.

use bevy::{ecs::component::Component, render::extract_component::ExtractComponent};

/// Light occluder for 2D environments.
///
/// This is meant to be added to a [`Mesh2d`](bevy::mesh::Mesh2d) which will determine the occluded shape.
///
/// This fully occludes all non-ambient light.
#[derive(Component, ExtractComponent, Clone, Copy, Default)]
pub struct Light2dOccluder;
