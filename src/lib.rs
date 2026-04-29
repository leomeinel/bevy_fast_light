//! Simple 2D lighting for Bevy focused on performance over features.

mod light;
mod occluder;
mod plugin;
mod sprite_depth;
mod utils;

pub mod prelude {
    pub use crate::light::{AmbientLight2d, PointLight2d};
    pub use crate::occluder::Light2dOccluder;
    pub use crate::plugin::FastLightPlugin;
}
