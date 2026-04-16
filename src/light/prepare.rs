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

//! Preparation [`RenderSystems`](bevy::render::RenderSystems).

use bevy::{
    ecs::{
        resource::Resource,
        system::{Query, Res, ResMut},
    },
    platform::collections::HashMap,
    render::{
        renderer::RenderDevice,
        texture::{CachedTexture, TextureCache},
        view::{ExtractedView, RetainedViewEntity, ViewTarget},
    },
};

use crate::{plugin::prelude::*, utils::prelude::*};

/// [`CachedTexture`]s for 2d lights.
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
        let texture =
            cached_scaled_2d_texture(&mut texture_cache, &render_device, &settings, view_target);

        textures
            .0
            .insert(extracted_view.retained_view_entity, texture);
    }
}
