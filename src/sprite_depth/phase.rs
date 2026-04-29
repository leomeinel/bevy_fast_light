//! [`PhaseItem`]s and related for [`Sprite`] z-level rendering.

use std::ops::Range;

use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    ecs::{
        entity::Entity,
        system::{Local, Query, Res, ResMut},
    },
    math::FloatOrd,
    render::{
        render_phase::{
            CachedRenderPipelinePhaseItem, DrawFunctionId, DrawFunctions, PhaseItem,
            PhaseItemExtraIndex, SetItemPipeline, SortedPhaseItem, ViewSortedRenderPhases,
        },
        render_resource::{CachedRenderPipelineId, PipelineCache, SpecializedRenderPipelines},
        sync_world::MainEntity,
        view::{ExtractedView, Msaa, RenderVisibleEntities},
    },
    sprite::Sprite,
    sprite_render::{ExtractedSprites, SetSpriteViewBindGroup, SpritePipelineKey},
};
use fixedbitset::FixedBitSet;

use crate::sprite_depth::prelude::*;

/// [`PhaseItem`] drawn in the render phase for [`Sprite`] z-level rendering.
///
/// Last updated from [`bevy`]@0.18.1.
pub(super) struct SpriteDepthPhase {
    pub sort_key: FloatOrd,
    pub entity: (Entity, MainEntity),
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    pub batch_range: Range<u32>,
    pub extracted_index: usize,
    pub extra_index: PhaseItemExtraIndex,
    pub indexed: bool,
}
impl PhaseItem for SpriteDepthPhase {
    #[inline]
    fn entity(&self) -> Entity {
        self.entity.0
    }
    #[inline]
    fn main_entity(&self) -> MainEntity {
        self.entity.1
    }
    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }
    #[inline]
    fn batch_range(&self) -> &Range<u32> {
        &self.batch_range
    }
    #[inline]
    fn batch_range_mut(&mut self) -> &mut Range<u32> {
        &mut self.batch_range
    }
    #[inline]
    fn extra_index(&self) -> PhaseItemExtraIndex {
        self.extra_index.clone()
    }
    #[inline]
    fn batch_range_and_extra_index_mut(&mut self) -> (&mut Range<u32>, &mut PhaseItemExtraIndex) {
        (&mut self.batch_range, &mut self.extra_index)
    }
}
impl SortedPhaseItem for SpriteDepthPhase {
    type SortKey = FloatOrd;
    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        self.sort_key
    }
    #[inline]
    fn indexed(&self) -> bool {
        self.indexed
    }
}
impl CachedRenderPipelinePhaseItem for SpriteDepthPhase {
    #[inline]
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline
    }
}

/// Custom implementation of [`DrawSprite`](bevy::sprite_render::DrawSprite).
///
/// This is mostly copied from `DrawSprite` of [`sprite_render`](bevy::sprite_render).
///
/// Last updated from [`bevy`]@0.18.1.
pub(super) type DrawSpriteDepth = (
    SetItemPipeline,
    SetSpriteViewBindGroup<0>,
    SetSpriteDepthTextureBindGroup<1>,
    DrawSpriteDepthBatch,
);

/// Queue drawable entities as [`SpriteDepthPhase`]s phase items in render phases ready for sorting.
///
/// This is mostly copied from [`queue_sprites`](bevy::sprite_render::queue_sprites).
///
/// Last updated from [`bevy`]@0.18.1.
pub fn queue_sprite_depths(
    mut view_entities: Local<FixedBitSet>,
    draw_functions: Res<DrawFunctions<SpriteDepthPhase>>,
    sprite_depth_pipeline: Res<SpriteDepthPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SpriteDepthPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    extracted_sprites: Res<ExtractedSprites>,
    mut sprite_depth_phases: ResMut<ViewSortedRenderPhases<SpriteDepthPhase>>,
    mut views: Query<(
        &RenderVisibleEntities,
        &ExtractedView,
        &Msaa,
        Option<&Tonemapping>,
        Option<&DebandDither>,
    )>,
) {
    let draw_function = draw_functions.read().id::<DrawSpriteDepth>();

    for (visible_entities, view, msaa, tonemapping, dither) in &mut views {
        let Some(phase) = sprite_depth_phases.get_mut(&view.retained_view_entity) else {
            continue;
        };

        let msaa_key = SpritePipelineKey::from_msaa_samples(msaa.samples());
        let mut view_key = SpritePipelineKey::from_hdr(view.hdr) | msaa_key;
        if !view.hdr {
            if let Some(tonemapping) = tonemapping {
                view_key |= SpritePipelineKey::TONEMAP_IN_SHADER;
                view_key |= match tonemapping {
                    Tonemapping::None => SpritePipelineKey::TONEMAP_METHOD_NONE,
                    Tonemapping::Reinhard => SpritePipelineKey::TONEMAP_METHOD_REINHARD,
                    Tonemapping::ReinhardLuminance => {
                        SpritePipelineKey::TONEMAP_METHOD_REINHARD_LUMINANCE
                    }
                    Tonemapping::AcesFitted => SpritePipelineKey::TONEMAP_METHOD_ACES_FITTED,
                    Tonemapping::AgX => SpritePipelineKey::TONEMAP_METHOD_AGX,
                    Tonemapping::SomewhatBoringDisplayTransform => {
                        SpritePipelineKey::TONEMAP_METHOD_SOMEWHAT_BORING_DISPLAY_TRANSFORM
                    }
                    Tonemapping::TonyMcMapface => SpritePipelineKey::TONEMAP_METHOD_TONY_MC_MAPFACE,
                    Tonemapping::BlenderFilmic => SpritePipelineKey::TONEMAP_METHOD_BLENDER_FILMIC,
                };
            }
            if let Some(DebandDither::Enabled) = dither {
                view_key |= SpritePipelineKey::DEBAND_DITHER;
            }
        }

        let pipeline = pipelines.specialize(&pipeline_cache, &sprite_depth_pipeline, view_key);

        view_entities.clear();
        view_entities.extend(
            visible_entities
                .iter::<Sprite>()
                .map(|(_, e)| e.index_u32() as usize),
        );

        phase.items.reserve(extracted_sprites.sprites.len());

        for (index, extracted_sprite) in extracted_sprites.sprites.iter().enumerate() {
            let view_index = extracted_sprite.main_entity.index_u32();
            if !view_entities.contains(view_index as usize) {
                continue;
            }

            // These items will be sorted by depth with other phase items
            let sort_key = FloatOrd(extracted_sprite.transform.translation().z);

            // Add the item to the render phase
            phase.add(SpriteDepthPhase {
                draw_function,
                pipeline,
                entity: (
                    extracted_sprite.render_entity,
                    extracted_sprite.main_entity.into(),
                ),
                sort_key,
                // `batch_range` is calculated in `prepare_sprite_image_bind_groups`
                batch_range: 0..0,
                extra_index: PhaseItemExtraIndex::None,
                extracted_index: index,
                indexed: true,
            });
        }
    }
}
