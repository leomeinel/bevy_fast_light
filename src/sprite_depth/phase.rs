//! [`PhaseItem`]s and related for [`Sprite`] z-level rendering.

use std::ops::Range;

use bevy::{
    camera::CompositingSpace,
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    ecs::{
        entity::{Entity, EntityHash},
        query::With,
        system::{Local, Query, Res, ResMut},
    },
    material::{descriptor::CachedRenderPipelineId, labels::DrawFunctionId},
    math::FloatOrd,
    render::{
        camera::ExtractedCamera,
        render_phase::{
            CachedRenderPipelinePhaseItem, DrawFunctions, PhaseItem, PhaseItemExtraIndex,
            SetItemPipeline, SortedPhaseItem, ViewSortedRenderPhases,
        },
        render_resource::{PipelineCache, SpecializedRenderPipelines},
        sync_world::MainEntity,
        view::{ExtractedView, Msaa, RenderVisibleEntities},
    },
    sprite::Sprite,
    sprite_render::{ExtractedSprites, SetSpriteViewBindGroup, SpritePipelineKey},
};
use fixedbitset::FixedBitSet;
use indexmap::IndexMap;

use crate::{extract::prelude::*, sprite_depth::prelude::*};

/// Custom implementation of [`Transparent2d`](bevy::core_pipeline::core_2d::Transparent2d).
///
/// Last updated from [`bevy`]@0.19.0.
pub struct SpriteDepthPhase {
    pub sort_key: FloatOrd,
    pub entity: (Entity, MainEntity),
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    pub batch_range: Range<u32>,
    pub extracted_index: usize,
    pub extra_index: PhaseItemExtraIndex,
    /// Whether the mesh in question is indexed (uses an index buffer in
    /// addition to its vertex buffer).
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
    fn sort(items: &mut IndexMap<(Entity, MainEntity), SpriteDepthPhase, EntityHash>) {
        items.sort_by_key(|_, item| item.sort_key());
    }

    fn recalculate_sort_keys(
        _: &mut IndexMap<(Entity, MainEntity), Self, EntityHash>,
        _: &ExtractedView,
    ) {
    }

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

/// [`RenderCommand`](bevy::render::render_phase::RenderCommand) for sprite rendering.
///
/// This is mostly copied from [`DrawSprite`](bevy::sprite_render::DrawSprite).
///
/// Last updated from [`bevy`]@0.19.0.
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
/// Last updated from [`bevy`]@0.19.0.
pub fn queue_sprite_depths(
    mut view_entities: Local<FixedBitSet>,
    draw_functions: Res<DrawFunctions<SpriteDepthPhase>>,
    sprite_depth_pipeline: Res<SpriteDepthPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SpriteDepthPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    extracted_sprites: Res<ExtractedSprites>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<SpriteDepthPhase>>,
    mut cameras: Query<
        (
            &RenderVisibleEntities,
            &ExtractedCamera,
            &ExtractedView,
            &Msaa,
            Option<&Tonemapping>,
            Option<&DebandDither>,
        ),
        With<ExtractedAmbientLight2d>,
    >,
) {
    let draw_function = draw_functions.read().id::<DrawSpriteDepth>();

    for (visible_entities, camera, view, msaa, tonemapping, dither) in &mut cameras {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        let msaa_key = SpritePipelineKey::from_msaa_samples(msaa.samples());
        let mut view_key = SpritePipelineKey::from_target_format(view.target_format) | msaa_key;

        if camera
            .compositing_space
            .is_some_and(|s| s == CompositingSpace::Srgb)
        {
            view_key |= SpritePipelineKey::SRGB_COMPOSITING;
        }
        if camera
            .compositing_space
            .is_some_and(|s| s == CompositingSpace::Oklab)
        {
            view_key |= SpritePipelineKey::OKLAB_COMPOSITING;
        }

        if !camera.hdr {
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
                    Tonemapping::KhronosPbrNeutral => SpritePipelineKey::TONEMAP_METHOD_PBR_NEUTRAL,
                };
            }
            if let Some(DebandDither::Enabled) = dither {
                view_key |= SpritePipelineKey::DEBAND_DITHER;
            }
        }

        let pipeline = pipelines.specialize(&pipeline_cache, &sprite_depth_pipeline, view_key);

        view_entities.clear();
        if let Some(visible_entities) = visible_entities.get::<Sprite>() {
            view_entities.extend(
                visible_entities
                    .iter_visible()
                    .map(|(_, e)| e.index_u32() as usize),
            );
        }

        transparent_phase
            .items
            .reserve(extracted_sprites.sprites.len());

        for (index, extracted_sprite) in extracted_sprites.sprites.iter().enumerate() {
            let view_index = extracted_sprite.main_entity.index_u32();

            if !view_entities.contains(view_index as usize) {
                continue;
            }

            // These items will be sorted by depth with other phase items
            let sort_key = FloatOrd(extracted_sprite.transform.translation().z);

            // Add the item to the render phase
            transparent_phase.add_transient(SpriteDepthPhase {
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
