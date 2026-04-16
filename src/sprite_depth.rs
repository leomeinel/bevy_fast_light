/*
 * File: sprite_depth.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/PVDoriginal/firefly
 */

//! Modules for rendering to a scalable texture that uses the red channel for z-levels of all [`Sprite`](bevy::sprite::Sprite)s.
//!
//! This is the first render stage of [`FastLightPlugin`](crate::prelude::FastLightPlugin).

mod extract;
mod node;
mod phase;
mod pipeline;
mod plugin;
mod prepare;

pub(crate) mod prelude {
    pub(super) use super::node::SpriteDepthNode;
    pub(super) use super::phase::{DrawSpriteDepth, SpriteDepthPhase};
    pub(super) use super::pipeline::SpriteDepthPipeline;
    pub(crate) use super::plugin::{SpriteDepthLabel, SpriteDepthPlugin};
    pub(crate) use super::prepare::SpriteDepthTextures;
    pub(super) use super::{
        DrawSpriteDepthBatch, SetSpriteDepthTextureBindGroup, SpriteDepthBatch, SpriteDepthBatches,
        SpriteDepthImageBindGroups, SpriteDepthInstance, SpriteDepthMeta,
    };
}

use std::ops::Range;

use bevy::{
    asset::AssetId,
    ecs::{
        entity::Entity,
        query::ROQueryItem,
        resource::Resource,
        system::{
            SystemParamItem,
            lifetimeless::{Read, SRes},
        },
    },
    image::Image,
    math::{Affine3A, Vec4},
    platform::collections::HashMap,
    prelude::{Deref, DerefMut},
    render::{
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::{BindGroup, BufferUsages, IndexFormat, RawBufferVec},
        view::{ExtractedView, RetainedViewEntity},
    },
};
use bytemuck::{Pod, Zeroable};

/// Custom implementation of `SpriteInstance`.
///
/// This is mostly copied from [`sprite_render`](bevy::sprite_render).
///
/// Last updated from [`bevy`]@0.18.1.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(super) struct SpriteDepthInstance {
    // Affine 4x3 transposed to 3x4
    pub i_model_transpose: [Vec4; 3],
    pub i_uv_offset_scale: [f32; 4],
}
impl SpriteDepthInstance {
    #[inline]
    fn from(transform: &Affine3A, uv_offset_scale: &Vec4) -> Self {
        let transpose_model_3x3 = transform.matrix3.transpose();
        Self {
            i_model_transpose: [
                transpose_model_3x3.x_axis.extend(transform.translation.x),
                transpose_model_3x3.y_axis.extend(transform.translation.y),
                transpose_model_3x3.z_axis.extend(transform.translation.z),
            ],
            i_uv_offset_scale: uv_offset_scale.to_array(),
        }
    }
}

/// Custom implementation of [`SpriteMeta`](bevy::sprite_render::SpriteMeta).
///
/// This is mostly copied from [`sprite_render`](bevy::sprite_render).
///
/// Last updated from [`bevy`]@0.18.1.
#[derive(Resource)]
pub(super) struct SpriteDepthMeta {
    sprite_index_buffer: RawBufferVec<u32>,
    sprite_instance_buffer: RawBufferVec<SpriteDepthInstance>,
}
impl Default for SpriteDepthMeta {
    fn default() -> Self {
        Self {
            sprite_index_buffer: RawBufferVec::<u32>::new(BufferUsages::INDEX),
            sprite_instance_buffer: RawBufferVec::<SpriteDepthInstance>::new(BufferUsages::VERTEX),
        }
    }
}

/// Custom implementation of [`SpriteBatches`](bevy::sprite_render::SpriteBatches).
///
/// This is mostly copied from [`sprite_render`](bevy::sprite_render).
///
/// Last updated from [`bevy`]@0.18.1.
#[derive(Resource, Deref, DerefMut, Default)]
pub(super) struct SpriteDepthBatches(HashMap<(RetainedViewEntity, Entity), SpriteDepthBatch>);

/// Custom implementation of [`SpriteBatch`](bevy::sprite_render::SpriteBatch).
///
/// This is mostly copied from [`sprite_render`](bevy::sprite_render).
///
/// Last updated from [`bevy`]@0.18.1.
#[derive(PartialEq, Eq, Clone, Debug)]
pub(super) struct SpriteDepthBatch {
    pub(super) image_handle_id: AssetId<Image>,
    pub(super) range: Range<u32>,
}

/// Custom implementation of [`ImageBindGroups`](bevy::sprite_render::ImageBindGroups).
///
/// This is mostly copied from [`sprite_render`](bevy::sprite_render).
///
/// Last updated from [`bevy`]@0.18.1.
#[derive(Resource, Default)]
pub(super) struct SpriteDepthImageBindGroups {
    values: HashMap<AssetId<Image>, BindGroup>,
}

/// Custom implementation of [`SetSpriteTextureBindGroup`](bevy::sprite_render::SetSpriteTextureBindGroup).
///
/// This is mostly copied from [`sprite_render`](bevy::sprite_render).
///
/// Last updated from [`bevy`]@0.18.1.
pub(super) struct SetSpriteDepthTextureBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetSpriteDepthTextureBindGroup<I> {
    type Param = (SRes<SpriteDepthImageBindGroups>, SRes<SpriteDepthBatches>);
    type ViewQuery = Read<ExtractedView>;
    type ItemQuery = ();

    fn render<'w>(
        item: &P,
        view: ROQueryItem<'w, '_, Self::ViewQuery>,
        _entity: Option<()>,
        (image_bind_groups, batches): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let image_bind_groups = image_bind_groups.into_inner();
        let Some(batch) = batches.get(&(view.retained_view_entity, item.entity())) else {
            return RenderCommandResult::Skip;
        };

        pass.set_bind_group(
            I,
            image_bind_groups
                .values
                .get(&batch.image_handle_id)
                .unwrap(),
            &[],
        );
        RenderCommandResult::Success
    }
}

/// Custom implementation of [`DrawSpriteBatch`](bevy::sprite_render::DrawSpriteBatch).
///
/// This is mostly copied from [`sprite_render`](bevy::sprite_render).
///
/// Last updated from [`bevy`]@0.18.1.
pub(super) struct DrawSpriteDepthBatch;
impl<P: PhaseItem> RenderCommand<P> for DrawSpriteDepthBatch {
    type Param = (SRes<SpriteDepthMeta>, SRes<SpriteDepthBatches>);
    type ViewQuery = Read<ExtractedView>;
    type ItemQuery = ();

    fn render<'w>(
        item: &P,
        view: ROQueryItem<'w, '_, Self::ViewQuery>,
        _entity: Option<()>,
        (sprite_meta, batches): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let sprite_meta = sprite_meta.into_inner();
        let Some(batch) = batches.get(&(view.retained_view_entity, item.entity())) else {
            return RenderCommandResult::Skip;
        };

        pass.set_index_buffer(
            sprite_meta.sprite_index_buffer.buffer().unwrap().slice(..),
            IndexFormat::Uint32,
        );
        pass.set_vertex_buffer(
            0,
            sprite_meta
                .sprite_instance_buffer
                .buffer()
                .unwrap()
                .slice(..),
        );
        pass.draw_indexed(0..6, 0, batch.range.clone());
        RenderCommandResult::Success
    }
}
