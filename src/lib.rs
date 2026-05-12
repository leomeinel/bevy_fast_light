//! Simple 2D lighting for Bevy focused on performance over features.

mod composite;
mod extract;
mod light;
mod occluder;
mod plugin;
mod sprite_depth;

pub mod prelude {
    pub use crate::light::{AmbientLight2d, MeshLight};
    pub use crate::occluder::MeshOccluder;
    pub use crate::plugin::FastLightPlugin;
}
