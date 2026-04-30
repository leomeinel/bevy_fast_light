#import bevy_render::view::{View, uv_to_ndc, position_ndc_to_world}

#import bevy_fast_light::light::types::{ExtractedPointLight2d, ExtractedLight2dMeta, Light2dVertexOutput}

@group(0) @binding(0)
var<uniform> view: View;

@group(1) @binding(0)
var sprite_depth_texture: texture_2d<f32>;
@group(1) @binding(1)
var sprite_depth_sampler: sampler;
@group(1) @binding(2)
var occluder_texture: texture_2d<f32>;
@group(1) @binding(3)
var occluder_sampler: sampler;
@group(1) @binding(4)
var<uniform> light_meta: ExtractedLight2dMeta;
// NOTE: WebGL2 does not support storage buffers and only supports up to 4096 bytes per uniform buffer.
#if AVAILABLE_STORAGE_BUFFER_BINDINGS == 0
    // NOTE: `ExtractedPointLight2d` is 32 bytes and `4096. / 32. = 128.`.
    const MAX_LIGHTS: u32 = 128;
    @group(1) @binding(5)
    var<uniform> point_lights: array<ExtractedPointLight2d, MAX_LIGHTS>;
#else
    @group(1) @binding(5)
    var<storage, read> point_lights: array<ExtractedPointLight2d>;
#endif

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> Light2dVertexOutput {
    let uv = vec2<f32>(f32(vertex_index >> 1u), f32(vertex_index & 1u)) * 2.0;
    let clip_position = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    let ndc = uv_to_ndc(uv);
    let world_position = position_ndc_to_world(vec3<f32>(ndc, 0.0), view.world_from_clip).xy;

    return Light2dVertexOutput(
        clip_position,
        uv,
        world_position,
    );
}

@fragment
fn fragment(in: Light2dVertexOutput) -> @location(0) vec4<f32> {
    let sprite_depth_color = textureSample(sprite_depth_texture, sprite_depth_sampler, in.uv);
    let occluder_color = textureSample(occluder_texture, occluder_sampler, in.uv);
    if occluder_color.r > 0.5 && occluder_color.g > sprite_depth_color.r {
        discard;
    }

    var light_2d_color = vec3<f32>(0.);
    for (var i = 0u; i < light_meta.count; i++) {
        let light = point_lights[i];
        let dist = in.world_position - light.world_pos;
        let length_sq = dot(dist, dist);

        if length_sq > light.outer_radius_sq {
            continue;
        }

        if length_sq <= light.inner_radius_sq {
            light_2d_color += light.color;
        } else {
            let radius_delta_frac = (length_sq - light.inner_radius_sq) * light.inv_radius_delta_sq;
            let falloff = smoothstep(0., 1., 1. - radius_delta_frac);
            let attenuation = falloff * falloff;

            light_2d_color += light.color * attenuation;
        }
    }

    return vec4<f32>(light_2d_color, 1.);
}
