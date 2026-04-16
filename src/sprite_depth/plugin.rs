/*
 * File: plugin.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Plugin for rendering z-levels of [`Sprite`](bevy::sprite::Sprite)s to a scalable texture.

use bevy::{
    app::{App, Plugin},
    asset::embedded_asset,
    core_pipeline::core_2d::graph::Core2d,
    ecs::schedule::IntoScheduleConfigs as _,
    render::{
        ExtractSchedule, Render, RenderApp, RenderStartup, RenderSystems,
        render_graph::{RenderGraphExt, RenderLabel, ViewNodeRunner},
        render_phase::{
            AddRenderCommand, DrawFunctions, ViewSortedRenderPhases, sort_phase_system,
        },
        render_resource::SpecializedRenderPipelines,
    },
};

use crate::sprite_depth::prelude::*;

/// Plugin for rendering z-levels of [`Sprite`](bevy::sprite::Sprite)s to a scalable texture.
pub(crate) struct SpriteDepthPlugin;
impl Plugin for SpriteDepthPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "sprite_depth.wgsl");

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<DrawFunctions<SpriteDepthPhase>>()
            .init_resource::<SpecializedRenderPipelines<SpriteDepthPipeline>>()
            .init_resource::<ViewSortedRenderPhases<SpriteDepthPhase>>()
            .init_resource::<SpriteDepthTextures>()
            .init_resource::<SpriteDepthMeta>()
            .init_resource::<SpriteDepthBatches>()
            .init_resource::<SpriteDepthImageBindGroups>();

        render_app.add_render_command::<SpriteDepthPhase, DrawSpriteDepth>();

        render_app.add_systems(RenderStartup, super::pipeline::init_sprite_depth_pipeline);

        render_app.add_systems(
            ExtractSchedule,
            super::extract::extract_occluder_view_entities,
        );

        render_app.add_systems(
            Render,
            (
                super::phase::queue_sprite_depths.in_set(RenderSystems::Queue),
                sort_phase_system::<SpriteDepthPhase>.in_set(RenderSystems::PhaseSort),
                (
                    super::prepare::prepare_sprite_depth_texture,
                    super::prepare::prepare_sprite_depth_view_bind_groups,
                    super::prepare::prepare_sprite_depth_image_bind_groups,
                )
                    .in_set(RenderSystems::PrepareResources),
            ),
        );

        render_app
            .add_render_graph_node::<ViewNodeRunner<SpriteDepthNode>>(Core2d, SpriteDepthLabel);
    }
}

/// Label for render graph edges for [`SpriteDepthNode`].
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct SpriteDepthLabel;
