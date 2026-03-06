#import bevy_fast_light::types::{AmbientLight2dMaterial}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: AmbientLight2dMaterial;

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return material.color;
}
