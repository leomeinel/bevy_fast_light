/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 */

//! [`Plugin`] for rendering light map from [`MeshLightTextures`](crate::light::prelude::MeshLightTextures) to the screen texture.

use bevy::{
    app::{App, Plugin},
    asset::embedded_asset,
    core_pipeline::{Core2d, Core2dSystems},
    ecs::schedule::IntoScheduleConfigs,
    render::{RenderApp, RenderStartup, extract_component::UniformComponentPlugin},
};

use crate::extract::prelude::*;

/// [`Plugin`] for rendering light map from [`MeshLightTextures`](crate::light::prelude::MeshLightTextures) to the screen texture.
pub(crate) struct CompositePlugin;
impl Plugin for CompositePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "composite.wgsl");

        app.add_plugins(UniformComponentPlugin::<ExtractedAmbientLight2d>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(RenderStartup, super::pipeline::init_composite_pipeline);
        render_app.add_systems(
            Core2d,
            super::system::composite.in_set(Core2dSystems::PostProcess),
        );
    }
}
