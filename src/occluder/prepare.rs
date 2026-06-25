/*
 * Heavily inspired by:
 * - https://github.com/malbernaz/bevy_lit
 */

//! Preparation [`RenderSystems`](bevy::render::RenderSystems).

use bevy::{
    ecs::{
        query::With,
        resource::Resource,
        system::{Query, Res, ResMut},
    },
    platform::collections::HashMap,
    render::{
        render_resource::{TextureDescriptor, TextureDimension, TextureFormat, TextureUsages},
        renderer::RenderDevice,
        texture::{CachedTexture, TextureCache},
        view::{ExtractedView, RetainedViewEntity, ViewTarget},
    },
};

use crate::extract::prelude::*;

/// [`CachedTexture`]s for [`MeshOccluder`](crate::prelude::MeshOccluder).
#[derive(Resource, Default)]
pub(crate) struct OccluderTextures(pub(crate) HashMap<RetainedViewEntity, CachedTexture>);

/// Prepare [`CachedTexture`]s and insert into [`OccluderTextures`].
pub(super) fn prepare_occluder_texture(
    views: Query<(&ViewTarget, &ExtractedView), With<ExtractedAmbientLight2d>>,
    mut textures: ResMut<OccluderTextures>,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
) {
    for (view_target, extracted_view) in views {
        let texture_descriptor = TextureDescriptor {
            label: Some("occluder_texture"),
            size: view_target.main_texture().size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = texture_cache.get(&render_device, texture_descriptor);

        textures
            .0
            .insert(extracted_view.retained_view_entity, texture);
    }
}
