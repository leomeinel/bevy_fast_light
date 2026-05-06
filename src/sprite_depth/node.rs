//! [`ViewNode`]s for rendering z-levels of [`Sprite`](bevy::sprite::Sprite)s to a scalable texture.

use bevy::{
    ecs::{query::QueryItem, world::World},
    log::error,
    render::{
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_phase::ViewSortedRenderPhases,
        render_resource::{Operations, RenderPassColorAttachment, RenderPassDescriptor},
        renderer::RenderContext,
        view::ExtractedView,
    },
};

use crate::{light::prelude::*, occluder::prelude::*, sprite_depth::prelude::*};

/// [`ViewNode`] that renders the z-levels of [`Sprite`](bevy::sprite::Sprite)s to a scalable texture from [`SpriteDepthTextures`].
#[derive(Default)]
pub(super) struct SpriteDepthNode;
impl ViewNode for SpriteDepthNode {
    type ViewQuery = (&'static ExtractedView, &'static ExtractedAmbientLight2d);

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (extracted_view, _): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.view_entity();
        let sprite_depth_phases = world.resource::<ViewSortedRenderPhases<SpriteDepthPhase>>();
        let occluder_textures = world.resource::<OccluderTextures>();
        let (Some(sprite_depth_phase), Some(occluder_texture)) = (
            sprite_depth_phases.get(&extracted_view.retained_view_entity),
            occluder_textures
                .0
                .get(&extracted_view.retained_view_entity),
        ) else {
            return Ok(());
        };

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("sprite_depth_render_pass"),
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

        if let Err(err) = sprite_depth_phase.render(&mut render_pass, world, view_entity) {
            error!("Error encountered while rendering the stencil phase {err:?}");
        }

        Ok(())
    }
}
