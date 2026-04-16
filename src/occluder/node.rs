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

//! [`ViewNode`]s for light occlusion.

use bevy::{
    ecs::{query::QueryItem, world::World},
    log::error,
    render::{
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_phase::ViewSortedRenderPhases,
        render_resource::*,
        renderer::RenderContext,
        view::{ExtractedView, ViewTarget},
    },
};

use crate::occluder::prelude::*;

/// [`ViewNode`] that renders occluders to a texture from [`OccluderTextures`].
#[derive(Default)]
pub(super) struct OccluderNode;
impl ViewNode for OccluderNode {
    type ViewQuery = (&'static ViewTarget, &'static ExtractedView);

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (_, extracted_view): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.view_entity();
        let occluder_phases = world.resource::<ViewSortedRenderPhases<OccluderPhase>>();
        let occluder_textures = world.resource::<OccluderTextures>();
        let (Some(occluder_phase), Some(occluder_texture)) = (
            occluder_phases.get(&extracted_view.retained_view_entity),
            occluder_textures
                .0
                .get(&extracted_view.retained_view_entity),
        ) else {
            return Ok(());
        };

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("occluder_render_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &occluder_texture.default_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if let Err(err) = occluder_phase.render(&mut render_pass, world, view_entity) {
            error!("Error encountered while rendering the stencil phase {err:?}");
        }

        Ok(())
    }
}
