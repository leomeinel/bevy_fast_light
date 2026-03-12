/*
 * File: texture_scale.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Scene with a gray [`Rectangle`] as background and a red [`PointLight2d`] using a lower [`FastLightPlugin::texture_scale`].

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_fast_light::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FastLightPlugin {
                // NOTE: This helps with resource usage.
                //       The default is 1. / 8.
                texture_scale: 1. / 32.,
                ..default()
            },
        ))
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
        MeshMaterial2d(materials.add(Color::from(tailwind::GRAY_500))),
    ));

    commands.spawn(PointLight2d {
        color: tailwind::RED_500.into(),
        intensity: 4.,
        outer_radius: 200.,
        ..default()
    });
}
