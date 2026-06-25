//! [`FastLightPlugin`] and related.
//!
//! ## Render stages
//!
//! 1. Render to a texture that uses the red channel for z-levels of all [`Sprites`](bevy::sprite::Sprite)s.
//! 2. Render to a texture that uses the red channel for determining if an occluder exists and the green channel for its z-level.
//! 3. Render a light map to a texture.
//! 4. Compose from light map to screen texture.

use bevy::{
    app::{App, Plugin},
    render::{ExtractSchedule, RenderApp},
};

use crate::{
    composite::prelude::*, light::prelude::*, occluder::prelude::*, sprite_depth::prelude::*,
};

/// [`Plugin`] for fast 2D lighting.
///
/// You also need to add an [`AmbientLight2d`] to a [Camera2d](bevy::camera::Camera2d) for this to work.
///
/// Additionally you can spawn [`MeshLight`]s to light up certain areas or [`MeshOccluder`]s for light occlusion.
pub struct FastLightPlugin;
impl Plugin for FastLightPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SpriteDepthPlugin,
            OccluderPlugin,
            MeshLightPlugin,
            CompositePlugin,
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(
            ExtractSchedule,
            (
                super::extract::extract_ambient_light,
                super::extract::extract_mesh_lights,
                super::extract::extract_view_entities,
            ),
        );
    }
}
