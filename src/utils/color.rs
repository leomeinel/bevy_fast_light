//! Utilities related to [`Color`].

use bevy::{
    color::{Color, ColorToComponents},
    math::Vec3,
};

/// Extension of [`Color`] to add additional functionality.
pub(crate) trait ColorExt {
    /// Convert to a Vec3 scaled by `intensity`
    fn to_scaled_vec3(self, intensity: f32) -> Vec3;
}
impl ColorExt for Color {
    fn to_scaled_vec3(self, intensity: f32) -> Vec3 {
        self.to_linear().to_vec3() * intensity
    }
}
