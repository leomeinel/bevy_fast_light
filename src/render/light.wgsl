#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::{uv_to_ndc, position_ndc_to_world}
#import bevy_render::view::View

#import bevy_fast_light::types::{ExtractedAmbientLight2d, ExtractedPointLight2d}

@group(0) @binding(0)
var screen_texture: texture_2d<f32>;
@group(0) @binding(1)
var screen_sampler: sampler;
@group(0) @binding(2)
var<uniform> view: View;
@group(0) @binding(3)
var<uniform> ambient: ExtractedAmbientLight2d;

// NOTE: WebGL2 does not support storage buffers and only supports up to 4096 bytes per uniform buffer.
#if AVAILABLE_STORAGE_BUFFER_BINDINGS == 0
    // NOTE: `ExtractedPointLight2d` is 48 bytes and `4096. / 48. > 85.`.
    const MAX_LIGHTS = 85u;
    @group(0) @binding(4)
    var<uniform> lights: array<ExtractedPointLight2d, MAX_LIGHTS>;
#else
    @group(0) @binding(4)
    var<storage, read> lights: array<ExtractedPointLight2d>;
#endif

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let ndc = uv_to_ndc(in.uv);
    let world_pos = position_ndc_to_world(vec3<f32>(ndc, 0.0), view.world_from_clip).xy;

    let src_color = textureSample(screen_texture, screen_sampler, in.uv);
    var light_factor = ambient.color;

    for (var i = 0u; i < ambient.light_count; i++) {
        let light = lights[i];
        let dist = world_pos - light.world_pos;
        let dist_sq = dot(dist, dist);

        if dist_sq <= light.inner_radius_sq {
            light_factor += light.color;
        } else if dist_sq <= light.outer_radius_sq {
            let radius_delta_frac = (dist_sq - light.inner_radius_sq) * light.inv_radius_delta_sq;
            let falloff = saturate(1. - radius_delta_frac * radius_delta_frac);
            let attenuation = falloff * falloff;

            light_factor += light.color * attenuation;
        }
    }

    return src_color * light_factor;
}
