// Controlled identity-pose preview. Bevy supplies the camera and transform.
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex(@builtin(instance_index) instance: u32, @location(0) position: vec3<f32>, @location(2) uv: vec2<f32>) -> VertexOut {
    var out: VertexOut;
    out.position = mesh_position_local_to_clip(get_world_from_local(instance), vec4(position, 1.0));
    out.uv = uv;
    return out;
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var stage_zero: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var stage_zero_sampler: sampler;

@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    // The controlled source fixture supplies white material and tint.
    let sample = textureSample(stage_zero, stage_zero_sampler, in.uv);
    let decoded_rgb = select(sample.rgb / vec3(12.92), pow((sample.rgb + vec3(0.055)) / vec3(1.055), vec3(2.4)), sample.rgb > vec3(0.04045));
    return vec4(decoded_rgb, sample.a);
}
