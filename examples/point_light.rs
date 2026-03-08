/*
 * File: point_light.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Scene with a green [`Rectangle`] as background and a [`PointLight2d`] of the same color.

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_fast_light::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, FastLightPlugin))
        .add_systems(Startup, setup)
        .run()
}

/// Setup scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(ClearColor(Color::WHITE));
    commands.spawn((
        Camera2d,
        // `AmbientLight2d` is required to be able to render `PointLight2d`.
        AmbientLight2d::default(),
    ));

    // Background object
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(600., 600.))),
        MeshMaterial2d(materials.add(Color::from(tailwind::GREEN_500))),
    ));

    commands.spawn(PointLight2d {
        color: tailwind::GREEN_500.into(),
        outer_radius: 200.,
        ..default()
    });
}
