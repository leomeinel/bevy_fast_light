/*
 * File: utils.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Utilities to be used in the crate.

mod color;
mod prepare;

pub(crate) mod prelude {
    pub(crate) use super::color::ColorExt;
    pub(crate) use super::prepare::cached_scaled_2d_texture;
}
