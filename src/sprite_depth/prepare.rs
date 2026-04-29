//! Preparation [`RenderSystems`](bevy::render::RenderSystems).

use bevy::{
    asset::{AssetEvent, AssetId},
    core_pipeline::tonemapping::{Tonemapping, TonemappingLuts, get_lut_bindings},
    ecs::{
        entity::Entity,
        query::With,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    math::{Affine3A, Quat, Vec2, Vec4},
    platform::collections::HashMap,
    render::{
        render_asset::RenderAssets,
        render_phase::{PhaseItem as _, ViewSortedRenderPhases},
        render_resource::{BindGroupEntries, PipelineCache},
        renderer::{RenderDevice, RenderQueue},
        texture::{CachedTexture, FallbackImage, GpuImage, TextureCache},
        view::{ExtractedView, RetainedViewEntity, ViewTarget, ViewUniforms},
    },
    sprite::SpriteScalingMode,
    sprite_render::{
        ExtractedSlices, ExtractedSpriteKind, ExtractedSprites, SpriteAssetEvents,
        SpriteViewBindGroup,
    },
};

use crate::{plugin::prelude::*, sprite_depth::prelude::*, utils::prelude::*};

/// [`CachedTexture`]s for [`Sprite`](bevy::sprite::Sprite) depth.
#[derive(Resource, Default)]
pub(crate) struct SpriteDepthTextures(pub(crate) HashMap<RetainedViewEntity, CachedTexture>);

/// Prepare scaled [`CachedTexture`]s and insert into [`SpriteDepthTextures`].
pub(super) fn prepare_sprite_depth_texture(
    views: Query<(&ViewTarget, &ExtractedView)>,
    mut textures: ResMut<SpriteDepthTextures>,
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

/// Prepare [`SpriteViewBindGroup`].
///
/// This is mostly copied from [`prepare_sprite_view_bind_groups`](bevy::sprite_render::prepare_sprite_view_bind_groups).
///
/// Last updated from [`bevy`]@0.18.1.
pub(super) fn prepare_sprite_depth_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    sprite_pipeline: Res<SpriteDepthPipeline>,
    view_uniforms: Res<ViewUniforms>,
    views: Query<(Entity, &Tonemapping), With<ExtractedView>>,
    tonemapping_luts: Res<TonemappingLuts>,
    images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    for (entity, tonemapping) in &views {
        let lut_bindings =
            get_lut_bindings(&images, &tonemapping_luts, tonemapping, &fallback_image);
        let view_bind_group = render_device.create_bind_group(
            "mesh2d_view_bind_group",
            &pipeline_cache.get_bind_group_layout(&sprite_pipeline.view_layout),
            &BindGroupEntries::sequential((view_binding.clone(), lut_bindings.0, lut_bindings.1)),
        );

        commands.entity(entity).insert(SpriteViewBindGroup {
            value: view_bind_group,
        });
    }
}

/// Prepare [`SpriteDepthImageBindGroups`].
///
/// This is mostly copied from [`prepare_sprite_image_bind_groups`](bevy::sprite_render::prepare_sprite_image_bind_groups).
///
/// Last updated from [`bevy`]@0.18.1.
pub(super) fn prepare_sprite_depth_image_bind_groups(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    mut sprite_meta: ResMut<SpriteDepthMeta>,
    sprite_pipeline: Res<SpriteDepthPipeline>,
    mut image_bind_groups: ResMut<SpriteDepthImageBindGroups>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    extracted_sprites: Res<ExtractedSprites>,
    extracted_slices: Res<ExtractedSlices>,
    mut phases: ResMut<ViewSortedRenderPhases<SpriteDepthPhase>>,
    events: Res<SpriteAssetEvents>,
    mut batches: ResMut<SpriteDepthBatches>,
) {
    // If an image has changed, the GpuImage has (probably) changed
    for event in &events.images {
        match event {
            AssetEvent::Added { .. } |
            // Images don't have dependencies
            AssetEvent::LoadedWithDependencies { .. } => {}
            AssetEvent::Unused { id } | AssetEvent::Modified { id } | AssetEvent::Removed { id } => {
                image_bind_groups.values.remove(id);
            }
        };
    }

    batches.clear();

    // Clear the sprite instances
    sprite_meta.sprite_instance_buffer.clear();

    // Index buffer indices
    let mut index = 0;

    let image_bind_groups = &mut *image_bind_groups;

    for (retained_view, transparent_phase) in phases.iter_mut() {
        let mut current_batch = None;
        let mut batch_item_index = 0;
        let mut batch_image_size = Vec2::ZERO;
        let mut batch_image_handle = AssetId::invalid();

        // Iterate through the phase items and detect when successive sprites that can be batched.
        // Spawn an entity with a `SpriteBatch` component for each possible batch.
        // Compatible items share the same entity.
        for item_index in 0..transparent_phase.items.len() {
            let item = &transparent_phase.items[item_index];

            let Some(extracted_sprite) = extracted_sprites
                .sprites
                .get(item.extracted_index)
                .filter(|extracted_sprite| extracted_sprite.render_entity == item.entity())
            else {
                // If there is a phase item that is not a sprite, then we must start a new
                // batch to draw the other phase item(s) and to respect draw order. This can be
                // done by invalidating the batch_image_handle
                batch_image_handle = AssetId::invalid();
                continue;
            };

            if batch_image_handle != extracted_sprite.image_handle_id {
                let Some(gpu_image) = gpu_images.get(extracted_sprite.image_handle_id) else {
                    continue;
                };

                batch_image_size = gpu_image.size_2d().as_vec2();
                batch_image_handle = extracted_sprite.image_handle_id;
                image_bind_groups
                    .values
                    .entry(batch_image_handle)
                    .or_insert_with(|| {
                        render_device.create_bind_group(
                            "sprite_depth_material_bind_group",
                            &pipeline_cache.get_bind_group_layout(&sprite_pipeline.material_layout),
                            &BindGroupEntries::sequential((
                                &gpu_image.texture_view,
                                &gpu_image.sampler,
                            )),
                        )
                    });

                batch_item_index = item_index;
                current_batch = Some(batches.entry((*retained_view, item.entity())).insert(
                    SpriteDepthBatch {
                        image_handle_id: batch_image_handle,
                        range: index..index,
                    },
                ));
            }
            match extracted_sprite.kind {
                ExtractedSpriteKind::Single {
                    anchor,
                    rect,
                    scaling_mode,
                    custom_size,
                } => {
                    // By default, the size of the quad is the size of the texture
                    let mut quad_size = batch_image_size;
                    let mut texture_size = batch_image_size;

                    // Calculate vertex data for this item
                    // If a rect is specified, adjust UVs and the size of the quad
                    let mut uv_offset_scale = if let Some(rect) = rect {
                        let rect_size = rect.size();
                        quad_size = rect_size;
                        // Update texture size to the rect size
                        // It will help scale properly only portion of the image
                        texture_size = rect_size;
                        Vec4::new(
                            rect.min.x / batch_image_size.x,
                            rect.max.y / batch_image_size.y,
                            rect_size.x / batch_image_size.x,
                            -rect_size.y / batch_image_size.y,
                        )
                    } else {
                        Vec4::new(0.0, 1.0, 1.0, -1.0)
                    };

                    if extracted_sprite.flip_x {
                        uv_offset_scale.x += uv_offset_scale.z;
                        uv_offset_scale.z *= -1.0;
                    }
                    if extracted_sprite.flip_y {
                        uv_offset_scale.y += uv_offset_scale.w;
                        uv_offset_scale.w *= -1.0;
                    }

                    // Override the size if a custom one is specified
                    quad_size = custom_size.unwrap_or(quad_size);

                    // Used for translation of the quad if `TextureScale::Fit...` is specified.
                    let mut quad_translation = Vec2::ZERO;

                    // Scales the texture based on the `texture_scale` field.
                    if let Some(scaling_mode) = scaling_mode {
                        apply_scaling(
                            scaling_mode,
                            texture_size,
                            &mut quad_size,
                            &mut quad_translation,
                            &mut uv_offset_scale,
                        );
                    }

                    let transform = extracted_sprite.transform.affine()
                        * Affine3A::from_scale_rotation_translation(
                            quad_size.extend(1.0),
                            Quat::IDENTITY,
                            ((quad_size + quad_translation) * (-anchor - Vec2::splat(0.5)))
                                .extend(0.0),
                        );

                    // Store the vertex data and add the item to the render phase
                    sprite_meta
                        .sprite_instance_buffer
                        .push(SpriteDepthInstance::from(&transform, &uv_offset_scale));

                    current_batch.as_mut().unwrap().get_mut().range.end += 1;
                    index += 1;
                }
                ExtractedSpriteKind::Slices { ref indices } => {
                    for i in indices.clone() {
                        let slice = &extracted_slices.slices[i];
                        let rect = slice.rect;
                        let rect_size = rect.size();

                        // Calculate vertex data for this item
                        let mut uv_offset_scale: Vec4;

                        // If a rect is specified, adjust UVs and the size of the quad
                        uv_offset_scale = Vec4::new(
                            rect.min.x / batch_image_size.x,
                            rect.max.y / batch_image_size.y,
                            rect_size.x / batch_image_size.x,
                            -rect_size.y / batch_image_size.y,
                        );

                        if extracted_sprite.flip_x {
                            uv_offset_scale.x += uv_offset_scale.z;
                            uv_offset_scale.z *= -1.0;
                        }
                        if extracted_sprite.flip_y {
                            uv_offset_scale.y += uv_offset_scale.w;
                            uv_offset_scale.w *= -1.0;
                        }

                        let transform = extracted_sprite.transform.affine()
                            * Affine3A::from_scale_rotation_translation(
                                slice.size.extend(1.0),
                                Quat::IDENTITY,
                                (slice.size * -Vec2::splat(0.5) + slice.offset).extend(0.0),
                            );

                        // Store the vertex data and add the item to the render phase
                        sprite_meta
                            .sprite_instance_buffer
                            .push(SpriteDepthInstance::from(&transform, &uv_offset_scale));

                        current_batch.as_mut().unwrap().get_mut().range.end += 1;
                        index += 1;
                    }
                }
            }
            transparent_phase.items[batch_item_index]
                .batch_range_mut()
                .end += 1;
        }
        sprite_meta
            .sprite_instance_buffer
            .write_buffer(&render_device, &render_queue);

        if sprite_meta.sprite_index_buffer.len() != 6 {
            sprite_meta.sprite_index_buffer.clear();

            // NOTE: This code is creating 6 indices pointing to 4 vertices.
            // The vertices form the corners of a quad based on their two least significant bits.
            // 10   11
            //
            // 00   01
            // The sprite shader can then use the two least significant bits as the vertex index.
            // The rest of the properties to transform the vertex positions and UVs (which are
            // implicit) are baked into the instance transform, and UV offset and scale.
            // See bevy_sprite_render/src/render/sprite.wgsl for the details.
            sprite_meta.sprite_index_buffer.push(2);
            sprite_meta.sprite_index_buffer.push(0);
            sprite_meta.sprite_index_buffer.push(1);
            sprite_meta.sprite_index_buffer.push(1);
            sprite_meta.sprite_index_buffer.push(3);
            sprite_meta.sprite_index_buffer.push(2);

            sprite_meta
                .sprite_index_buffer
                .write_buffer(&render_device, &render_queue);
        }
    }
}

/// Apply scaling.
///
/// This is mostly copied from [`sprite_render`](bevy::sprite_render).
///
/// Last updated from [`bevy`]@0.18.1.
fn apply_scaling(
    scaling_mode: SpriteScalingMode,
    texture_size: Vec2,
    quad_size: &mut Vec2,
    quad_translation: &mut Vec2,
    uv_offset_scale: &mut Vec4,
) {
    let quad_ratio = quad_size.x / quad_size.y;
    let texture_ratio = texture_size.x / texture_size.y;
    let tex_quad_scale = texture_ratio / quad_ratio;
    let quad_tex_scale = quad_ratio / texture_ratio;

    match scaling_mode {
        SpriteScalingMode::FillCenter => {
            if quad_ratio > texture_ratio {
                // offset texture to center by y coordinate
                uv_offset_scale.y += (uv_offset_scale.w - uv_offset_scale.w * tex_quad_scale) * 0.5;
                // sum up scales
                uv_offset_scale.w *= tex_quad_scale;
            } else {
                // offset texture to center by x coordinate
                uv_offset_scale.x += (uv_offset_scale.z - uv_offset_scale.z * quad_tex_scale) * 0.5;
                uv_offset_scale.z *= quad_tex_scale;
            };
        }
        SpriteScalingMode::FillStart => {
            if quad_ratio > texture_ratio {
                uv_offset_scale.y += uv_offset_scale.w - uv_offset_scale.w * tex_quad_scale;
                uv_offset_scale.w *= tex_quad_scale;
            } else {
                uv_offset_scale.z *= quad_tex_scale;
            }
        }
        SpriteScalingMode::FillEnd => {
            if quad_ratio > texture_ratio {
                uv_offset_scale.w *= tex_quad_scale;
            } else {
                uv_offset_scale.x += uv_offset_scale.z - uv_offset_scale.z * quad_tex_scale;
                uv_offset_scale.z *= quad_tex_scale;
            }
        }
        SpriteScalingMode::FitCenter => {
            if texture_ratio > quad_ratio {
                // Scale based on width
                quad_size.y *= quad_tex_scale;
            } else {
                // Scale based on height
                quad_size.x *= tex_quad_scale;
            }
        }
        SpriteScalingMode::FitStart => {
            if texture_ratio > quad_ratio {
                // The quad is scaled to match the image ratio, and the quad translation is adjusted
                // to start of the quad within the original quad size.
                let scale = Vec2::new(1.0, quad_tex_scale);
                let new_quad = *quad_size * scale;
                let offset = *quad_size - new_quad;
                *quad_translation = Vec2::new(0.0, -offset.y);
                *quad_size = new_quad;
            } else {
                let scale = Vec2::new(tex_quad_scale, 1.0);
                let new_quad = *quad_size * scale;
                let offset = *quad_size - new_quad;
                *quad_translation = Vec2::new(offset.x, 0.0);
                *quad_size = new_quad;
            }
        }
        SpriteScalingMode::FitEnd => {
            if texture_ratio > quad_ratio {
                let scale = Vec2::new(1.0, quad_tex_scale);
                let new_quad = *quad_size * scale;
                let offset = *quad_size - new_quad;
                *quad_translation = Vec2::new(0.0, offset.y);
                *quad_size = new_quad;
            } else {
                let scale = Vec2::new(tex_quad_scale, 1.0);
                let new_quad = *quad_size * scale;
                let offset = *quad_size - new_quad;
                *quad_translation = Vec2::new(-offset.x, 0.0);
                *quad_size = new_quad;
            }
        }
    }
}
