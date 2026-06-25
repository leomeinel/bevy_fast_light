/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 * - https://github.com/jgayfer/bevy_light_2d
 */

//! Render occluders to a texture from [`OccluderTextures`].

use bevy::{
    ecs::{system::Res, world::World},
    log::error,
    render::{
        render_phase::ViewBinnedRenderPhases,
        render_resource::*,
        renderer::{RenderContext, ViewQuery},
        view::ExtractedView,
    },
};

use crate::{extract::prelude::*, occluder::prelude::*};

/// Render occluders to a texture from [`OccluderTextures`].
pub(super) fn occluder(
    world: &World,
    view: ViewQuery<(&ExtractedView, &ExtractedAmbientLight2d)>,
    mut ctx: RenderContext,
    occluder_phases: Res<ViewBinnedRenderPhases<OccluderPhase>>,
    occluder_textures: Res<OccluderTextures>,
) {
    let view_entity = view.entity();
    let (extracted_view, _) = view.into_inner();
    let (Some(occluder_phase), Some(occluder_texture)) = (
        occluder_phases.get(&extracted_view.retained_view_entity),
        occluder_textures
            .0
            .get(&extracted_view.retained_view_entity),
    ) else {
        return;
    };

    let mut render_pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("occluder_render_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &occluder_texture.default_view,
            depth_slice: None,
            resolve_target: None,
            // NOTE: We need to load here because we need the value written by `sprite_depth`.
            ops: Operations {
                load: LoadOp::Load,
                store: StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    if let Err(err) = occluder_phase.render(&mut render_pass, world, view_entity) {
        error!("Error encountered while rendering OccluderPhase {err:?}");
    }
}
