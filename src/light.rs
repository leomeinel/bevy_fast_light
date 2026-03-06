/*
 * File: light.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Light types and related objects.

use bevy::render::render_resource::{BlendComponent, BlendFactor, BlendOperation, BlendState};

pub mod ambient_light;
pub mod point_light;

/// [`BlendState`] for additive blending for 2D lights.
///
/// This implements the following formula:
/// `result = src_color * 1.0 + dst_color * 1.0`
pub(crate) const BLEND_ADD: BlendState = BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    },
};

/// [`BlendState`] for multiplicative blending for 2D lights.
///
/// This implements the following formula:
/// `result = dst_color * src_color + (1 - src_alpha) * dst_color`
pub(crate) const BLEND_MULTIPLY: BlendState = BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::Dst,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent::OVER,
};
