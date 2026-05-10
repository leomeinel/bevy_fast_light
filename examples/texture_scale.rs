//! Scene with a green [`Rectangle`] as background and an amber [`MeshLight`] using a lower [`FastLightPlugin::texture_scale`].

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_fast_light::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FastLightPlugin {
                // NOTE: Reducing this helps with resource usage.
                texture_scale: 1. / 16.,
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
    commands.insert_resource(ClearColor(tailwind::NEUTRAL_500.into()));
    commands.spawn((
        Camera2d,
        // NOTE: `AmbientLight2d` is required to be able to render `MeshLight`.
        AmbientLight2d::default(),
    ));

    // Background object
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(600., 600.))),
        MeshMaterial2d(materials.add(Color::from(tailwind::GREEN_500))),
    ));

    commands.spawn((
        MeshLight {
            color: tailwind::AMBER_500.into(),
            // NOTE: With `AmbientLight2d` intensity at 1., you might have to increase `MeshLight` intensity.
            //       Otherwise it will not be visible, just like shining a flashlight at day vs. at night.
            intensity: 2.,
        },
        // NOTE: `Mesh2d` is required for the shape of `MeshLight`.
        Mesh2d(meshes.add(Circle::new(200.))),
    ));
}
