/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 */

//! Render pipelines for light occlusion.

use bevy::{
    asset::{AssetId, AssetServer, Handle, load_embedded_asset},
    ecs::{
        entity::Entity,
        resource::Resource,
        system::{Commands, Res, SystemParamItem, lifetimeless::SRes},
    },
    mesh::{Mesh, MeshVertexBufferLayoutRef},
    render::{
        batching::GetBatchData,
        mesh::{RenderMesh, allocator::MeshAllocator},
        render_asset::RenderAssets,
        render_resource::*,
        sync_world::MainEntity,
    },
    shader::Shader,
    sprite_render::{Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform, RenderMesh2dInstances},
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
            write_mask: ColorWrites::RED | ColorWrites::GREEN,
        })];

        descriptor.multisample = MultisampleState::default();
        descriptor.depth_stencil = None;

        Ok(descriptor)
    }
}
impl GetBatchData for OccluderPipeline {
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
