#define_import_path bevy_fast_light::types

struct ExtractedAmbientLight2d {
    color: vec3<f32>,
    _padding: f32,
};

struct ExtractedPointLight2d {
    color: vec3<f32>,
    inner_radius_sq: f32,
    world_pos: vec2<f32>,
    outer_radius_sq: f32,
    inv_radius_delta_sq: f32,
}

struct Light2dMeta {
    count: u32,
    _padding: vec3<f32>,
}
