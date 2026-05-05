//! Utilities to be used in the crate.

mod prepare;

pub(crate) mod prelude {
    pub(crate) use super::prepare::cached_scaled_2d_texture;
}
