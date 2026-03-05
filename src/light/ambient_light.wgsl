#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

#import bevy_fast_light::types::{AmbientLight2dMaterial}

@group(0) @binding(0)
var screen_texture: texture_2d<f32>;
@group(0) @binding(1)
var screen_sampler: sampler;
@group(0) @binding(2)
var<uniform> material: AmbientLight2dMaterial;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, screen_sampler, in.uv);
    return color * material.color;
}
