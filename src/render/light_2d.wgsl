#import bevy_render::view::{uv_to_ndc, position_ndc_to_world}
#import bevy_render::view::View

#import bevy_fast_light::types::{ExtractedPointLight2d, ExtractedLight2dMeta, Light2dVertexOutput}

@group(0) @binding(0)
var<uniform> view: View;

@group(1) @binding(0)
var<uniform> light_meta: ExtractedLight2dMeta;
// NOTE: WebGL2 does not support storage buffers and only supports up to 4096 bytes per uniform buffer.
#if AVAILABLE_STORAGE_BUFFER_BINDINGS == 0
    // NOTE: `ExtractedPointLight2d` is 48 bytes and `4096. / 32. = 128.`.
    const MAX_LIGHTS = 128u;
    @group(1) @binding(1)
    var<uniform> point_lights: array<ExtractedPointLight2d, MAX_LIGHTS>;
#else
    @group(1) @binding(1)
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
    var light_2d_color = vec3<f32>(0.);
    for (var i = 0u; i < light_meta.count; i++) {
        let light = point_lights[i];
        let dist = in.world_position - light.world_pos;
        let dist_sq = dot(dist, dist);

        if dist_sq <= light.inner_radius_sq {
            light_2d_color += light.color;
        } else if dist_sq <= light.outer_radius_sq {
            let radius_delta_frac = (dist_sq - light.inner_radius_sq) * light.inv_radius_delta_sq;
            let falloff = saturate(1. - radius_delta_frac * radius_delta_frac);
            let attenuation = falloff * falloff;

            light_2d_color += light.color * attenuation;
        }
    }

    return vec4<f32>(light_2d_color, 1.);
}
