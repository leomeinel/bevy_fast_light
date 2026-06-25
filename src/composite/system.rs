/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 * - https://github.com/jgayfer/bevy_light_2d
 */

//! Render light map from [`MeshLightTextures`] to the screen texture.

use bevy::{
    ecs::system::{Local, Res},
    render::{
        extract_component::ComponentUniforms,
        render_resource::*,
        renderer::{RenderContext, ViewQuery},
        view::{ExtractedView, ViewTarget},
    },
};

use crate::{composite::prelude::*, extract::prelude::*, light::prelude::*, utils::prelude::*};

/// Render light map from [`MeshLightTextures`] to the screen texture.
///
/// ## Formula
///
/// (texture_output + ambient_color) * screen_texture.
pub(super) fn composite(
    view: ViewQuery<(&ViewTarget, &ExtractedView, &ExtractedAmbientLight2d)>,
    mut cache: Local<BindGroupCache>,
    mut ctx: RenderContext,
    composite_pipeline: Option<Res<CompositePipeline>>,
    pipeline_cache: Res<PipelineCache>,
    light_textures: Res<MeshLightTextures>,
    ambient: Res<ComponentUniforms<ExtractedAmbientLight2d>>,
) {
    let Some(composite_pipeline) = composite_pipeline else {
        return;
    };
    let (view_target, extracted_view, _) = view.into_inner();
    let (Some(pipeline), Some(light_texture), Some(ambient)) = (
        pipeline_cache.get_render_pipeline(composite_pipeline.pipeline_id),
        light_textures.0.get(&extracted_view.retained_view_entity),
        ambient.uniforms().binding(),
    ) else {
        return;
    };

    let post_process = view_target.post_process_write();
    let fragment_bind_group = match &mut cache.0 {
        Some((texture_id, bind_group)) if post_process.source.id() == *texture_id => bind_group,
        cached => {
            let bind_group = ctx.render_device().create_bind_group(
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

            let (_, bind_group) = cached.insert((post_process.source.id(), bind_group));
            bind_group
        }
    };

    let mut render_pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
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
            multiview_mask: None,
        });
    render_pass.set_pipeline(pipeline);
    render_pass.set_bind_group(0, fragment_bind_group, &[]);
    render_pass.draw(0..3, 0..1);
}
