/*
 * File: render.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://bevy.org/examples/shaders/custom-post-processing/
 */

//! Render modules for rendering lights to the screen texture.

// FIXME: When despawning `AmbientLight2d`, the lighting effect does not get updated.
//        This causes lights to for example stay active on the title screen if someone
//        despawns `AmbientLight2d` when exiting gameplay.
//        Since we are now waiting for components to have changed before extracting,
//        this also triggers if `AmbientLight2d` has not been despawned, therefore
//        I need to find an actual solution for this.

mod extract;
mod node;
mod pipeline;
pub(super) mod plugin;
mod prepare;
