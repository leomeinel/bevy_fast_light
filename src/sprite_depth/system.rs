//! Render z-levels of [`Sprite`](bevy::sprite::Sprite)s to a texture from [`OccluderTextures`].

use bevy::{
    ecs::{system::Res, world::World},
    log::error,
    render::{
        render_phase::ViewSortedRenderPhases,
        render_resource::{Operations, RenderPassColorAttachment, RenderPassDescriptor},
        renderer::{RenderContext, ViewQuery},
        view::ExtractedView,
    },
};

use crate::{extract::prelude::*, occluder::prelude::*, sprite_depth::prelude::*};

/// Render z-levels of [`Sprite`](bevy::sprite::Sprite)s to a texture from [`OccluderTextures`].
pub(super) fn sprite_depth(
    world: &World,
    view: ViewQuery<(&ExtractedView, &ExtractedAmbientLight2d)>,
    mut ctx: RenderContext,
    sprite_depth_phases: Res<ViewSortedRenderPhases<SpriteDepthPhase>>,
    occluder_textures: Res<OccluderTextures>,
) {
    let view_entity = view.entity();
    let (extracted_view, _) = view.into_inner();
    let (Some(transparent_phase), Some(occluder_texture)) = (
        sprite_depth_phases.get(&extracted_view.retained_view_entity),
        occluder_textures
            .0
            .get(&extracted_view.retained_view_entity),
    ) else {
        return;
    };

    let mut render_pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("sprite_depth_render_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &occluder_texture.default_view,
            depth_slice: None,
            resolve_target: None,
            // NOTE: We need to load here because we need the value written by `sprite_depth`.
            ops: Operations::default(),
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    if let Err(err) = transparent_phase.render(&mut render_pass, world, view_entity) {
        error!("Error encountered while rendering SpriteDepthPhase {err:?}");
    }
}
