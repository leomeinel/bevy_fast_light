/*
 * File: node.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 * - https://github.com/jgayfer/bevy_light_2d
 */

//! [`ViewNode`]s for rendering lights to the screen texture.

use bevy::{
    ecs::{query::QueryItem, world::World},
    render::{
        extract_component::ComponentUniforms,
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        view::{ExtractedView, ViewTarget, ViewUniformOffset, ViewUniforms},
    },
};
use smallvec::{SmallVec, smallvec};

use crate::{light::prelude::*, occluder::prelude::*, sprite_depth::prelude::*};

/// [`ViewNode`] that renders non-ambient lights to a texture from [`Light2dTextures`].
#[derive(Default)]
pub(super) struct Light2dNode;
impl ViewNode for Light2dNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ExtractedView,
        &'static ViewUniformOffset,
        &'static ExtractedAmbientLight2d,
    );

    fn run(
        &self,
        _: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (_, extracted_view, view_offset, _): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view = world.resource::<ViewUniforms>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let light_pipeline = world.resource::<Light2dPipeline>();
        let sprite_depth_textures = world.resource::<SpriteDepthTextures>();
        let occluder_textures = world.resource::<OccluderTextures>();
        let light_textures = world.resource::<Light2dTextures>();
        let light_meta = world.resource::<ComponentUniforms<ExtractedLight2dMeta>>();
        let point_lights = world.resource::<GpuArrayBuffer<ExtractedPointLight2d>>();
        // NOTE: `light_meta` and `point_lights` might be `None` in this `let-else`. This will result in no light updates.
        //       We can disregard that case because after the first `ExtractedPointLight2d` has been added, `light_meta`
        //       will always be correct and `GpuArrayBuffer<ExtractedPointLight2d>` will exist but
        //       might contain invalid data.
        //       If no lights were ever present, this will skip non-ambient light updates which is desired..
        let (
            Some(view),
            Some(pipeline),
            Some(sprite_depth_texture),
            Some(occluder_texture),
            Some(light_texture),
            Some(light_meta),
            Some(point_lights),
        ) = (
            view.uniforms.binding(),
            pipeline_cache.get_render_pipeline(light_pipeline.pipeline_id),
            sprite_depth_textures
                .0
                .get(&extracted_view.retained_view_entity),
            occluder_textures
                .0
                .get(&extracted_view.retained_view_entity),
            light_textures.0.get(&extracted_view.retained_view_entity),
            light_meta.uniforms().binding(),
            point_lights.binding(),
        )
        else {
            return Ok(());
        };

        let vertex_bind_group = render_context.render_device().create_bind_group(
            "light_2d_vertex_bind_group",
            &pipeline_cache.get_bind_group_layout(&light_pipeline.vertex_layout),
            &BindGroupEntries::single(view),
        );
        let fragment_bind_group = render_context.render_device().create_bind_group(
            "light_2d_fragment_bind_group",
            &pipeline_cache.get_bind_group_layout(&light_pipeline.fragment_layout),
            &BindGroupEntries::sequential((
                &sprite_depth_texture.default_view,
                &light_pipeline.sprite_depth_sampler,
                &occluder_texture.default_view,
                &light_pipeline.occluder_sampler,
                light_meta,
                point_lights,
            )),
        );

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
