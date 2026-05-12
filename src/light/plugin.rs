/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 */

//! [`Plugin`] for for rendering [`MeshLight`]s to a texture.

use bevy::{
    app::{App, Plugin},
    asset::embedded_asset,
    core_pipeline::core_2d::graph::Core2d,
    ecs::schedule::IntoScheduleConfigs as _,
    render::{
        Render, RenderApp, RenderDebugFlags, RenderStartup, RenderSystems,
        render_graph::{RenderGraphExt, RenderLabel, ViewNodeRunner},
        render_phase::{
            AddRenderCommand as _, BinnedRenderPhasePlugin, DrawFunctions, ViewBinnedRenderPhases,
        },
        render_resource::SpecializedMeshPipelines,
    },
    sprite_render::{Mesh2dPipeline, init_mesh_2d_pipeline},
};

use crate::{light::prelude::*, occluder::prelude::*};

/// [`Plugin`] for rendering [`MeshLight`]s to a texture.
pub(crate) struct MeshLightPlugin;
impl Plugin for MeshLightPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "mesh_light.wgsl");

        app.add_plugins((
            BinnedRenderPhasePlugin::<MeshLightPhase, Mesh2dPipeline>::new(
                RenderDebugFlags::default(),
            ),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<DrawFunctions<MeshLightPhase>>()
            .init_resource::<SpecializedMeshPipelines<MeshLightPipeline>>()
            .init_resource::<ViewBinnedRenderPhases<MeshLightPhase>>()
            .init_resource::<MeshLightTextures>()
            .init_resource::<MeshLightFragmentBindGroups>()
            .init_resource::<MeshLightUniformBuffers>();

        render_app.add_render_command::<MeshLightPhase, DrawMeshLight>();

        render_app.add_systems(
            RenderStartup,
            super::pipeline::init_mesh_light_pipeline.after(init_mesh_2d_pipeline),
        );

        render_app.add_systems(
            Render,
            (
                super::phase::queue_mesh_lights.in_set(RenderSystems::Queue),
                (
                    super::prepare::prepare_mesh_light_texture,
                    (
                        super::prepare::prepare_mesh_light_buffers,
                        super::prepare::prepare_mesh_light_fragment_bind_groups
                            .after(OccluderSet::PrepareTexture),
                    )
                        .chain(),
                )
                    .in_set(RenderSystems::PrepareResources),
            ),
        );

        render_app.add_render_graph_node::<ViewNodeRunner<MeshLightNode>>(Core2d, MeshLightLabel);
    }
}

/// Label for render graph edges for [`MeshLightNode`].
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct MeshLightLabel;
