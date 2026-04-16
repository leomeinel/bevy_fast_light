/*
 * File: node.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! [`ViewNode`]s for rendering z-levels of [`Sprite`](bevy::sprite::Sprite)s to a scalable texture.

use bevy::{
    ecs::{query::QueryItem, world::World},
    log::error,
    render::{
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_phase::ViewSortedRenderPhases,
        render_resource::{Operations, RenderPassColorAttachment, RenderPassDescriptor},
        renderer::RenderContext,
        view::{ExtractedView, ViewTarget},
    },
};

use crate::sprite_depth::prelude::*;

/// [`ViewNode`] that renders the z-levels of [`Sprite`](bevy::sprite::Sprite)s to a scalable texture from [`SpriteDepthTextures`].
#[derive(Default)]
pub(super) struct SpriteDepthNode;
impl ViewNode for SpriteDepthNode {
    type ViewQuery = (&'static ViewTarget, &'static ExtractedView);

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (_, extracted_view): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.view_entity();
        let sprite_depth_phases = world.resource::<ViewSortedRenderPhases<SpriteDepthPhase>>();
        let sprite_depth_textures = world.resource::<SpriteDepthTextures>();
        let (Some(sprite_depth_phase), Some(sprite_depth_texture)) = (
            sprite_depth_phases.get(&extracted_view.retained_view_entity),
            sprite_depth_textures
                .0
                .get(&extracted_view.retained_view_entity),
        ) else {
            return Ok(());
        };

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("sprite_depth_render_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &sprite_depth_texture.default_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if let Err(err) = sprite_depth_phase.render(&mut render_pass, world, view_entity) {
            error!("Error encountered while rendering the stencil phase {err:?}");
        }

        Ok(())
    }
}
