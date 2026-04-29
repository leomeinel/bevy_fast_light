/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 */

// TODO: `light_2d` could possible just use a render phase to avoid storage buffers and
//       improve performance since this could allow us to avoid the fullscreen rendering
//       for that.

//! [`Plugin`] for rendering lights to the screen texture.

use bevy::{
    app::{App, Plugin, PostUpdate},
    asset::embedded_asset,
    camera::{
        primitives::Aabb,
        visibility::{NoFrustumCulling, VisibilitySystems},
    },
    core_pipeline::core_2d::graph::Core2d,
    ecs::{
        entity::Entity,
        query::{Changed, Or, Without},
        schedule::IntoScheduleConfigs as _,
        system::{Commands, Query},
    },
    render::{
        ExtractSchedule, Render, RenderApp, RenderStartup, RenderSystems,
        extract_component::UniformComponentPlugin,
        extract_resource::ExtractResourcePlugin,
        gpu_component_array_buffer::GpuComponentArrayBufferPlugin,
        render_graph::{RenderGraphExt, RenderLabel, ViewNodeRunner},
    },
    shader::load_shader_library,
};

use crate::{light::prelude::*, plugin::prelude::*};

/// [`Plugin`] for rendering lights to the screen texture.
pub(crate) struct Light2dPlugin;
impl Plugin for Light2dPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "types.wgsl");
        embedded_asset!(app, "light_2d.wgsl");
        embedded_asset!(app, "light_2d_composite.wgsl");

        app.add_plugins((
            ExtractResourcePlugin::<FastLightSettings>::default(),
            UniformComponentPlugin::<ExtractedAmbientLight2d>::default(),
            UniformComponentPlugin::<ExtractedLight2dMeta>::default(),
            GpuComponentArrayBufferPlugin::<ExtractedPointLight2d>::default(),
        ));

        app.add_systems(
            PostUpdate,
            update_point_light_bounds.in_set(VisibilitySystems::CalculateBounds),
        );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<Light2dTextures>();

        render_app.add_systems(
            RenderStartup,
            (
                super::pipeline::init_light_2d_pipeline,
                super::pipeline::init_light_2d_composite_pipeline,
            ),
        );

        render_app.add_systems(
            ExtractSchedule,
            (
                super::extract::extract_ambient_light,
                super::extract::extract_meta,
                super::extract::extract_point_lights,
            ),
        );

        render_app.add_systems(
            Render,
            super::prepare::prepare_light_2d_texture.in_set(RenderSystems::PrepareResources),
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

/// Update [`Aabb`] for [`PointLight2d`].
///
/// This allows [`PointLight2d`] to integrate with bevy native frustum culling.
fn update_point_light_bounds(
    light_query: Query<
        (Entity, &PointLight2d),
        (
            Or<(Changed<PointLight2d>, Without<Aabb>)>,
            Without<NoFrustumCulling>,
        ),
    >,
    mut commands: Commands,
) {
    for (entity, light) in light_query {
        commands.entity(entity).insert(light.aabb());
    }
}
