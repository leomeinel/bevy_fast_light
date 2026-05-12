#import bevy_sprite::{mesh2d_vertex_output::VertexOutput, mesh2d_view_bindings::view}
#import bevy_render::view::frag_coord_to_uv

@group(2) @binding(0)
var occluder_texture: texture_2d<f32>;
@group(2) @binding(1)
var occluder_sampler: sampler;
@group(2) @binding(2)
var<uniform> light_color: vec4<f32>;

const HALF_UV = vec2<f32>(0.5);
const INV_RADIUS_SQ = 1. / (0.5 * 0.5);

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // NOTE: This is not technically correct, but since the output texture is always at viewport resolution this works.
    let viewport_uv = frag_coord_to_uv(in.position.xy, view.viewport);
    let occluder_color = textureSample(occluder_texture, occluder_sampler, viewport_uv);
    let sprite_depth = occluder_color.r;
    let is_occluder = occluder_color.g > 0.5;
    let occluder_depth = occluder_color.b;
    // NOTE: Checking if sprite_depth is 0 is necessary for occluders on z-level 0 if no sprites are rendered.
    let is_occluded = is_occluder && (sprite_depth == 0. || occluder_depth > sprite_depth);

    let dist = in.uv - HALF_UV;
    let length_sq = dot(dist, dist);
    let falloff = smoothstep(1., 0., length_sq * INV_RADIUS_SQ);
    let attenuation = falloff * falloff;

    return light_color * attenuation * select(1., 0., is_occluded);
}
