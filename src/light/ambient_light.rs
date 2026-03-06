/*
 * File: ambient_light.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Ambient light that uses multiplicative blending for a [`Camera2d`] in a 2D environment.

use bevy::{
    app::{App, Plugin, Update},
    asset::{Asset, AssetPath, Assets, embedded_asset, embedded_path},
    camera::{Camera, Camera2d},
    color::{Alpha as _, Color, LinearRgba},
    ecs::{
        component::Component,
        lifecycle::Add,
        observer::On,
        query::{Changed, With},
        system::{Commands, Query, ResMut},
    },
    math::primitives::Rectangle,
    mesh::{Mesh, Mesh2d, MeshVertexBufferLayoutRef},
    reflect::{Reflect, TypePath},
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dKey, Material2dPlugin, MeshMaterial2d},
};

use crate::{light::BLEND_MULTIPLY, log::error::ERR_AMBIENT_NO_CAMERA};

/// [`Plugin`] that configures [`AmbientLight2d`].
pub(crate) struct AmbientLight2dPlugin;
impl Plugin for AmbientLight2dPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "ambient_light.wgsl");

        app.add_plugins(Material2dPlugin::<AmbientLight2dMaterial>::default());

        app.add_systems(Update, (sync_material, sync_mesh));

        app.add_observer(on_add);
    }
}

/// Ambient light that uses multiplicative blending for a [`Camera2d`] in a 2D environment.
#[derive(Component, Reflect, Clone)]
pub struct AmbientLight2d {
    /// The [`Color`] of the light.
    pub color: Color,
    /// The intensity of the light.
    pub intensity: f32,
}
impl Default for AmbientLight2d {
    /// This is visually the same as without [`AmbientLight2d`].
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.,
        }
    }
}

/// Custom [`Material2d`] for [`AmbientLight2d`].
///
/// This is customized for multiplicative blending from [`BLEND_MULTIPLY`].
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Copy, Default, PartialEq)]
struct AmbientLight2dMaterial {
    #[uniform(0)]
    color: LinearRgba,
}
impl Material2d for AmbientLight2dMaterial {
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
    fn depth_bias(&self) -> f32 {
        // NOTE: This will override anything except a `depth_bias` of `f32::INFINITY`.
        f32::MAX
    }

    fn fragment_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("ambient_light.wgsl"))
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
            target_state.blend = Some(BLEND_MULTIPLY);
        }

        Ok(())
    }
}
impl From<&AmbientLight2d> for AmbientLight2dMaterial {
    fn from(value: &AmbientLight2d) -> Self {
        let color = (value.color.to_linear() * value.intensity).with_alpha(1.);
        Self { color }
    }
}
impl AmbientLight2dMaterial {
    fn set_if_neq(&mut self, new: &AmbientLight2dMaterial) {
        if self != new {
            *self = *new;
        }
    }
}

/// Visualize [`AmbientLight2d`] after it has been added.
///
/// ## Actions
///
/// - Creates [`AmbientLight2dMaterial`] from [`AmbientLight2d`].
/// - Inserts according [`Mesh2d`] and [`MeshMaterial2d`].
fn on_add(
    event: On<Add, AmbientLight2d>,
    query: Query<&AmbientLight2d, With<Camera2d>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<AmbientLight2dMaterial>>,
) {
    let entity = event.entity;
    let light = query.get(entity).expect(ERR_AMBIENT_NO_CAMERA);
    let mesh = meshes.add(Rectangle::default());
    let material = materials.add(AmbientLight2dMaterial::from(light));

    commands
        .entity(entity)
        .insert((Mesh2d(mesh), MeshMaterial2d(material)));
}

/// Sync [`AmbientLight2dMaterial`] from [`AmbientLight2d`] if it has changed.
fn sync_material(
    query: Query<
        (&AmbientLight2d, &mut MeshMaterial2d<AmbientLight2dMaterial>),
        Changed<AmbientLight2d>,
    >,
    mut materials: ResMut<Assets<AmbientLight2dMaterial>>,
) {
    for (light, material) in query {
        let material = materials.get_mut(&material.0).unwrap();
        material.set_if_neq(&AmbientLight2dMaterial::from(light));
    }
}

/// Sync [`Mesh2d`] of [`AmbientLight2d`] from [`Camera::logical_viewport_size()`].
fn sync_mesh(
    camera_query: Query<&Camera, (Changed<Camera>, With<AmbientLight2d>)>,
    light_query: Query<&mut Mesh2d, With<AmbientLight2d>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for camera in camera_query {
        let Some(viewport_size) = camera.logical_viewport_size() else {
            continue;
        };
        for mesh in &light_query {
            let mesh = meshes.get_mut(&mesh.0).unwrap();
            *mesh = Mesh::from(Rectangle::from_size(viewport_size));
        }
    }
}
