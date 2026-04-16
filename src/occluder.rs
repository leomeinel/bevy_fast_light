/*
 * File: occluder.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Different occluders and modules for rendering.
//!
//! This renders to a scalable texture that uses the red channel for determining if an occluder exists and the green channel for its' z-level.
//!
//! This is the second render stage of [`FastLightPlugin`](crate::prelude::FastLightPlugin).

mod extract;
mod node;
mod phase;
mod pipeline;
mod plugin;
mod prepare;

pub(super) mod prelude {
    pub(crate) use super::Light2dOccluder;
    pub(super) use super::extract::{self};
    pub(super) use super::node::OccluderNode;
    pub(super) use super::phase::{self, DrawOccluder, OccluderPhase};
    pub(super) use super::pipeline::{self, OccluderPipeline};
    pub(crate) use super::plugin::{OccluderLabel, OccluderPlugin};
    pub(crate) use super::prepare::OccluderTextures;
    pub(super) use super::prepare::{self};
}

use bevy::{ecs::component::Component, render::extract_component::ExtractComponent};

/// Light occluder for 2D environments.
///
/// This is meant to be added to a [`Mesh2d`](bevy::mesh::Mesh2d) which will determine the occluded shape.
///
/// This fully occludes all non-ambient light in lower z-levels.
#[derive(Component, ExtractComponent, Clone, Copy, Default)]
pub struct Light2dOccluder;
