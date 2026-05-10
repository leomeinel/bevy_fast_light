/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-post-processing/
 * - https://github.com/jgayfer/bevy_light_2d
 */

//! [`ViewNode`]s for rendering [`MeshLight`]s to a scalable texture.

use bevy::{
    ecs::{query::QueryItem, world::World},
    log::error,
    render::{
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_phase::ViewBinnedRenderPhases,
        render_resource::*,
        renderer::RenderContext,
        view::ExtractedView,
    },
};

use crate::{extract::prelude::*, light::prelude::*};

/// [`ViewNode`] that renders non-ambient lights to a scalable texture from [`MeshLightTextures`].
#[derive(Default)]
pub(super) struct MeshLightNode;
impl ViewNode for MeshLightNode {
    type ViewQuery = (&'static ExtractedView, &'static ExtractedAmbientLight2d);

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (extracted_view, _): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.view_entity();
        let light_phases = world.resource::<ViewBinnedRenderPhases<MeshLightPhase>>();
        let light_textures = world.resource::<MeshLightTextures>();
        let (Some(light_phase), Some(light_texture)) = (
            light_phases.get(&extracted_view.retained_view_entity),
            light_textures.0.get(&extracted_view.retained_view_entity),
        ) else {
            return Ok(());
        };

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
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
        });

        if let Err(err) = light_phase.render(&mut render_pass, world, view_entity) {
            error!("Error for light_phase in MeshLightNode {err:?}");
        }

        Ok(())
    }
}
