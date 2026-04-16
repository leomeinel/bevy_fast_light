#define_import_path bevy_fast_light::occluder::types

struct OccluderVertex {
    @builtin(instance_index)
    instance_index: u32,
    @location(0)
    position: vec3<f32>,
};

struct OccluderVertexOutput {
    @builtin(position)
    clip_position: vec4<f32>,
    @location(1)
    normalized_z: f32,
};
