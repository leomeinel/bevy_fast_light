/*
 * File: phase.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://bevy.org/examples/shaders/custom-render-phase/
 */

use std::ops::Range;

use bevy::{
    ecs::{
        entity::Entity,
        query::With,
        system::{Query, Res, ResMut},
    },
    log::error,
    math::FloatOrd,
    mesh::Mesh2d,
    render::{
        mesh::RenderMesh,
        render_asset::RenderAssets,
        render_phase::{
            CachedRenderPipelinePhaseItem, DrawFunctionId, DrawFunctions, PhaseItem,
            PhaseItemExtraIndex, SetItemPipeline, SortedPhaseItem, ViewSortedRenderPhases,
        },
        render_resource::{CachedRenderPipelineId, PipelineCache, SpecializedMeshPipelines},
        sync_world::MainEntity,
        view::{ExtractedView, Msaa, RenderVisibleEntities},
    },
    sprite_render::{
        DrawMesh2d, Mesh2dPipelineKey, RenderMesh2dInstances, SetMesh2dBindGroup,
        SetMesh2dViewBindGroup,
    },
};

use crate::{prelude::Light2dOccluder, render::pipeline::Light2dOccluderPipeline};

/// [`PhaseItem`] drawn in the render phase for light occlusion from [`Light2dOccluder`].
pub(super) struct Light2dOccluderPhaseItem {
    pub sort_key: FloatOrd,
    pub entity: (Entity, MainEntity),
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    pub batch_range: Range<u32>,
    pub extra_index: PhaseItemExtraIndex,
    pub indexed: bool,
}
impl PhaseItem for Light2dOccluderPhaseItem {
    #[inline]
    fn entity(&self) -> Entity {
        self.entity.0
    }
    #[inline]
    fn main_entity(&self) -> MainEntity {
        self.entity.1
    }
    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }
    #[inline]
    fn batch_range(&self) -> &Range<u32> {
        &self.batch_range
    }
    #[inline]
    fn batch_range_mut(&mut self) -> &mut Range<u32> {
        &mut self.batch_range
    }
    #[inline]
    fn extra_index(&self) -> PhaseItemExtraIndex {
        self.extra_index.clone()
    }
    #[inline]
    fn batch_range_and_extra_index_mut(&mut self) -> (&mut Range<u32>, &mut PhaseItemExtraIndex) {
        (&mut self.batch_range, &mut self.extra_index)
    }
}
impl SortedPhaseItem for Light2dOccluderPhaseItem {
    type SortKey = FloatOrd;
    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        self.sort_key
    }
    #[inline]
    fn indexed(&self) -> bool {
        self.indexed
    }
}
impl CachedRenderPipelinePhaseItem for Light2dOccluderPhaseItem {
    #[inline]
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline
    }
}

/// Draw function for light occlusion from [`Light2dOccluder`].
pub(super) type DrawLight2dOccluder = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMesh2dBindGroup<1>,
    DrawMesh2d,
);

/// Queue drawable entities as [`Light2dOccluderPhaseItem`]s phase items in render phases ready for sorting.
pub(super) fn queue_light_2d_occluders(
    mut views: Query<(&ExtractedView, &RenderVisibleEntities, &Msaa)>,
    mut occluder_render_phases: ResMut<ViewSortedRenderPhases<Light2dOccluderPhaseItem>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<Light2dOccluderPipeline>>,
    occluder_draw_functions: Res<DrawFunctions<Light2dOccluderPhaseItem>>,
    pipeline_cache: Res<PipelineCache>,
    occluder_draw_pipeline: Res<Light2dOccluderPipeline>,
    render_meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderMesh2dInstances>,
    has_marker: Query<(), With<Light2dOccluder>>,
) {
    for (view, visible_entities, msaa) in &mut views {
        let Some(occluder_phase) = occluder_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };
        let occluder_draw = occluder_draw_functions.read().id::<DrawLight2dOccluder>();
        let view_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples())
            | Mesh2dPipelineKey::from_hdr(view.hdr);

        for (render_entity, visible_entity) in visible_entities.iter::<Mesh2d>() {
            if has_marker.get(*render_entity).is_err() {
                continue;
            }

            let Some(mesh_instance) = render_mesh_instances.get(visible_entity) else {
                continue;
            };
            let Some(mesh) = render_meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let mesh_key =
                view_key | Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology());
            let pipeline_id = pipelines.specialize(
                &pipeline_cache,
                &occluder_draw_pipeline,
                mesh_key,
                &mesh.layout,
            );
            let pipeline_id = match pipeline_id {
                Ok(id) => id,
                Err(err) => {
                    error!("{}", err);
                    continue;
                }
            };
            let mesh_translation = &mesh_instance.transforms.world_from_local.translation;

            occluder_phase.add(Light2dOccluderPhaseItem {
                sort_key: FloatOrd(mesh_translation.z),
                entity: (*render_entity, *visible_entity),
                pipeline: pipeline_id,
                draw_function: occluder_draw,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: mesh.indexed(),
            });
        }
    }
}
