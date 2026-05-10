/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 * - https://github.com/jgayfer/bevy_light_2d
 */

//! [`ViewNode`]s for rendering light map from [`MeshLightTextures`] to the screen texture.

use bevy::{
    ecs::{query::QueryItem, world::World},
    render::{
        extract_component::ComponentUniforms,
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_resource::*,
        renderer::RenderContext,
        view::{ExtractedView, ViewTarget},
    },
};

use crate::{composite::prelude::*, extract::prelude::*, light::prelude::*};

/// [`ViewNode`] that renders light map from [`MeshLightTextures`] to the screen texture.
///
/// ## Formula
///
/// (texture_output + ambient_color) * screen_texture.
#[derive(Default)]
pub(super) struct CompositeNode;
impl ViewNode for CompositeNode {
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
        let composite_pipeline = world.resource::<CompositePipeline>();
        let light_textures = world.resource::<MeshLightTextures>();
        let ambient = world.resource::<ComponentUniforms<ExtractedAmbientLight2d>>();
        let (Some(pipeline), Some(light_texture), Some(ambient)) = (
            pipeline_cache.get_render_pipeline(composite_pipeline.pipeline_id),
            light_textures.0.get(&extracted_view.retained_view_entity),
            ambient.uniforms().binding(),
        ) else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();
        let fragment_bind_group = render_context.render_device().create_bind_group(
            "composite_fragment_bind_group",
            &pipeline_cache.get_bind_group_layout(&composite_pipeline.fragment_layout),
            &BindGroupEntries::sequential((
                post_process.source,
                &composite_pipeline.screen_sampler,
                &light_texture.default_view,
                &composite_pipeline.light_sampler,
                ambient,
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("composite_render_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
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
