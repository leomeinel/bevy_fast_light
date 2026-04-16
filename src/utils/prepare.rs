/*
 * File: prepare.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Utilities related to preparation [`RenderSystems`](bevy::render::RenderSystems).

use bevy::render::{
    render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    renderer::RenderDevice,
    texture::{CachedTexture, TextureCache},
    view::ViewTarget,
};

use crate::plugin::prelude::*;

/// [`CachedTexture`] used by prepare systems.
///
/// It uses [`TextureFormat::Rgba8Unorm`] for maximum compatibility and supports [`TextureUsages::RENDER_ATTACHMENT`] and [`TextureUsages::TEXTURE_BINDING`].
pub(crate) fn cached_scaled_2d_texture(
    texture_cache: &mut TextureCache,
    render_device: &RenderDevice,
    settings: &FastLightSettings,
    view_target: &ViewTarget,
) -> CachedTexture {
    let size = view_target.main_texture().size();
    let size = Extent3d {
        width: (size.width as f32 * settings.texture_scale).round() as u32,
        height: (size.height as f32 * settings.texture_scale).round() as u32,
        ..size
    };
    let texture_descriptor = TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };
    texture_cache.get(render_device, texture_descriptor)
}
