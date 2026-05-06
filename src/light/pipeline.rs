/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
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
    },
    shader::Shader,
    sprite_render::{Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform, RenderMesh2dInstances},
    utils::default,
};

use crate::light::prelude::*;

/// Pipeline that computes lighting in the shader.
#[derive(Resource)]
pub(super) struct Light2dPipeline {
    pub(super) mesh_pipeline: Mesh2dPipeline,
    pub(super) fragment_layout: BindGroupLayoutDescriptor,
    pub(super) occluder_sampler: Sampler,
    pub(super) shader: Handle<Shader>,
}
impl SpecializedMeshPipeline for Light2dPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        descriptor.label = Some("light_2d_pipeline".into());
        descriptor.layout.push(self.fragment_layout.clone());

        let fragment = descriptor.fragment.as_mut().unwrap();
        fragment.shader = self.shader.clone();
        fragment.targets = vec![Some(ColorTargetState {
            format: TextureFormat::Rgba8Unorm,
            // NOTE: This is needed since we need to alpha blend the rendered meshes.
            //       Since we are multiplying everything in `light_2d` by `attenuation`, we need `BlendState::PREMULTIPLIED_ALPHA_BLENDING`.
            blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })];

        descriptor.multisample = MultisampleState::default();
        descriptor.depth_stencil = None;

        Ok(descriptor)
    }
}
impl GetBatchData for Light2dPipeline {
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

/// Pipeline that multiplies a low resolution texture with the screen texture in the shader.
#[derive(Resource)]
pub(super) struct Light2dCompositePipeline {
    pub(super) fragment_layout: BindGroupLayoutDescriptor,
    pub(super) screen_sampler: Sampler,
    pub(super) light_sampler: Sampler,
    pub(super) pipeline_id: CachedRenderPipelineId,
}

/// Initialize [`Light2dPipeline`].
pub(super) fn init_light_2d_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mesh_pipeline: Res<Mesh2dPipeline>,
    render_device: Res<RenderDevice>,
) {
    let fragment_layout = BindGroupLayoutDescriptor::new(
        "light_2d_fragment_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<ExtractedMeshLight2d>(false),
            ),
        ),
    );

    commands.insert_resource(Light2dPipeline {
        mesh_pipeline: mesh_pipeline.clone(),
        fragment_layout,
        occluder_sampler: render_device.create_sampler(&SamplerDescriptor::default()),
        shader: load_embedded_asset!(asset_server.as_ref(), "light_2d.wgsl"),
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
    let light_sampler = render_device.create_sampler(&SamplerDescriptor {
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
        light_sampler,
        pipeline_id,
    });
}
