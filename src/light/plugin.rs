/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 */

//! [`Plugin`] for rendering lights to the screen texture.

use bevy::{
    app::{App, Plugin},
    asset::embedded_asset,
    core_pipeline::core_2d::graph::Core2d,
    ecs::schedule::IntoScheduleConfigs as _,
    render::{
        ExtractSchedule, Render, RenderApp, RenderDebugFlags, RenderStartup, RenderSystems,
        extract_component::UniformComponentPlugin,
        extract_resource::ExtractResourcePlugin,
        render_graph::{RenderGraphExt, RenderLabel, ViewNodeRunner},
        render_phase::{
            AddRenderCommand as _, BinnedRenderPhasePlugin, DrawFunctions, ViewBinnedRenderPhases,
        },
        render_resource::SpecializedMeshPipelines,
    },
    shader::load_shader_library,
    sprite_render::{Mesh2dPipeline, init_mesh_2d_pipeline},
};

use crate::{light::prelude::*, occluder::prelude::*, plugin::prelude::*};

/// [`Plugin`] for rendering lights to the screen texture.
pub(crate) struct Light2dPlugin;
impl Plugin for Light2dPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "types.wgsl");
        embedded_asset!(app, "light_2d.wgsl");
        embedded_asset!(app, "light_2d_composite.wgsl");

        app.add_plugins((
            BinnedRenderPhasePlugin::<Light2dPhase, Mesh2dPipeline>::new(
                RenderDebugFlags::default(),
            ),
            ExtractResourcePlugin::<FastLightSettings>::default(),
            UniformComponentPlugin::<ExtractedAmbientLight2d>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<DrawFunctions<Light2dPhase>>()
            .init_resource::<SpecializedMeshPipelines<Light2dPipeline>>()
            .init_resource::<ViewBinnedRenderPhases<Light2dPhase>>()
            .init_resource::<Light2dTextures>()
            .init_resource::<Light2dFragmentBindGroups>()
            .init_resource::<Light2dUniformBuffers>();

        render_app.add_render_command::<Light2dPhase, DrawLight2d>();

        render_app.add_systems(
            RenderStartup,
            (
                super::pipeline::init_light_2d_pipeline.after(init_mesh_2d_pipeline),
                super::pipeline::init_light_2d_composite_pipeline,
            ),
        );

        render_app.add_systems(
            ExtractSchedule,
            (
                super::extract::extract_ambient_light,
                super::extract::extract_mesh_lights,
            ),
        );

        render_app.add_systems(
            Render,
            (
                super::phase::queue_light_2ds.in_set(RenderSystems::Queue),
                (
                    super::prepare::prepare_light_2d_texture,
                    (
                        super::prepare::prepare_light_2d_buffers,
                        super::prepare::prepare_light_2d_fragment_bind_groups
                            .after(OccluderSet::PrepareTexture),
                    )
                        .chain(),
                )
                    .in_set(RenderSystems::PrepareResources),
            ),
        );

        render_app
            .add_render_graph_node::<ViewNodeRunner<Light2dNode>>(Core2d, Light2dLabel)
            .add_render_graph_node::<ViewNodeRunner<Light2dCompositeNode>>(
                Core2d,
                Light2dCompositeLabel,
            );
    }
}

/// Label for render graph edges for [`Light2dNode`].
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct Light2dLabel;

/// Label for render graph edges for [`Light2dCompositeNode`].
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct Light2dCompositeLabel;
