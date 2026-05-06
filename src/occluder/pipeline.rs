/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 */

//! Render pipelines for light occlusion.

use bevy::{
    asset::{AssetServer, Handle, load_embedded_asset},
    ecs::{
        resource::Resource,
        system::{Commands, Res},
    },
    mesh::MeshVertexBufferLayoutRef,
    render::render_resource::*,
    shader::Shader,
    sprite_render::{Mesh2dPipeline, Mesh2dPipelineKey},
};

/// Pipeline that computes occluders in the shader.
#[derive(Resource, Clone)]
pub(super) struct OccluderPipeline {
    pub(super) mesh_pipeline: Mesh2dPipeline,
    pub(super) shader: Handle<Shader>,
}
impl SpecializedMeshPipeline for OccluderPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.label = Some("occluder_pipeline".into());

        descriptor.vertex.shader = self.shader.clone();
        let fragment = descriptor.fragment.as_mut().unwrap();
        fragment.shader = self.shader.clone();
        fragment.targets = vec![Some(ColorTargetState {
            format: TextureFormat::Rgba8Unorm,
            blend: None,
            write_mask: ColorWrites::GREEN | ColorWrites::BLUE,
        })];

        descriptor.multisample = MultisampleState::default();
        descriptor.depth_stencil = None;

        Ok(descriptor)
    }
}

/// Initialize [`OccluderPipeline`].
pub(super) fn init_occluder_pipeline(
    mut commands: Commands,
    mesh_pipeline: Res<Mesh2dPipeline>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(OccluderPipeline {
        mesh_pipeline: mesh_pipeline.clone(),
        shader: load_embedded_asset!(asset_server.as_ref(), "occluder.wgsl"),
    });
}
