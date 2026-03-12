/*
 * File: pipeline.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Render pipelines for rendering lights to the screen texture.

use bevy::{
    asset::{AssetServer, load_embedded_asset},
    core_pipeline::FullscreenShader,
    ecs::{
        resource::Resource,
        system::{Commands, Res},
    },
    image::BevyDefault as _,
    render::{
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::RenderDevice,
        view::ViewUniform,
    },
    utils::default,
};

use crate::render::extract::{
    ExtractedAmbientLight2d, ExtractedLight2dMeta, ExtractedPointLight2d,
};

/// Pipeline that computes lighting in the shader.
#[derive(Resource)]
pub(super) struct Light2dPipeline {
    pub(super) vertex_layout: BindGroupLayoutDescriptor,
    pub(super) fragment_layout: BindGroupLayoutDescriptor,
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
                uniform_buffer::<ExtractedLight2dMeta>(false),
                GpuArrayBuffer::<ExtractedPointLight2d>::binding_layout(&limits),
            ),
        ),
    );

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
