/*
 * File: point_light.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/malbernaz/bevy_lit
 */

//! Point light that uses additive blending to simulate light in a 2D environment.

// FIXME: If the ambient light is very low, the color of this light overpowers the color behind it too much.

use bevy::{
    app::{App, Plugin, Update},
    asset::{Asset, AssetPath, Assets, embedded_asset, embedded_path},
    color::{Alpha as _, Color, LinearRgba},
    ecs::{
        component::Component,
        lifecycle::Add,
        observer::On,
        query::Changed,
        system::{Commands, Query, ResMut},
    },
    math::{FloatPow as _, primitives::Circle},
    mesh::{Mesh, Mesh2d, MeshVertexBufferLayoutRef},
    reflect::{Reflect, TypePath},
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dKey, Material2dPlugin, MeshMaterial2d},
};

use crate::light::BLEND_ADD;

/// [`Plugin`] that configures [`PointLight2d`].
pub(crate) struct PointLight2dPlugin;
impl Plugin for PointLight2dPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "point_light.wgsl");
        app.add_plugins(Material2dPlugin::<PointLight2dMaterial>::default());

        app.add_systems(Update, sync_material);

        app.add_observer(on_add);
    }
}

/// Point light that uses additive blending to simulate light in a 2D environment.
#[derive(Component, Reflect, Clone)]
pub struct PointLight2d {
    // FIXME: Implement this!
    /// Whether the light should cast shadows.
    ///
    /// NOTE: This has not been implemented yet!
    pub cast_shadows: bool,
    /// The [`Color`] of the light.
    pub color: Color,
    /// The intensity of the light.
    pub intensity: f32,
    /// The inner radius of the light.
    ///
    /// This will always use `intensity`.
    pub inner_radius: f32,
    /// The outer radius of the light.
    ///
    /// `intensity` will degrade outwards.
    pub outer_radius: f32,
}
impl Default for PointLight2d {
    fn default() -> Self {
        Self {
            cast_shadows: false,
            color: Color::WHITE,
            intensity: 1.,
            inner_radius: 0.,
            outer_radius: 64.,
        }
    }
}

/// Custom [`Material2d`] for [`PointLight2d`].
///
/// This is customized for additive blending from [`BLEND_ADD`].
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Copy, Default, PartialEq)]
struct PointLight2dMaterial {
    #[uniform(0)]
    cast_shadows: u32,
    #[uniform(0)]
    color: LinearRgba,
    #[uniform(0)]
    inner_radius_sq: f32,
    #[uniform(0)]
    outer_radius_sq: f32,
    #[uniform(0)]
    inv_outer_radius_sq: f32,
}
impl Material2d for PointLight2dMaterial {
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
    fn depth_bias(&self) -> f32 {
        // NOTE: This will override anything.
        f32::INFINITY
    }

    fn fragment_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("point_light.wgsl"))
            .with_source("embedded")
            .into()
    }
    fn vertex_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("point_light.wgsl"))
            .with_source("embedded")
            .into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment
            && let Some(target_state) = &mut fragment.targets[0]
        {
            target_state.blend = Some(BLEND_ADD);
        }

        Ok(())
    }
}
impl From<&PointLight2d> for PointLight2dMaterial {
    fn from(value: &PointLight2d) -> Self {
        let cast_shadows = if value.cast_shadows { 1 } else { 0 };
        let color = (value.color.to_linear() * value.intensity).with_alpha(1.);
        let inner_radius_sq = value.inner_radius.squared();
        let outer_radius_sq = value.outer_radius.squared();
        let inv_outer_radius_sq = 1. / outer_radius_sq;

        Self {
            cast_shadows,
            color,
            inner_radius_sq,
            outer_radius_sq,
            inv_outer_radius_sq,
        }
    }
}
impl PointLight2dMaterial {
    fn set_if_neq(&mut self, new: &PointLight2dMaterial) {
        if self != new {
            *self = *new;
        }
    }
}

/// Visualize [`PointLight2d`] after it has been added.
///
/// ## Actions
///
/// - Creates a [`Circle`] from [`PointLight2d::outer_radius`].
/// - Creates [`PointLight2dMaterial`] from [`PointLight2d`].
/// - Inserts according [`Mesh2d`] and [`MeshMaterial2d`].
fn on_add(
    event: On<Add, PointLight2d>,
    query: Query<&PointLight2d>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PointLight2dMaterial>>,
) {
    let entity = event.entity;
    // NOTE: unwrap should never fail here and is therefore safe.
    let light = query.get(entity).unwrap();
    let mesh = meshes.add(Circle::new(light.outer_radius));
    let material = materials.add(PointLight2dMaterial::from(light));

    commands
        .entity(entity)
        .insert((Mesh2d(mesh), MeshMaterial2d(material)));
}

/// Sync [`PointLight2dMaterial`] from [`PointLight2d`] if it has changed.
fn sync_material(
    query: Query<(&PointLight2d, &mut MeshMaterial2d<PointLight2dMaterial>), Changed<PointLight2d>>,
    mut materials: ResMut<Assets<PointLight2dMaterial>>,
) {
    for (light, material) in query {
        let material = materials.get_mut(&material.0).unwrap();
        material.set_if_neq(&PointLight2dMaterial::from(light));
    }
}
