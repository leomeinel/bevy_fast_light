/*
 * Heavily inspired by:
 * - https://github.com/malbernaz/bevy_lit
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

/// [`CachedTexture`]s for [`Light2dOccluder`](crate::prelude::Light2dOccluder).
#[derive(Resource, Default)]
pub(crate) struct OccluderTextures(pub(crate) HashMap<RetainedViewEntity, CachedTexture>);

/// Prepare scaled [`CachedTexture`]s and insert into [`OccluderTextures`].
pub(super) fn prepare_occluder_texture(
    views: Query<(&ViewTarget, &ExtractedView)>,
    mut textures: ResMut<OccluderTextures>,
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
