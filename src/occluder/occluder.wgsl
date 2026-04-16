#import bevy_sprite::{mesh2d_functions::{get_world_from_local, mesh2d_position_local_to_clip}}

#import bevy_fast_light::occluder::types::{OccluderVertex, OccluderVertexOutput}

// NOTE: `Rgba8Unorm` is 8 bits per channel. This lets us distinguish between 256 colors.
//       Therefore `MAX_Z` can't be too high. `Rgba8Unorm` is apparently one of the best
//       supported texture formats and I don't necessarily need more precision.
const MAX_Z: f32 = 32.;

@vertex
fn vertex(vertex: OccluderVertex) -> OccluderVertexOutput {
    let world_from_local = get_world_from_local(vertex.instance_index);
    let clip_position = mesh2d_position_local_to_clip(world_from_local, vec4<f32>(vertex.position, 1.0));
    let mesh_z = world_from_local[3].z;

    return OccluderVertexOutput (
        clip_position,
        saturate(mesh_z / MAX_Z),
    );
}

@fragment
fn fragment(in: OccluderVertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, in.normalized_z, 0., 1.);
}
