/*
 * File: render.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://bevy.org/examples/shaders/custom-post-processing/
 */

//! [`Light2dRenderPlugin`] and related structs.

// FIXME: When despawning `AmbientLight2d`, the lighting effect does not get updated.
//        This causes lights to for example stay active on the title screen if someone
//        despawns `AmbientLight2d` when exiting gameplay.

mod extract;
mod node;
mod pipeline;

use bevy::{
    asset::embedded_asset,
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        extract_component::UniformComponentPlugin,
        gpu_component_array_buffer::GpuComponentArrayBufferPlugin,
        render_graph::{RenderGraphExt, RenderLabel, ViewNodeRunner},
    },
    shader::load_shader_library,
};

use crate::render::{
    extract::{
        ExtractedAmbientLight2d, ExtractedPointLight2d, extract_ambient, extract_point_lights,
    },
    node::Light2dNode,
    pipeline::init_light_2d_pipeline,
};

/// [`Plugin`] handling all rendering of this crate.
///
/// ## Purpose
///
/// - Load shader libraries and embed shader assets.
/// - Prepare [`Component`]s for [`RenderApp`].
/// - Initialize pipeline.
/// - Extract data from the main world into the render world.
/// - Add render graph node/edges.
pub(crate) struct Light2dRenderPlugin;
impl Plugin for Light2dRenderPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "./types.wgsl");
        embedded_asset!(app, "./render/light.wgsl");

        app.add_plugins((
            UniformComponentPlugin::<ExtractedAmbientLight2d>::default(),
            GpuComponentArrayBufferPlugin::<ExtractedPointLight2d>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(RenderStartup, init_light_2d_pipeline);

        render_app.add_systems(ExtractSchedule, (extract_ambient, extract_point_lights));

        render_app
            .add_render_graph_node::<ViewNodeRunner<Light2dNode>>(Core2d, Light2dLabel)
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::Tonemapping,
                    Light2dLabel,
                    Node2d::EndMainPassPostProcessing,
                ),
            );
    }
}

/// Label for render graph edges.
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct Light2dLabel;
