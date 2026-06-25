//! Plugin for rendering z-levels of [`Sprite`](bevy::sprite::Sprite)s to a texture.

use bevy::{
    app::{App, Plugin},
    asset::embedded_asset,
    core_pipeline::{Core2d, Core2dSystems, core_2d::Transparent2d},
    ecs::schedule::IntoScheduleConfigs as _,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems, render_phase::AddRenderCommand,
        render_resource::SpecializedRenderPipelines,
    },
};

use crate::sprite_depth::prelude::*;

/// Plugin for rendering z-levels of [`Sprite`](bevy::sprite::Sprite)s to a texture.
pub(crate) struct SpriteDepthPlugin;
impl Plugin for SpriteDepthPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "sprite_depth.wgsl");

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedRenderPipelines<SpriteDepthPipeline>>()
            .init_resource::<SpriteDepthMeta>()
            .init_resource::<SpriteDepthBatches>()
            .init_resource::<SpriteDepthImageBindGroups>();

        render_app.add_render_command::<Transparent2d, DrawSpriteDepth>();

        render_app.add_systems(RenderStartup, super::pipeline::init_sprite_depth_pipeline);
        render_app.add_systems(
            Render,
            (
                super::phase::queue_sprite_depths.in_set(RenderSystems::Queue),
                (
                    super::prepare::prepare_sprite_depth_view_bind_groups,
                    super::prepare::prepare_sprite_depth_image_bind_groups,
                )
                    .in_set(RenderSystems::PrepareResources),
            ),
        );
        render_app.add_systems(
            Core2d,
            super::system::sprite_depth.in_set(Core2dSystems::Prepass),
        );
    }
}
