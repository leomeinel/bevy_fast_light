/*
 * Heavily inspired by:
 * - https://github.com/malbernaz/bevy_lit
 */

//! Preparation [`RenderSystems`](bevy::render::RenderSystems).

use bevy::{
    ecs::{
        entity::Entity,
        query::With,
        resource::Resource,
        system::{Query, Res, ResMut},
    },
    platform::collections::HashMap,
    render::{
        render_resource::{
            BindGroupEntries, Buffer, BufferInitDescriptor, BufferUsages, PipelineCache,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{CachedTexture, TextureCache},
        view::{ExtractedView, RetainedViewEntity, ViewTarget},
    },
};
use bytemuck::cast_slice;

use crate::{light::prelude::*, occluder::prelude::*, plugin::prelude::*, utils::prelude::*};

/// [`CachedTexture`]s for [MeshLight2d].
#[derive(Resource, Default)]
pub(super) struct Light2dTextures(pub(super) HashMap<RetainedViewEntity, CachedTexture>);

/// [`Buffer`]s mapped to [`MeshLight2d`] [`Entity`]s.
#[derive(Resource, Default)]
pub(super) struct Light2dUniformBuffers(pub(super) HashMap<Entity, Buffer>);

/// Prepare scaled [`CachedTexture`]s and insert into [`Light2dTextures`].
pub(super) fn prepare_light_2d_texture(
    views: Query<(&ViewTarget, &ExtractedView)>,
    mut textures: ResMut<Light2dTextures>,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    settings: Res<FastLightSettings>,
) {
    for (view_target, extracted_view) in views {
        let texture = cached_scaled_2d_texture(
            &mut texture_cache,
            &render_device,
            &settings,
            view_target,
            "light_2d_texture",
        );

        textures
            .0
            .insert(extracted_view.retained_view_entity, texture);
    }
}

/// Prepare [`Light2dUniformBuffers`].
pub(super) fn prepare_light_2d_buffers(
    light_query: Query<(Entity, &ExtractedMeshLight2d)>,
    mut light_buffers: ResMut<Light2dUniformBuffers>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    light_buffers.0.clear();
    for (entity, light) in &light_query {
        let light_buffer = light_buffers.0.entry(entity).or_insert_with(|| {
            render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("light_2d_uniform_buffer"),
                contents: cast_slice(&[*light]),
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            })
        });
        render_queue.write_buffer(light_buffer, 0, cast_slice(&[*light]));
    }
}

/// Prepare [`Light2dFragmentBindGroups`].
pub(super) fn prepare_light_2d_fragment_bind_groups(
    views: Query<&ExtractedView>,
    light_query: Query<Entity, With<ExtractedMeshLight2d>>,
    mut light_bind_groups: ResMut<Light2dFragmentBindGroups>,
    light_buffers: Res<Light2dUniformBuffers>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    light_pipeline: Res<Light2dPipeline>,
    occluder_textures: Res<OccluderTextures>,
) {
    light_bind_groups.0.clear();
    for extracted_view in &views {
        let Some(occluder_texture) = occluder_textures
            .0
            .get(&extracted_view.retained_view_entity)
        else {
            continue;
        };

        for entity in &light_query {
            let Some(light_buffer) = light_buffers.0.get(&entity) else {
                continue;
            };

            let fragment_bind_group = render_device.create_bind_group(
                "light_2d_fragment_bind_group",
                &pipeline_cache.get_bind_group_layout(&light_pipeline.fragment_layout),
                &BindGroupEntries::sequential((
                    &occluder_texture.default_view,
                    &light_pipeline.occluder_sampler,
                    light_buffer.as_entire_binding(),
                )),
            );
            light_bind_groups.0.insert(entity, fragment_bind_group);
        }
    }
}
