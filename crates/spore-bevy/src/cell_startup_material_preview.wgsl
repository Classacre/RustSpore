// Controlled source-material subset. Both basis and lighting are synthetic.
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
struct VertexOut { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, };
@vertex fn vertex(@builtin(instance_index) instance: u32, @location(0) position: vec3<f32>, @location(2) uv: vec2<f32>) -> VertexOut {
 var o: VertexOut; o.position=mesh_position_local_to_clip(get_world_from_local(instance),vec4(position,1.0));o.uv=uv;return o;
}
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var stage0:texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var sampler0:sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var stage1:texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var sampler1:sampler;
@fragment fn fragment(in:VertexOut)->@location(0) vec4<f32> {
 let diffuse=textureSample(stage0,sampler0,in.uv);let packed=textureSample(stage1,sampler1,in.uv);
 let normal=normalize(2.0*packed.rgb-vec3(0.99849999));
 let c0=vec4<f32>(bitcast<f32>(0x42700000u),bitcast<f32>(0x3f733333u),bitcast<f32>(0x3f800000u),bitcast<f32>(0x3eb33333u));
 let light=mix(0.666666687*normal.y,1.0,packed.a*c0.w);
 let reflected=clamp(2.0*normal.z*normal.z-1.0,0.0,1.0);
 let specular=(packed.a*c0.y)*pow(reflected,c0.x);
 return vec4(diffuse.rgb*light+vec3(specular),diffuse.a);
}
