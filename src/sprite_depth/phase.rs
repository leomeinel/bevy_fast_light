//! [`PhaseItem`]s and related for [`Sprite`] z-level rendering.

use bevy::{
    camera::CompositingSpace,
    core_pipeline::{
        core_2d::Transparent2d,
        tonemapping::{DebandDither, Tonemapping},
    },
    ecs::{
        query::With,
        system::{Local, Query, Res, ResMut},
    },
    math::FloatOrd,
    render::{
        camera::ExtractedCamera,
        render_phase::{
            DrawFunctions, PhaseItemExtraIndex, SetItemPipeline, ViewSortedRenderPhases,
        },
        render_resource::{PipelineCache, SpecializedRenderPipelines},
        view::{ExtractedView, Msaa, RenderVisibleEntities},
    },
    sprite::Sprite,
    sprite_render::{ExtractedSprites, SetSpriteViewBindGroup, SpritePipelineKey},
};
use fixedbitset::FixedBitSet;

use crate::{extract::prelude::*, sprite_depth::prelude::*};

/// [`RenderCommand`](bevy::render::render_phase::RenderCommand) for sprite rendering.
///
/// This is mostly copied from [`DrawSprite`](bevy::sprite_render::render::DrawSprite).
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
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    sprite_depth_pipeline: Res<SpriteDepthPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SpriteDepthPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    extracted_sprites: Res<ExtractedSprites>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
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
            transparent_phase.add_transient(Transparent2d {
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
