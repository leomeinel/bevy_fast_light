/*
 * File: pipeline.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://bevy.org/examples/shaders/custom-post-processing/
 */

//! Render pipelines for rendering lights to the screen texture.

use bevy::{
    asset::{AssetId, AssetServer, Handle, load_embedded_asset},
    core_pipeline::FullscreenShader,
    ecs::{
        entity::Entity,
        resource::Resource,
        system::{Commands, Res, SystemParamItem, lifetimeless::SRes},
    },
    image::BevyDefault as _,
    mesh::{Mesh, MeshVertexBufferLayoutRef},
    render::{
        batching::GetBatchData,
        mesh::{RenderMesh, allocator::MeshAllocator},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::RenderDevice,
        sync_world::MainEntity,
        view::ViewUniform,
    },
    shader::Shader,
    sprite_render::{Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform, RenderMesh2dInstances},
    utils::default,
};

use crate::render::extract::{
    ExtractedAmbientLight2d, ExtractedLight2dMeta, ExtractedPointLight2d,
};

// FIXME: This currently blocks all light, it should however only block light where there is nothing in front of the Mesh.
/// Pipeline that computes occluders in the shader.
#[derive(Resource, Clone)]
pub(super) struct Light2dOccluderPipeline {
    pub(super) mesh_pipeline: Mesh2dPipeline,
    pub(super) shader: Handle<Shader>,
}
impl SpecializedMeshPipeline for Light2dOccluderPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.label = Some("light_2d_occluder_pipeline".into());

        descriptor.vertex.shader = self.shader.clone();
        let fragment = descriptor.fragment.as_mut().unwrap();
        fragment.shader = self.shader.clone();
        fragment.targets = vec![Some(ColorTargetState {
            format: TextureFormat::R8Unorm,
            blend: None,
            write_mask: ColorWrites::RED,
        })];

        descriptor.multisample = MultisampleState::default();
        descriptor.depth_stencil = None;

        Ok(descriptor)
    }
}
impl GetBatchData for Light2dOccluderPipeline {
    type Param = (
        SRes<RenderMesh2dInstances>,
        SRes<RenderAssets<RenderMesh>>,
        SRes<MeshAllocator>,
    );
    type CompareData = AssetId<Mesh>;
    type BufferData = Mesh2dUniform;

    fn get_batch_data(
        (mesh_instances, _, _): &SystemParamItem<Self::Param>,
        (_, main_entity): (Entity, MainEntity),
    ) -> Option<(Self::BufferData, Option<Self::CompareData>)> {
        let mesh_instance = mesh_instances.get(&main_entity)?;
        let mesh_uniform = {
            let mesh_transforms = &mesh_instance.transforms;
            let world_from_local = mesh_transforms.world_from_local.to_transpose();
            let (local_from_world_transpose_a, local_from_world_transpose_b) =
                mesh_transforms.world_from_local.inverse_transpose_3x3();
            Mesh2dUniform {
                world_from_local,
                local_from_world_transpose_a,
                local_from_world_transpose_b,
                flags: mesh_transforms.flags,
                tag: mesh_instance.tag,
            }
        };
        Some((
            mesh_uniform,
            mesh_instance
                .automatic_batching
                .then_some(mesh_instance.mesh_asset_id),
        ))
    }
}

/// Pipeline that computes lighting in the shader.
#[derive(Resource)]
pub(super) struct Light2dPipeline {
    pub(super) vertex_layout: BindGroupLayoutDescriptor,
    pub(super) fragment_layout: BindGroupLayoutDescriptor,
    pub(super) light_2d_occluder_sampler: Sampler,
    pub(super) pipeline_id: CachedRenderPipelineId,
}

/// Pipeline that multiplies a low resolution texture with the screen texture in the shader.
#[derive(Resource)]
pub(super) struct Light2dCompositePipeline {
    pub(super) fragment_layout: BindGroupLayoutDescriptor,
    pub(super) screen_sampler: Sampler,
    pub(super) light_2d_sampler: Sampler,
    pub(super) pipeline_id: CachedRenderPipelineId,
}

/// Initialize [`Light2dOccluderPipeline`].
pub(super) fn init_light_2d_occluder_pipeline(
    mut commands: Commands,
    mesh_pipeline: Res<Mesh2dPipeline>,
    asset_server: Res<AssetServer>,
) {
    let shader = load_embedded_asset!(asset_server.as_ref(), "light_2d_occluder.wgsl");

    commands.insert_resource(Light2dOccluderPipeline {
        mesh_pipeline: mesh_pipeline.clone(),
        shader,
    });
}

/// Initialize [`Light2dPipeline`].
pub(super) fn init_light_2d_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
) {
    let limits = render_device.limits();
    let vertex_layout = BindGroupLayoutDescriptor::new(
        "light_2d_vertex_bind_group_layout",
        &BindGroupLayoutEntries::single(ShaderStages::VERTEX, uniform_buffer::<ViewUniform>(true)),
    );
    let fragment_layout = BindGroupLayoutDescriptor::new(
        "light_2d_fragment_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<ExtractedLight2dMeta>(false),
                GpuArrayBuffer::<ExtractedPointLight2d>::binding_layout(&limits),
            ),
        ),
    );

    // NOTE: We are using linear sampling here to avoid pixelated lights
    let light_2d_occluder_sampler = render_device.create_sampler(&SamplerDescriptor {
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,
        ..default()
    });

    let shader = load_embedded_asset!(asset_server.as_ref(), "light_2d.wgsl");
    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("light_2d_pipeline".into()),
        layout: vec![vertex_layout.clone(), fragment_layout.clone()],
        vertex: VertexState {
            shader: shader.clone(),
            ..default()
        },
        fragment: Some(FragmentState {
            shader,
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(Light2dPipeline {
        vertex_layout,
        fragment_layout,
        light_2d_occluder_sampler,
        pipeline_id,
    });
}

/// Initialize [`Light2dCompositePipeline`].
pub(super) fn init_light_2d_composite_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
) {
    let fragment_layout = BindGroupLayoutDescriptor::new(
        "light_2d_composite_fragment_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<ExtractedAmbientLight2d>(false),
            ),
        ),
    );

    let screen_sampler = render_device.create_sampler(&SamplerDescriptor::default());
    // NOTE: We are using linear sampling here to avoid pixelated lights
    let light_2d_sampler = render_device.create_sampler(&SamplerDescriptor {
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,
        ..default()
    });
    let shader = load_embedded_asset!(asset_server.as_ref(), "light_2d_composite.wgsl");
    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("light_2d_composite_pipeline".into()),
        layout: vec![fragment_layout.clone()],
        vertex: fullscreen_shader.to_vertex_state(),
        fragment: Some(FragmentState {
            shader,
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(Light2dCompositePipeline {
        fragment_layout,
        screen_sampler,
        light_2d_sampler,
        pipeline_id,
    });
}
