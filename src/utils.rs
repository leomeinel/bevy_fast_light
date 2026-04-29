//! Utilities to be used in the crate.

mod color;
mod prepare;

pub(crate) mod prelude {
    pub(crate) use super::color::ColorExt;
    pub(crate) use super::prepare::cached_scaled_2d_texture;
}
