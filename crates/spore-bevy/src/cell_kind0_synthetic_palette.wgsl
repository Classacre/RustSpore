// Controlled synthetic palette, not an observed animation pose.
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
struct VertexOut { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vertex(@builtin(instance_index) instance: u32, @location(0) p: vec3<f32>, @location(2) uv: vec2<f32>, @location(4) indices: vec4<f32>, @location(5) weights: vec4<f32>) -> VertexOut {
    let k = indices / 3.0;
    let tx = weights.x * (k.x * 0.004) + weights.y * (k.y * 0.004) + weights.z * (k.z * 0.004);
    let ty = weights.x * (f32(u32(k.x) % 3u) * 0.004) + weights.y * (f32(u32(k.y) % 3u) * 0.004) + weights.z * (f32(u32(k.z) % 3u) * 0.004);
    let p_skin = p * (weights.x + weights.y + weights.z) + vec3(tx, ty, 0.0);
    var out: VertexOut;
    out.position = mesh_position_local_to_clip(get_world_from_local(instance), vec4(p_skin, 1.0));
    out.uv = uv;
    return out;
}
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var stage_zero: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var stage_zero_sampler: sampler;
@fragment fn fragment(in: VertexOut) -> @location(0) vec4<f32> { return textureSample(stage_zero, stage_zero_sampler, in.uv); }
