#define_import_path bevy_fast_light::occluder::types

struct Vertex {
    @builtin(instance_index)
    instance_index: u32,
    @location(0)
    position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position)
    clip_position: vec4<f32>,
    @location(0)
    normalized_z: f32,
};
