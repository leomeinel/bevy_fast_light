/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 */

//! Render pipelines for rendering [`MeshLight`](crate::prelude::MeshLight)s to a texture.

use bevy::{
    asset::{AssetServer, Handle, load_embedded_asset},
    ecs::{
        resource::Resource,
        system::{Commands, Res},
    },
    mesh::MeshVertexBufferLayoutRef,
    render::{
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::RenderDevice,
    },
    shader::Shader,
    sprite_render::{Mesh2dPipeline, Mesh2dPipelineKey},
};

use crate::extract::prelude::*;

/// Pipeline that computes lighting in the shader.
#[derive(Resource)]
pub(super) struct MeshLightPipeline {
    pub(super) mesh_pipeline: Mesh2dPipeline,
    pub(super) fragment_layout: BindGroupLayoutDescriptor,
    pub(super) occluder_sampler: Sampler,
    pub(super) shader: Handle<Shader>,
}
impl SpecializedMeshPipeline for MeshLightPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        descriptor.label = Some("mesh_light_pipeline".into());
        descriptor.layout.push(self.fragment_layout.clone());

        let fragment = descriptor.fragment.as_mut().unwrap();
        fragment.shader = self.shader.clone();
        fragment.targets = vec![Some(ColorTargetState {
            format: TextureFormat::Rgba8Unorm,
            // NOTE: This is needed since we need to alpha blend the rendered meshes.
            //       Since we are multiplying everything in `mesh_light` by `attenuation`, we need `BlendState::PREMULTIPLIED_ALPHA_BLENDING`.
            blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })];

        descriptor.multisample = MultisampleState::default();
        descriptor.depth_stencil = None;

        Ok(descriptor)
    }
}

/// Initialize [`MeshLightPipeline`].
pub(super) fn init_mesh_light_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mesh_pipeline: Res<Mesh2dPipeline>,
    render_device: Res<RenderDevice>,
) {
    let fragment_layout = BindGroupLayoutDescriptor::new(
        "mesh_light_fragment_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<ExtractedMeshLight>(false),
            ),
        ),
    );

    commands.insert_resource(MeshLightPipeline {
        mesh_pipeline: mesh_pipeline.clone(),
        fragment_layout,
        occluder_sampler: render_device.create_sampler(&SamplerDescriptor::default()),
        shader: load_embedded_asset!(asset_server.as_ref(), "mesh_light.wgsl"),
    });
}
