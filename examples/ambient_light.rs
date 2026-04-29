//! Scene with a light sky colored [`AmbientLight2d`] with a lower [`AmbientLight2d::intensity`], a green [`Rectangle`] as background and an amber [`PointLight2d`].

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_fast_light::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, FastLightPlugin::default()))
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
        // NOTE: `AmbientLight2d` is required to be able to render `PointLight2d`.
        AmbientLight2d {
            color: Color::from(tailwind::SKY_200),
            intensity: 0.5,
        },
    ));

    // Background object
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(600., 600.))),
        MeshMaterial2d(materials.add(Color::from(tailwind::GREEN_500))),
    ));

    commands.spawn(PointLight2d {
        color: tailwind::AMBER_500.into(),
        intensity: 1.,
        outer_radius: 200.,
        ..default()
    });
}
