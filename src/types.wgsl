#define_import_path bevy_fast_light::types

struct AmbientLight2dMaterial {
    color: vec4<f32>,
};

struct PointLight2dMaterial {
    cast_shadows: u32,
    color: vec4<f32>,
    inner_radius_sq: f32,
    outer_radius_sq: f32,
    inv_outer_radius_sq: f32,
};

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec2<f32>,
    @location(1) world_position_origin: vec2<f32>,
};
