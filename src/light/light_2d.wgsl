#import bevy_sprite::{mesh2d_vertex_output::VertexOutput, mesh2d_view_bindings::view}

#import bevy_fast_light::light::types::MeshLight2d

@group(2) @binding(0)
var sprite_depth_texture: texture_2d<f32>;
@group(2) @binding(1)
var sprite_depth_sampler: sampler;
@group(2) @binding(2)
var occluder_texture: texture_2d<f32>;
@group(2) @binding(3)
var occluder_sampler: sampler;
@group(2) @binding(4)
var<uniform> light: MeshLight2d;

const HALF_UV = vec2<f32>(0.5);
const RADIUS_SQ = 0.5 * 0.5;
const INV_RADIUS_SQ = 1. / RADIUS_SQ;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Heavily inspired by: https://github.com/bevyengine/bevy/blob/main/crates/bevy_pbr/src/render/utils.wgsl
    let viewport_uv = (in.position.xy - view.viewport.xy) / view.viewport.zw;

    let sprite_depth_color = textureSample(sprite_depth_texture, sprite_depth_sampler, viewport_uv);
    let occluder_color = textureSample(occluder_texture, occluder_sampler, viewport_uv);
    // NOTE: We need to check `sprite_depth_color.r == 0.` because otherwise if occluders are on z-level 0 and no sprites
    //       are rendered, the whole occluder will not be rendered.
    if occluder_color.r > 0.5 && (occluder_color.g > sprite_depth_color.r || sprite_depth_color.r == 0.) {
        discard;
    }

    let dist = in.uv - HALF_UV;
    let length_sq = dot(dist, dist);
    let falloff = smoothstep(1., 0., length_sq * INV_RADIUS_SQ);
    let attenuation = falloff * falloff;

    return light.color * attenuation;
}
