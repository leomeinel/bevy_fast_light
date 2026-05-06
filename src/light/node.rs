/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 * - https://github.com/jgayfer/bevy_light_2d
 */

//! [`ViewNode`]s for rendering lights to the screen texture.

use bevy::{
    ecs::{query::QueryItem, world::World},
    log::error,
    render::{
        extract_component::ComponentUniforms,
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_phase::ViewSortedRenderPhases,
        render_resource::*,
        renderer::RenderContext,
        view::{ExtractedView, ViewTarget},
    },
};

use crate::light::prelude::*;

/// [`ViewNode`] that renders non-ambient lights to a texture from [`Light2dTextures`].
#[derive(Default)]
pub(super) struct Light2dNode;
impl ViewNode for Light2dNode {
    type ViewQuery = (&'static ExtractedView, &'static ExtractedAmbientLight2d);

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (extracted_view, _): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.view_entity();
        let light_phases = world.resource::<ViewSortedRenderPhases<Light2dPhase>>();
        let light_textures = world.resource::<Light2dTextures>();
        let (Some(light_phase), Some(light_texture)) = (
            light_phases.get(&extracted_view.retained_view_entity),
            light_textures.0.get(&extracted_view.retained_view_entity),
        ) else {
            return Ok(());
        };

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("light_2d_render_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &light_texture.default_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if let Err(err) = light_phase.render(&mut render_pass, world, view_entity) {
            error!("Error for light_phase in Light2dNode {err:?}");
        }

        Ok(())
    }
}

/// [`ViewNode`] that renders to the screen texture.
///
/// ## Formula
///
/// (texture_output + ambient_color) * screen_texture.
///
/// ## Note
///
/// texture_output is from [`Light2dNode`].
#[derive(Default)]
pub(super) struct Light2dCompositeNode;
impl ViewNode for Light2dCompositeNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ExtractedView,
        &'static ExtractedAmbientLight2d,
    );

    fn run(
        &self,
        _: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, extracted_view, _): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let light_composite_pipeline = world.resource::<Light2dCompositePipeline>();
        let light_textures = world.resource::<Light2dTextures>();
        let ambient = world.resource::<ComponentUniforms<ExtractedAmbientLight2d>>();
        let (Some(pipeline), Some(light_texture), Some(ambient)) = (
            pipeline_cache.get_render_pipeline(light_composite_pipeline.pipeline_id),
            light_textures.0.get(&extracted_view.retained_view_entity),
            ambient.uniforms().binding(),
        ) else {
            return Ok(());
        };

        let screen_texture = view_target.post_process_write();
        let fragment_bind_group = render_context.render_device().create_bind_group(
            "light_2d_composite_fragment_bind_group",
            &pipeline_cache.get_bind_group_layout(&light_composite_pipeline.fragment_layout),
            &BindGroupEntries::sequential((
                screen_texture.source,
                &light_composite_pipeline.screen_sampler,
                &light_texture.default_view,
                &light_composite_pipeline.light_sampler,
                ambient,
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("light_2d_composite_render_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: screen_texture.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &fragment_bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
