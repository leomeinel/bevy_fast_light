#import bevy_render::maths::affine3_to_square
#import bevy_sprite::sprite_view_bindings::view

// NOTE: `Rgba8Unorm` is 8 bits per channel. This lets us distinguish between 256 colors.
//       Therefore `MAX_Z` can't be too high. `Rgba8Unorm` is apparently one of the best
//       supported texture formats and I don't necessarily need more precision.
const MAX_Z: f32 = 32.;

// NOTE: The structs have to be defined here, otherwise I get:
//       `Composable module identifiers must not require substitution according to naga writeback rules [...]`
struct VertexInput {
    @builtin(vertex_index)
    index: u32,
    // NOTE: Instance-rate vertex buffer members prefixed with i_
    // NOTE: i_model_transpose_colN are the 3 columns of a 3x4 matrix that is the transpose of the
    // affine 4x3 model matrix.
    @location(0)
    i_model_transpose_col0: vec4<f32>,
    @location(1)
    i_model_transpose_col1: vec4<f32>,
    @location(2)
    i_model_transpose_col2: vec4<f32>,
    @location(3)
    i_uv_offset_scale: vec4<f32>,
}
struct VertexOutput {
    @builtin(position)
    clip_position: vec4<f32>,
    @location(0)
    uv: vec2<f32>,
    @location(1) @interpolate(flat)
    world_z: f32,
};

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    let vertex_position = vec3<f32>(
        f32(in.index & 0x1u),
        f32((in.index & 0x2u) >> 1u),
        0.0
    );
    let world_position = affine3_to_square(mat3x4<f32>(
        in.i_model_transpose_col0,
        in.i_model_transpose_col1,
        in.i_model_transpose_col2,
    )) * vec4<f32>(vertex_position, 1.0);
    let clip_position = view.clip_from_world * world_position;
    let uv = vec2<f32>(vertex_position.xy) * in.i_uv_offset_scale.zw + in.i_uv_offset_scale.xy;

    return VertexOutput (
        clip_position,
        uv,
        saturate(world_position.z / MAX_Z),
    );
}

@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    if textureSample(sprite_texture, sprite_sampler, in.uv).a <= 0.5 {
        discard;
    }

    return vec4<f32>(in.world_z, 0., 0., 1.);
}
