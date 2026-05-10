/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 */

//! Render pipelines for rendering light map from [`MeshLightTextures`](crate::light::prelude::MeshLightTextures) to the screen texture.

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
    },
    utils::default,
};

use crate::extract::prelude::*;

/// Pipeline that renders light map from [`MeshLightTextures`](crate::light::prelude::MeshLightTextures) to the screen texture.
#[derive(Resource)]
pub(super) struct CompositePipeline {
    pub(super) fragment_layout: BindGroupLayoutDescriptor,
    pub(super) screen_sampler: Sampler,
    pub(super) light_sampler: Sampler,
    pub(super) pipeline_id: CachedRenderPipelineId,
}

/// Initialize [`CompositePipeline`].
pub(super) fn init_composite_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
) {
    let fragment_layout = BindGroupLayoutDescriptor::new(
        "composite_fragment_bind_group_layout",
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

    let shader = load_embedded_asset!(asset_server.as_ref(), "composite.wgsl");
    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("composite_pipeline".into()),
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

    // NOTE: We are using linear sampling here to avoid pixelated lights
    let light_sampler = render_device.create_sampler(&SamplerDescriptor {
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,
        ..default()
    });

    commands.insert_resource(CompositePipeline {
        fragment_layout,
        screen_sampler: render_device.create_sampler(&SamplerDescriptor::default()),
        light_sampler,
        pipeline_id,
    });
}
