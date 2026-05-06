//! Simple 2D lighting for Bevy focused on performance over features.

mod extract;
mod light;
mod occluder;
mod plugin;
mod sprite_depth;
mod utils;

pub mod prelude {
    pub use crate::light::{AmbientLight2d, MeshLight2d};
    pub use crate::occluder::MeshOccluder2d;
    pub use crate::plugin::FastLightPlugin;
}
