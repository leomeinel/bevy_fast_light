/*
 * File: plugin.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! [`FastLightPlugin`] and related.
//!
//! # Render stages
//!
//! 1. Render to a scalable texture that uses the red channel for z-levels of all [`Sprites`](bevy::sprite::Sprite)s.
//! 2. Render to a scalable texture that uses the red channel for determining if an occluder exists and the green channel for its' z-level.
//! 3. Renders a light map to a scalable texture.
//! 4. Compose from light map to screen texture.

pub(crate) mod prelude {
    pub(crate) use super::FastLightSettings;
}

use bevy::{
    app::{App, Plugin},
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    ecs::resource::Resource,
    render::{RenderApp, extract_resource::ExtractResource, render_graph::RenderGraphExt as _},
};

use crate::{light::prelude::*, occluder::prelude::*, sprite_depth::prelude::*};

/// [`Plugin`] for fast 2D lighting.
///
/// You also need to add an [`AmbientLight2d`] to a [Camera2d](bevy::camera::Camera2d) for this to work.
///
/// Additionally you can spawn [`PointLight2d`]s to light up certain areas.
pub struct FastLightPlugin {
    /// Texture scale for any non-ambient light.
    ///
    /// Textures uses in rendering will be multiplied by this to get the light texture resolution.
    pub texture_scale: f32,
}
impl Default for FastLightPlugin {
    fn default() -> Self {
        Self { texture_scale: 0.5 }
    }
}
impl Plugin for FastLightPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FastLightSettings::from(self));

        app.add_plugins((SpriteDepthPlugin, OccluderPlugin, Light2dPlugin));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_render_graph_edges(
            Core2d,
            (
                Node2d::MainOpaquePass,
                SpriteDepthLabel,
                OccluderLabel,
                Node2d::MainTransparentPass,
                Node2d::Tonemapping,
                Light2dLabel,
                Light2dCompositeLabel,
                Node2d::EndMainPassPostProcessing,
            ),
        );
    }
}

/// Settings from [`FastLightPlugin`] as a [`Resource`].
///
/// This cannot be changed independently and should always be derived from [`FastLightPlugin`].
#[derive(Resource, Clone, Copy, ExtractResource)]
pub(crate) struct FastLightSettings {
    pub(crate) texture_scale: f32,
}
impl From<&FastLightPlugin> for FastLightSettings {
    fn from(plugin: &FastLightPlugin) -> Self {
        Self {
            texture_scale: plugin.texture_scale,
        }
    }
}
