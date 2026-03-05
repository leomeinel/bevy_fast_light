/*
 * File: ambient_light.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Ambient light for a [`Camera2d`] in a 2D environment.

// FIXME: This seems to desaturate the colors too much.

use bevy::{
    app::{App, Plugin, Update},
    asset::{AssetPath, embedded_asset, embedded_path},
    camera::Camera2d,
    color::{Color, LinearRgba},
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
    },
    ecs::{
        component::Component,
        lifecycle::Add,
        observer::On,
        query::{Changed, With},
        system::{Commands, Query},
    },
    reflect::Reflect,
    render::{
        extract_component::ExtractComponent,
        render_graph::{
            InternedRenderLabel, InternedRenderSubGraph, RenderLabel as _, RenderSubGraph as _,
        },
        render_resource::{AsBindGroup, ShaderType},
    },
    shader::ShaderRef,
};

use crate::log::error::*;

/// [`Plugin`] that configures [`AmbientLight2d`].
pub(crate) struct AmbientLight2dPlugin;
impl Plugin for AmbientLight2dPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "ambient_light.wgsl");

        app.add_plugins(FullscreenMaterialPlugin::<AmbientLight2dMaterial>::default());

        app.add_systems(Update, sync_material);

        app.add_observer(on_add);
    }
}

/// Ambient light for a [`Camera2d`] in a 2D environment.
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

/// Custom [`FullscreenMaterial`] for [`AmbientLight2d`].
#[derive(Component, ShaderType, AsBindGroup, ExtractComponent, Clone, Copy, Default, PartialEq)]
struct AmbientLight2dMaterial {
    #[uniform(0)]
    color: LinearRgba,
}
impl FullscreenMaterial for AmbientLight2dMaterial {
    fn fragment_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("ambient_light.wgsl"))
            .with_source("embedded")
            .into()
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            Node2d::Tonemapping.intern(),
            Self::node_label().intern(),
            Node2d::EndMainPassPostProcessing.intern(),
        ]
    }
    fn sub_graph() -> Option<InternedRenderSubGraph> {
        Some(Core2d.intern())
    }
}
impl From<&AmbientLight2d> for AmbientLight2dMaterial {
    fn from(value: &AmbientLight2d) -> Self {
        let color = value.color.to_linear() * value.intensity;
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
/// Inserts [`AmbientLight2dMaterial`] from [`AmbientLight2d`].
fn on_add(
    event: On<Add, AmbientLight2d>,
    query: Query<&AmbientLight2d, With<Camera2d>>,
    mut commands: Commands,
) {
    let entity = event.entity;
    let light = query.get(entity).expect(ERR_AMBIENT_NO_CAMERA);

    commands
        .entity(entity)
        .insert(AmbientLight2dMaterial::from(light));
}

/// Sync [`AmbientLight2dMaterial`] from [`AmbientLight2d`] if it has changed.
fn sync_material(
    query: Query<
        (&AmbientLight2d, &mut AmbientLight2dMaterial),
        (Changed<AmbientLight2d>, With<Camera2d>),
    >,
) {
    for (light, mut material) in query {
        material.set_if_neq(&AmbientLight2dMaterial::from(light));
    }
}
