# Bevy Fast Light

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/leomeinel/bevy_fast_light)
[![Crates.io](https://img.shields.io/crates/v/bevy_fast_light.svg)](https://crates.io/crates/bevy_fast_light)
[![Downloads](https://img.shields.io/crates/d/bevy_fast_light.svg)](https://crates.io/crates/bevy_fast_light)
[![Docs](https://docs.rs/bevy_fast_light/badge.svg)](https://docs.rs/bevy_fast_light/latest/bevy_fast_light/)

Simple 2D lighting for Bevy focused on performance over features.

:warning: | This is still in development and not at all feature complete.

## Features

- Simple and fast `PointLight2d` light source with `falloff` configurable via inner and outer radius.
- Simple and fast `AmbientLight2d` light source.

## Limitations

- There is currently no light occluders or shadow casting at all.

## Usage

Take a look at [`/examples`](https://github.com/leomeinel/bevy_fast_light/tree/main/examples) to find out how to use this crate.

### Examples

#### `ambient_light.rs`

Scene with a light sky colored `AmbientLight2d` with a lower `intensity`, a green `Rectangle` as background and an amber `PointLight2d`.

<img src="https://github.com/leomeinel/bevy_fast_light/blob/main/static/ambient_light.webp?raw=true" width="400" alt="ambient light example">

#### `point_light.rs`

Scene with a green `Rectangle` as background and an amber `PointLight2d`.

<img src="https://github.com/leomeinel/bevy_fast_light/blob/main/static/point_light.webp?raw=true" width="400" alt="point light example">

#### `texture_scale.rs`

Scene with a green `Rectangle` as background and an amber `PointLight2d` using a lower `texture_scale`.

<img src="https://github.com/leomeinel/bevy_fast_light/blob/main/static/texture_scale.webp?raw=true" width="400" alt="texture scale example">

## Alternatives

- [bevy_2d_screen_space_lightmaps](https://github.com/goto64/bevy_2d_screen_space_lightmaps)
- [bevy_light_2d](https://crates.io/crates/bevy_light_2d)
- [bevy_lit](https://crates.io/crates/bevy_lit)

## Resources

### Code

- [Bevy Example - Custom Post Processing](https://bevy.org/examples/shaders/custom-post-processing/)

### Articles

- [hackmd.io - Bevy's Rendering Crates](https://hackmd.io/@bevy/rendering_summary)
- [hackmd.io - The Abyss](https://hackmd.io/@bevy/the_abyss)
- [slembcke.net - 2D Lighting Techniques](https://www.slembcke.net/blog/2DLightingTechniques/)
- [slembcke.net - 2D Lighting with Hard Shadows](https://www.slembcke.net/blog/SuperFastHardShadows/)
- [WebGPU Shading Language](https://www.w3.org/TR/WGSL/)
