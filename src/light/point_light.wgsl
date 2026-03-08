#import bevy_sprite::mesh2d_functions::{get_world_from_local, mesh2d_position_local_to_clip}

#import bevy_fast_light::types::{PointLight2dMaterial, VertexOutput, Vertex}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: PointLight2dMaterial;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let world_from_local = get_world_from_local(vertex.instance_index);

    return VertexOutput(
        mesh2d_position_local_to_clip(world_from_local, vec4<f32>(vertex.position, 1.)),
        (world_from_local * vec4<f32>(vertex.position, 1.)).xy,
        (world_from_local * vec4<f32>(0., 0., 0., 1.)).xy,
    );
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let delta = in.world_position - in.world_position_origin;
    let distance_sq = dot(delta, delta);

    if distance_sq <= material.inner_radius_sq {
        return material.color;
    }

    let distance_sq_frac = distance_sq * material.inv_outer_radius_sq;
    let falloff = saturate(1. - distance_sq_frac * distance_sq_frac);
    let attenuation = falloff * falloff;

    return material.color * attenuation;
}
