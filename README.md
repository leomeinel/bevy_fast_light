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

## Usage

Take a look at [`/examples`](https://github.com/leomeinel/bevy_fast_light/tree/main/examples) to find out how to use my crate.

### Examples

#### `ambient_light.rs`

Basic scene with a blue `AmbientLight2d`, a red rectangle as background and a `PointLight2d` of the same color.

<img src="https://github.com/leomeinel/bevy_fast_light/blob/main/static/ambient_light.webp?raw=true" width="400" alt="ambient light example">

#### `point_light.rs`

Basic scene with a green rectangle as background and a `PointLight2d` of the same color.

<img src="https://github.com/leomeinel/bevy_fast_light/blob/main/static/point_light.webp?raw=true" width="400" alt="point light example">

## Alternatives

- [bevy_2d_screen_space_lightmaps](https://github.com/goto64/bevy_2d_screen_space_lightmaps)
- [bevy_light_2d](https://crates.io/crates/bevy_light_2d)
- [bevy_lit](https://crates.io/crates/bevy_lit)

## Inspired by

- [Bevy Example - CPU Drawing](https://bevy.org/examples/2d-rendering/cpu-draw/)
- [Bevy Example - 2d Shapes](https://bevy.org/examples/2d-rendering/2d-shapes/)
- [Bevy Example - Extended Material](https://bevy.org/examples/shaders/extended-material/)
- [aarthificial - Deferred Lights - Pixel Renderer Devlog #1](https://youtu.be/R6vQ9VmMz2w)
- [Barney Codes - Introduction to shaders: Learn the basics!](https://youtu.be/3mfvZ-mdtZQ)
- [Inkbox - Creating My Own 3D Graphics Engine](https://youtu.be/OJoZSRnU0is)
- [JohnBrx - Creating 3D Lighting for my 2D Game](https://youtu.be/UavoVWHrebM)
- [Shirley Leisurely Games - Overcoming Godot’s 2D Light Limit With a Custom Light System](https://youtu.be/Ux3IIhbZl34)
- [tEEvy - Understanding shaders is easy, actually](https://youtu.be/xnZfMRTmfJY)
- [github - motion-canvas/examples](https://github.com/motion-canvas/examples/tree/master/examples/deferred-lighting/src)
- [MDN - GLSL shaders](https://developer.mozilla.org/en-US/docs/Games/Techniques/3D_on_the_web/GLSL_Shaders)
- [slembcke.net - 2D Lighting Techniques](https://www.slembcke.net/blog/2DLightingTechniques/)
- [slembcke.net - 2D Lighting with Hard Shadows](https://www.slembcke.net/blog/SuperFastHardShadows/)
