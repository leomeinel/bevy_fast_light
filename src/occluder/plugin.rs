/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 */

//! [`Plugin`] for light occlusion.

use bevy::{
    app::{App, Plugin},
    asset::embedded_asset,
    core_pipeline::core_2d::graph::Core2d,
    ecs::schedule::{IntoScheduleConfigs as _, SystemSet},
    render::{
        Render, RenderApp, RenderDebugFlags, RenderStartup, RenderSystems,
        extract_component::ExtractComponentPlugin,
        render_graph::{RenderGraphExt, RenderLabel, ViewNodeRunner},
        render_phase::{
            AddRenderCommand as _, BinnedRenderPhasePlugin, DrawFunctions, ViewBinnedRenderPhases,
        },
        render_resource::SpecializedMeshPipelines,
    },
    shader::load_shader_library,
    sprite_render::{Mesh2dPipeline, init_mesh_2d_pipeline},
};

use crate::occluder::prelude::*;

/// [`Plugin`] for light occlusion.
pub(crate) struct OccluderPlugin;
impl Plugin for OccluderPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "types.wgsl");
        embedded_asset!(app, "occluder.wgsl");

        app.add_plugins((
            BinnedRenderPhasePlugin::<OccluderPhase, Mesh2dPipeline>::new(
                RenderDebugFlags::default(),
            ),
            ExtractComponentPlugin::<MeshOccluder2d>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<DrawFunctions<OccluderPhase>>()
            .init_resource::<SpecializedMeshPipelines<OccluderPipeline>>()
            .init_resource::<ViewBinnedRenderPhases<OccluderPhase>>()
            .init_resource::<OccluderTextures>();

        render_app.add_render_command::<OccluderPhase, DrawOccluder>();

        render_app.add_systems(
            RenderStartup,
            super::pipeline::init_occluder_pipeline.after(init_mesh_2d_pipeline),
        );

        render_app.add_systems(
            Render,
            (
                super::phase::queue_occluders.in_set(RenderSystems::Queue),
                super::prepare::prepare_occluder_texture
                    .in_set(OccluderSet::PrepareTexture)
                    .in_set(RenderSystems::PrepareResources),
            ),
        );

        render_app.add_render_graph_node::<ViewNodeRunner<OccluderNode>>(Core2d, OccluderLabel);
    }
}

/// Label for render graph edges for [`OccluderNode`].
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct OccluderLabel;

/// A system set for ordering occluder systems.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum OccluderSet {
    PrepareTexture,
}
