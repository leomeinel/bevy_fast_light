/*
 * File: plugin.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://bevy.org/examples/shaders/custom-post-processing/
 */

//! [`Plugin`] for rendering lights to the screen texture.

use bevy::{
    app::{App, Plugin},
    asset::embedded_asset,
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    ecs::schedule::IntoScheduleConfigs as _,
    render::{
        ExtractSchedule, Render, RenderApp, RenderDebugFlags, RenderStartup, RenderSystems,
        batching::no_gpu_preprocessing::batch_and_prepare_sorted_render_phase,
        extract_component::{ExtractComponentPlugin, UniformComponentPlugin},
        extract_resource::ExtractResourcePlugin,
        gpu_component_array_buffer::GpuComponentArrayBufferPlugin,
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

use crate::{
    plugin::FastLightSettings,
    prelude::Light2dOccluder,
    render::{
        extract::{
            ExtractedAmbientLight2d, ExtractedLight2dMeta, ExtractedPointLight2d, extract_ambient,
            extract_light_meta, extract_occluder_view_entities, extract_point_lights,
        },
        node::{Light2dCompositeNode, Light2dNode, Light2dOccluderNode},
        phase::{DrawLight2dOccluder, Light2dOccluderPhaseItem, queue_light_2d_occluders},
        pipeline::{
            Light2dOccluderPipeline, init_light_2d_composite_pipeline,
            init_light_2d_occluder_pipeline, init_light_2d_pipeline,
        },
        prepare::{
            Light2dOccluderTextures, Light2dTextures, prepare_light_2d_occluder_texture,
            prepare_light_2d_texture,
        },
    },
};

/// [`Plugin`] handling all rendering of this crate.
///
/// This is responsible for rendering lights to the screen texture.
///
/// ## Actions
///
/// - Load shader libraries and embed shader assets.
/// - Prepare [`Component`](bevy::ecs::component::Component)s for [`RenderApp`].
/// - Initialize pipeline.
/// - Extract data from the main world into the render world.
/// - Add render graph nodes/edges.
pub(crate) struct FastLightRenderPlugin;
impl Plugin for FastLightRenderPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "types.wgsl");
        embedded_asset!(app, "light_2d_occluder.wgsl");
        embedded_asset!(app, "light_2d.wgsl");
        embedded_asset!(app, "light_2d_composite.wgsl");

        app.add_plugins((
            SortedRenderPhasePlugin::<Light2dOccluderPhaseItem, Mesh2dPipeline>::new(
                RenderDebugFlags::default(),
            ),
            ExtractComponentPlugin::<Light2dOccluder>::default(),
            ExtractResourcePlugin::<FastLightSettings>::default(),
            UniformComponentPlugin::<ExtractedAmbientLight2d>::default(),
            UniformComponentPlugin::<ExtractedLight2dMeta>::default(),
            GpuComponentArrayBufferPlugin::<ExtractedPointLight2d>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<DrawFunctions<Light2dOccluderPhaseItem>>()
            .init_resource::<SpecializedMeshPipelines<Light2dOccluderPipeline>>()
            .init_resource::<ViewSortedRenderPhases<Light2dOccluderPhaseItem>>()
            .init_resource::<Light2dOccluderTextures>()
            .init_resource::<Light2dTextures>();

        render_app.add_render_command::<Light2dOccluderPhaseItem, DrawLight2dOccluder>();

        render_app.add_systems(
            RenderStartup,
            (
                init_light_2d_occluder_pipeline.after(init_mesh_2d_pipeline),
                init_light_2d_pipeline,
                init_light_2d_composite_pipeline,
            ),
        );

        render_app.add_systems(
            ExtractSchedule,
            (
                extract_occluder_view_entities,
                extract_ambient,
                extract_light_meta,
                extract_point_lights,
            ),
        );

        render_app.add_systems(
            Render,
            (
                queue_light_2d_occluders.in_set(RenderSystems::Queue),
                sort_phase_system::<Light2dOccluderPhaseItem>.in_set(RenderSystems::PhaseSort),
                (
                    batch_and_prepare_sorted_render_phase::<
                        Light2dOccluderPhaseItem,
                        Light2dOccluderPipeline,
                    >,
                    prepare_light_2d_occluder_texture,
                    prepare_light_2d_texture,
                )
                    .in_set(RenderSystems::PrepareResources),
            ),
        );

        render_app
            .add_render_graph_node::<ViewNodeRunner<Light2dOccluderNode>>(
                Core2d,
                Light2dOccluderLabel,
            )
            .add_render_graph_node::<ViewNodeRunner<Light2dNode>>(Core2d, Light2dLabel)
            .add_render_graph_node::<ViewNodeRunner<Light2dCompositeNode>>(
                Core2d,
                Light2dCompositeLabel,
            )
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::MainOpaquePass,
                    Light2dOccluderLabel,
                    Node2d::MainTransparentPass,
                    Node2d::Tonemapping,
                    Light2dLabel,
                    Light2dCompositeLabel,
                    Node2d::EndMainPassPostProcessing,
                ),
            );
    }
}

/// Label for render graph edges for [`Light2dOccluderNode`].
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct Light2dOccluderLabel;

/// Label for render graph edges for [`Light2dNode`].
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct Light2dLabel;

/// Label for render graph edges for [`Light2dCompositeNode`].
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct Light2dCompositeLabel;
