/*
 * File: pipeline.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::{
    asset::{AssetServer, Handle, load_embedded_asset},
    core_pipeline::tonemapping::get_lut_bind_group_layout_entries,
    ecs::{
        resource::Resource,
        system::{Commands, Res},
    },
    mesh::VertexBufferLayout,
    render::{
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        view::ViewUniform,
    },
    shader::{Shader, ShaderDefVal},
    sprite_render::SpritePipelineKey,
    utils::default,
};

/// Custom implementation of [`SpritePipeline`](bevy::sprite_render::SpritePipeline).
///
/// This is mostly copied from [`sprite_render`](bevy::sprite_render).
///
/// Last updated from [`bevy`]@0.18.1.
#[derive(Resource)]
pub(super) struct SpriteDepthPipeline {
    pub(super) view_layout: BindGroupLayoutDescriptor,
    pub(super) material_layout: BindGroupLayoutDescriptor,
    pub(super) shader: Handle<Shader>,
}
impl SpecializedRenderPipeline for SpriteDepthPipeline {
    type Key = SpritePipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = Vec::new();
        if key.contains(SpritePipelineKey::TONEMAP_IN_SHADER) {
            shader_defs.push("TONEMAP_IN_SHADER".into());
            shader_defs.push(ShaderDefVal::UInt(
                "TONEMAPPING_LUT_TEXTURE_BINDING_INDEX".into(),
                1,
            ));
            shader_defs.push(ShaderDefVal::UInt(
                "TONEMAPPING_LUT_SAMPLER_BINDING_INDEX".into(),
                2,
            ));

            let method = key.intersection(SpritePipelineKey::TONEMAP_METHOD_RESERVED_BITS);

            if method == SpritePipelineKey::TONEMAP_METHOD_NONE {
                shader_defs.push("TONEMAP_METHOD_NONE".into());
            } else if method == SpritePipelineKey::TONEMAP_METHOD_REINHARD {
                shader_defs.push("TONEMAP_METHOD_REINHARD".into());
            } else if method == SpritePipelineKey::TONEMAP_METHOD_REINHARD_LUMINANCE {
                shader_defs.push("TONEMAP_METHOD_REINHARD_LUMINANCE".into());
            } else if method == SpritePipelineKey::TONEMAP_METHOD_ACES_FITTED {
                shader_defs.push("TONEMAP_METHOD_ACES_FITTED".into());
            } else if method == SpritePipelineKey::TONEMAP_METHOD_AGX {
                shader_defs.push("TONEMAP_METHOD_AGX".into());
            } else if method == SpritePipelineKey::TONEMAP_METHOD_SOMEWHAT_BORING_DISPLAY_TRANSFORM
            {
                shader_defs.push("TONEMAP_METHOD_SOMEWHAT_BORING_DISPLAY_TRANSFORM".into());
            } else if method == SpritePipelineKey::TONEMAP_METHOD_BLENDER_FILMIC {
                shader_defs.push("TONEMAP_METHOD_BLENDER_FILMIC".into());
            } else if method == SpritePipelineKey::TONEMAP_METHOD_TONY_MC_MAPFACE {
                shader_defs.push("TONEMAP_METHOD_TONY_MC_MAPFACE".into());
            }

            // Debanding is tied to tonemapping in the shader, cannot run without it.
            if key.contains(SpritePipelineKey::DEBAND_DITHER) {
                shader_defs.push("DEBAND_DITHER".into());
            }
        }

        let instance_rate_vertex_buffer_layout = VertexBufferLayout {
            array_stride: 64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                // @location(0) i_model_transpose_col0: vec4<f32>,
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                // @location(1) i_model_transpose_col1: vec4<f32>,
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 1,
                },
                // @location(2) i_model_transpose_col2: vec4<f32>,
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 32,
                    shader_location: 2,
                },
                // @location(3) i_uv_offset_scale: vec4<f32>,
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 48,
                    shader_location: 3,
                },
            ],
        };

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: shader_defs.clone(),
                buffers: vec![instance_rate_vertex_buffer_layout],
                ..default()
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs,
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Rgba8Unorm,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::RED,
                })],
                ..default()
            }),
            layout: vec![self.view_layout.clone(), self.material_layout.clone()],
            depth_stencil: None,
            multisample: MultisampleState::default(),
            label: Some("sprite_depth_pipeline".into()),
            ..default()
        }
    }
}

/// Initialize [`SpriteDepthPipeline`].
///
/// This is mostly copied from [`init_sprite_pipeline`](bevy::sprite_render::init_sprite_pipeline).
///
/// Last updated from [`bevy`]@0.18.1.
pub(super) fn init_sprite_depth_pipeline(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tonemapping_lut_entries = get_lut_bind_group_layout_entries();
    let view_layout = BindGroupLayoutDescriptor::new(
        "sprite_depth_view_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (
                uniform_buffer::<ViewUniform>(true),
                tonemapping_lut_entries[0].visibility(ShaderStages::FRAGMENT),
                tonemapping_lut_entries[1].visibility(ShaderStages::FRAGMENT),
            ),
        ),
    );

    let material_layout = BindGroupLayoutDescriptor::new(
        "sprite_depth_material_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
            ),
        ),
    );

    commands.insert_resource(SpriteDepthPipeline {
        view_layout,
        material_layout,
        shader: load_embedded_asset!(asset_server.as_ref(), "sprite_depth.wgsl"),
    });
}
