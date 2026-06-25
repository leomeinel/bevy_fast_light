/*
 * Heavily inspired by:
 * - https://bevy.org/examples/shaders/custom-render-phase/
 * - https://github.com/bevyengine/bevy/blob/main/crates/bevy_core_pipeline/src/core_2d/mod.rs
 */

//! [`PhaseItem`]s and related for rendering [`MeshLight`]s to a texture.

use std::ops::Range;

use bevy::{
    ecs::{
        entity::Entity,
        query::With,
        resource::Resource,
        system::{Query, Res, ResMut},
    },
    log::error,
    mesh::Mesh2d,
    prelude::{Deref, DerefMut},
    render::{
        camera::{DirtySpecializations, PendingQueues},
        mesh::RenderMesh,
        render_asset::RenderAssets,
        render_phase::{
            BinnedPhaseItem, BinnedRenderPhaseType, CachedRenderPipelinePhaseItem, DrawFunctionId,
            DrawFunctions, InputUniformIndex, PhaseItem, PhaseItemBatchSetKey, PhaseItemExtraIndex,
            SetItemPipeline, ViewBinnedRenderPhases,
        },
        render_resource::{CachedRenderPipelineId, PipelineCache, SpecializedMeshPipelines},
        sync_world::MainEntity,
        view::{ExtractedView, RenderVisibleEntities},
    },
    sprite_render::{
        DrawMesh2d, Mesh2dPipelineKey, RenderMesh2dInstances, SetMesh2dBindGroup,
        SetMesh2dViewBindGroup, ViewKeyCache,
    },
};

use crate::{extract::prelude::*, light::prelude::*};

/// [`PhaseItem`] drawn in the render phase for rendering [`MeshLight`]s to a texture.
pub(crate) struct MeshLightPhase {
    #[allow(dead_code)]
    pub(crate) batch_set_key: MeshLightBatchSetKey,
    pub(crate) bin_key: MeshLightBinKey,
    pub(crate) representative_entity: (Entity, MainEntity),
    pub(crate) batch_range: Range<u32>,
    pub(crate) extra_index: PhaseItemExtraIndex,
}
impl PhaseItem for MeshLightPhase {
    #[inline]
    fn entity(&self) -> Entity {
        self.representative_entity.0
    }
    #[inline]
    fn main_entity(&self) -> MainEntity {
        self.representative_entity.1
    }
    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.bin_key.draw_function
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
impl BinnedPhaseItem for MeshLightPhase {
    type BinKey = MeshLightBinKey;

    type BatchSetKey = MeshLightBatchSetKey;

    fn new(
        batch_set_key: Self::BatchSetKey,
        bin_key: Self::BinKey,
        representative_entity: (Entity, MainEntity),
        batch_range: Range<u32>,
        extra_index: PhaseItemExtraIndex,
    ) -> Self {
        Self {
            batch_set_key,
            bin_key,
            representative_entity,
            batch_range,
            extra_index,
        }
    }
}
impl CachedRenderPipelinePhaseItem for MeshLightPhase {
    #[inline]
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.bin_key.pipeline
    }
}

/// Batch set key for [`MeshLightPhase`].
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub struct MeshLightBatchSetKey {
    indexed: bool,
}
impl PhaseItemBatchSetKey for MeshLightBatchSetKey {
    fn indexed(&self) -> bool {
        self.indexed
    }
}

/// Bin key for [`MeshLightPhase`].
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshLightBinKey {
    pipeline: CachedRenderPipelineId,
    draw_function: DrawFunctionId,
}

/// Draw function for rendering [`MeshLight`]s to a texture.
pub(super) type DrawMeshLight = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMesh2dBindGroup<1>,
    SetMeshLightFragmentBindGroup<2>,
    DrawMesh2d,
);

/// [`PendingQueues`] for [`MeshLight`].
#[derive(Default, Deref, DerefMut, Resource)]
pub(super) struct PendingLightQueues(PendingQueues);

/// Queue drawable entities for [`ViewBinnedRenderPhases<MeshLightPhase>`].
pub(super) fn queue_mesh_lights(
    mut views: Query<(&ExtractedView, &RenderVisibleEntities), With<ExtractedAmbientLight2d>>,
    has_marker: Query<(), With<ExtractedMeshLight>>,
    mut light_render_phases: ResMut<ViewBinnedRenderPhases<MeshLightPhase>>,
    mut pending_queues: ResMut<PendingLightQueues>,
    mut pipelines: ResMut<SpecializedMeshPipelines<MeshLightPipeline>>,
    dirty_specializations: Res<DirtySpecializations>,
    light_draw_functions: Res<DrawFunctions<MeshLightPhase>>,
    pipeline_cache: Res<PipelineCache>,
    light_draw_pipeline: Res<MeshLightPipeline>,
    render_meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderMesh2dInstances>,
    view_key_cache: Res<ViewKeyCache>,
) {
    for (view, visible_entities) in &mut views {
        let (Some(phase), Some(view_key), Some(render_visible_mesh_entities)) = (
            light_render_phases.get_mut(&view.retained_view_entity),
            view_key_cache.get(&view.retained_view_entity.main_entity),
            visible_entities.get::<Mesh2d>(),
        ) else {
            continue;
        };
        let draw_function = light_draw_functions.read().id::<DrawMeshLight>();
        let view_pending_queues = pending_queues.prepare_for_new_frame(view.retained_view_entity);
        for &main_entity in dirty_specializations
            .iter_to_dequeue(view.retained_view_entity, render_visible_mesh_entities)
        {
            phase.remove(main_entity);
        }

        for (render_entity, visible_entity) in dirty_specializations.iter_to_queue(
            view.retained_view_entity,
            render_visible_mesh_entities,
            &view_pending_queues.prev_frame,
        ) {
            if has_marker.get(*render_entity).is_err() {
                continue;
            }
            let Some(mesh_instance) = render_mesh_instances.get(visible_entity) else {
                view_pending_queues
                    .current_frame
                    .insert((*render_entity, *visible_entity));
                continue;
            };
            let Some(mesh) = render_meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };

            let mesh_key = *view_key
                | Mesh2dPipelineKey::from_primitive_topology_and_strip_index(
                    mesh.primitive_topology(),
                    mesh.index_format(),
                );
            let pipeline_id = pipelines.specialize(
                &pipeline_cache,
                &light_draw_pipeline,
                mesh_key,
                &mesh.layout,
            );
            let pipeline = match pipeline_id {
                Ok(id) => id,
                Err(err) => {
                    error!("{}", err);
                    continue;
                }
            };
            let batch_set_key = MeshLightBatchSetKey {
                indexed: mesh.indexed(),
            };
            let bin_key = MeshLightBinKey {
                pipeline,
                draw_function,
            };
            phase.add(
                batch_set_key,
                bin_key,
                (*render_entity, *visible_entity),
                InputUniformIndex::default(),
                // NOTE: We can't use `BinnedRenderPhaseType::BatchableMesh` because we are passing per object uniforms.
                BinnedRenderPhaseType::UnbatchableMesh,
            );
        }
    }
}
