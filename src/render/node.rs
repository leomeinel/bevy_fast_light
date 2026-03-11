/*
 * File: node.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/jgayfer/bevy_light_2d
 */

//! [`Light2dNode`] that is being used in [`Light2dRenderPlugin`](crate::render::Light2dRenderPlugin).

use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::ComponentUniforms,
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        view::{ViewTarget, ViewUniformOffset, ViewUniforms},
    },
};
use smallvec::{SmallVec, smallvec};

use crate::render::{
    ExtractedPointLight2d,
    extract::{ExtractedAmbientLight2d, Light2dMeta},
    pipeline::Light2dPipeline,
};

/// Render node used in [`Light2dRenderPlugin`](crate::render::Light2dRenderPlugin).
///
/// This updates the bind group and draws a fullscreen vertex.
#[derive(Default)]
pub(super) struct Light2dNode;
impl ViewNode for Light2dNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewUniformOffset,
        &'static ExtractedAmbientLight2d,
    );

    fn run(
        &self,
        _: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, view_offset, _): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let light_2d_pipeline = world.resource::<Light2dPipeline>();
        let ambient = world.resource::<ComponentUniforms<ExtractedAmbientLight2d>>();
        let light_meta = world.resource::<ComponentUniforms<Light2dMeta>>();
        let view = world.resource::<ViewUniforms>();
        let point_lights = world.resource::<GpuArrayBuffer<ExtractedPointLight2d>>();
        let (Some(pipeline), Some(ambient), Some(light_meta), Some(view), Some(point_lights)) = (
            pipeline_cache.get_render_pipeline(light_2d_pipeline.pipeline_id),
            ambient.uniforms().binding(),
            light_meta.uniforms().binding(),
            view.uniforms.binding(),
            point_lights.binding(),
        ) else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();
        let vertex_bind_group = render_context.render_device().create_bind_group(
            "light_2d_vertex_bind_group",
            &pipeline_cache.get_bind_group_layout(&light_2d_pipeline.vertex_layout),
            &BindGroupEntries::single(view),
        );
        let fragment_bind_group = render_context.render_device().create_bind_group(
            "light_2d_fragment_bind_group",
            &pipeline_cache.get_bind_group_layout(&light_2d_pipeline.fragment_layout),
            &BindGroupEntries::sequential((
                post_process.source,
                &light_2d_pipeline.sampler,
                ambient,
                light_meta,
                point_lights,
            )),
        );
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("light_2d_pass"),
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

        let mut fragment_offsets: SmallVec<[u32; 1]> = smallvec![];
        // NOTE: WebGL2 does not support storage buffers. `GpuArrayBuffer` chooses the correct buffer for us,
        //       but we have to add an offset for it here.
        let limits = world.resource::<RenderDevice>().limits();
        if limits.max_storage_buffers_per_shader_stage == 0 {
            fragment_offsets.push(0); // `ExtractedPointLight2d`
        }

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &vertex_bind_group, &[view_offset.offset]);
        render_pass.set_bind_group(1, &fragment_bind_group, &fragment_offsets);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
