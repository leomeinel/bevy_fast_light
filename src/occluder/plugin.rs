/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 */

// TODO: Directly using the texture from `sprite_depth` here to modify the occluders might be a better idea.
//       This would save one texture/sample input in `light_2d` and allow us to only write to the red
//       channel in `occluder`.

//! [`Plugin`] for light occlusion.

use bevy::{
    app::{App, Plugin},
    asset::embedded_asset,
    core_pipeline::core_2d::graph::Core2d,
    ecs::schedule::IntoScheduleConfigs as _,
    render::{
        ExtractSchedule, Render, RenderApp, RenderDebugFlags, RenderStartup, RenderSystems,
        batching::no_gpu_preprocessing::batch_and_prepare_sorted_render_phase,
        extract_component::ExtractComponentPlugin,
        render_graph::{RenderGraphExt, RenderLabel, ViewNodeRunner},
        render_phase::{
            AddRenderCommand, DrawFunctions, SortedRenderPhasePlugin, ViewSortedRenderPhases,
            sort_phase_system,
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
            SortedRenderPhasePlugin::<OccluderPhase, Mesh2dPipeline>::new(
                RenderDebugFlags::default(),
            ),
            ExtractComponentPlugin::<Light2dOccluder>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<DrawFunctions<OccluderPhase>>()
            .init_resource::<SpecializedMeshPipelines<OccluderPipeline>>()
            .init_resource::<ViewSortedRenderPhases<OccluderPhase>>()
            .init_resource::<OccluderTextures>();

        render_app.add_render_command::<OccluderPhase, DrawOccluder>();

        render_app.add_systems(
            RenderStartup,
            pipeline::init_occluder_pipeline.after(init_mesh_2d_pipeline),
        );

        render_app.add_systems(ExtractSchedule, extract::extract_occluder_view_entities);

        render_app.add_systems(
            Render,
            (
                (phase::queue_occluders).in_set(RenderSystems::Queue),
                (sort_phase_system::<OccluderPhase>,).in_set(RenderSystems::PhaseSort),
                (
                    batch_and_prepare_sorted_render_phase::<OccluderPhase, OccluderPipeline>,
                    prepare::prepare_occluder_texture,
                )
                    .in_set(RenderSystems::PrepareResources),
            ),
        );

        render_app.add_render_graph_node::<ViewNodeRunner<OccluderNode>>(Core2d, OccluderLabel);
    }
}

/// Label for render graph edges for [`OccluderNode`].
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct OccluderLabel;
