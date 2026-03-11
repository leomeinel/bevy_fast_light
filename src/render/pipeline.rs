/*
 * File: pipeline.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! [`Light2dPipeline`] that is being used in [`Light2dRenderPlugin`](crate::render::Light2dRenderPlugin).

use bevy::{
    asset::load_embedded_asset,
    prelude::*,
    render::{
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::RenderDevice,
        view::ViewUniform,
    },
};

use crate::render::{
    ExtractedLight2dMeta,
    extract::{ExtractedAmbientLight2d, ExtractedPointLight2d},
};

/// Pipeline that is being used in [`Light2dRenderPlugin`](crate::render::Light2dRenderPlugin).
///
/// It handles all 2D lighting.
#[derive(Resource)]
pub(super) struct Light2dPipeline {
    pub(super) vertex_layout: BindGroupLayoutDescriptor,
    pub(super) fragment_layout: BindGroupLayoutDescriptor,
    pub(super) sampler: Sampler,
    pub(super) pipeline_id: CachedRenderPipelineId,
}

/// Initialize [`Light2dPipeline`].
///
/// This adds a fragment shader that handles all 2D lighting and sets up bindings.
pub(super) fn init_light_2d_pipeline(
    mut commands: Commands,
    assets: Res<AssetServer>,
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
                uniform_buffer::<ExtractedAmbientLight2d>(false),
                uniform_buffer::<ExtractedLight2dMeta>(false),
                GpuArrayBuffer::<ExtractedPointLight2d>::binding_layout(&limits),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());

    let shader = load_embedded_asset!(assets.as_ref(), "./light.wgsl");
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
        sampler,
        pipeline_id,
    });
}
