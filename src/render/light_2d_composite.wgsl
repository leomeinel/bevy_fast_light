#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

#import bevy_fast_light::types::{ExtractedAmbientLight2d}

@group(0) @binding(0)
var screen_texture: texture_2d<f32>;
@group(0) @binding(1)
var screen_sampler: sampler;
@group(0) @binding(2)
var light_2d_texture: texture_2d<f32>;
@group(0) @binding(3)
var light_2d_sampler: sampler;
@group(0) @binding(4)
var<uniform> ambient: ExtractedAmbientLight2d;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let src_color = textureSample(screen_texture, screen_sampler, in.uv);
    let light_2d_color = textureSample(light_2d_texture, light_2d_sampler, in.uv);

    return src_color * (vec4<f32>(ambient.color, 1.) + light_2d_color);
}
