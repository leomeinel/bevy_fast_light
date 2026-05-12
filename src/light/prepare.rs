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
            TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{CachedTexture, TextureCache},
        view::{ExtractedView, RetainedViewEntity, ViewTarget},
    },
};
use bytemuck::cast_slice;

use crate::{extract::prelude::*, light::prelude::*, occluder::prelude::*};

/// [`CachedTexture`]s for [`MeshLight`].
#[derive(Resource, Default)]
pub(crate) struct MeshLightTextures(pub(crate) HashMap<RetainedViewEntity, CachedTexture>);

/// [`Buffer`]s mapped to [`MeshLight`] [`Entity`]s.
#[derive(Resource, Default)]
pub(super) struct MeshLightUniformBuffers(pub(super) HashMap<Entity, Buffer>);

/// Prepare [`CachedTexture`]s and insert into [`MeshLightTextures`].
pub(super) fn prepare_mesh_light_texture(
    views: Query<(&ViewTarget, &ExtractedView), With<ExtractedAmbientLight2d>>,
    mut textures: ResMut<MeshLightTextures>,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
) {
    for (view_target, extracted_view) in views {
        let texture_descriptor = TextureDescriptor {
            label: Some("mesh_light_texture").into(),
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

/// Prepare [`MeshLightUniformBuffers`].
pub(super) fn prepare_mesh_light_buffers(
    light_query: Query<(Entity, &ExtractedMeshLight)>,
    mut light_buffers: ResMut<MeshLightUniformBuffers>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    light_buffers.0.clear();
    for (entity, light) in &light_query {
        let light_buffer = light_buffers.0.entry(entity).or_insert_with(|| {
            render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("mesh_light_uniform_buffer"),
                contents: cast_slice(&[*light]),
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            })
        });
        render_queue.write_buffer(light_buffer, 0, cast_slice(&[*light]));
    }
}

/// Prepare [`MeshLightFragmentBindGroups`].
pub(super) fn prepare_mesh_light_fragment_bind_groups(
    views: Query<&ExtractedView, With<ExtractedAmbientLight2d>>,
    light_query: Query<Entity, With<ExtractedMeshLight>>,
    mut light_bind_groups: ResMut<MeshLightFragmentBindGroups>,
    light_buffers: Res<MeshLightUniformBuffers>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    light_pipeline: Res<MeshLightPipeline>,
    occluder_textures: Res<OccluderTextures>,
) {
    light_bind_groups.0.clear();
    for extracted_view in &views {
        let retained_view_entity = extracted_view.retained_view_entity;
        let Some(occluder_texture) = occluder_textures.0.get(&retained_view_entity) else {
            continue;
        };

        for entity in &light_query {
            let Some(light_buffer) = light_buffers.0.get(&entity) else {
                continue;
            };

            let fragment_bind_group = render_device.create_bind_group(
                "mesh_light_fragment_bind_group",
                &pipeline_cache.get_bind_group_layout(&light_pipeline.fragment_layout),
                &BindGroupEntries::sequential((
                    &occluder_texture.default_view,
                    &light_pipeline.occluder_sampler,
                    light_buffer.as_entire_binding(),
                )),
            );
            light_bind_groups
                .0
                .insert((retained_view_entity, entity), fragment_bind_group);
        }
    }
}
