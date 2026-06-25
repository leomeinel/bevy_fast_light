/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 * - https://bevy.org/examples/shaders/custom-render-phase/
 * - https://github.com/jgayfer/bevy_light_2d
 */

//! Render mesh lights to a texture from [`MeshLightTextures`].

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

use crate::{extract::prelude::*, light::prelude::*};

/// Render mesh lights to a texture from [`MeshLightTextures`]
pub(super) fn light(
    world: &World,
    view: ViewQuery<(&ExtractedView, &ExtractedAmbientLight2d)>,
    mut ctx: RenderContext,
    light_phases: Res<ViewBinnedRenderPhases<MeshLightPhase>>,
    light_textures: Res<MeshLightTextures>,
) {
    let view_entity = view.entity();
    let (extracted_view, _) = view.into_inner();
    let (Some(light_phase), Some(light_texture)) = (
        light_phases.get(&extracted_view.retained_view_entity),
        light_textures.0.get(&extracted_view.retained_view_entity),
    ) else {
        return;
    };

    let mut render_pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("mesh_light_render_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &light_texture.default_view,
            depth_slice: None,
            resolve_target: None,
            ops: Operations::default(),
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    if let Err(err) = light_phase.render(&mut render_pass, world, view_entity) {
        error!("Error encountered while rendering MeshLightPhase {err:?}");
    }
}
