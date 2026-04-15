#import bevy_sprite::{mesh2d_functions::{get_world_from_local, mesh2d_position_local_to_clip}}

#import bevy_fast_light::types::{Light2dOccluderVertex, Light2dOccluderVertexOutput}

@vertex
fn vertex(vertex: Light2dOccluderVertex) -> Light2dOccluderVertexOutput {
    let world_from_local = get_world_from_local(vertex.instance_index);
    let clip_position = mesh2d_position_local_to_clip(world_from_local, vec4<f32>(vertex.position, 1.0));
    return Light2dOccluderVertexOutput (
        clip_position,
    );
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return vec4(1.0);
}
