/*
 * File: prepare.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/malbernaz/bevy_lit
 */

//! Preparation systems for rendering.

use bevy::{
    ecs::{
        resource::Resource,
        system::{Query, Res, ResMut},
    },
    image::BevyDefault,
    platform::collections::HashMap,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::RenderDevice,
        texture::{CachedTexture, TextureCache},
        view::{ExtractedView, RetainedViewEntity, ViewTarget},
    },
};

use crate::plugin::FastLightSettings;

#[derive(Resource, Default)]
pub(super) struct Light2dTextures(pub(super) HashMap<RetainedViewEntity, CachedTexture>);

/// Prepare scaled [`CachedTexture`]s and insert into [`Light2dTextures`].
pub(super) fn prepare_light_2d_texture(
    views: Query<(&ViewTarget, &ExtractedView)>,
    mut textures: ResMut<Light2dTextures>,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    settings: Res<FastLightSettings>,
) {
    for (view_target, extracted_view) in views {
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
            format: TextureFormat::bevy_default(),
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = texture_cache.get(&render_device, texture_descriptor);

        textures
            .0
            .insert(extracted_view.retained_view_entity, texture);
    }
}
