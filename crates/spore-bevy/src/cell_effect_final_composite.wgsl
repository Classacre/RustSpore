@group(#{MATERIAL_BIND_GROUP}) @binding(0) var source_texture:texture_2d<f32>;
@fragment fn fragment(@builtin(position) position:vec4<f32>)->@location(0) vec4<f32>{
 let encoded = textureLoad(source_texture,vec2<i32>(position.xy),0);
 let decoded_rgb = select(encoded.rgb / vec3(12.92), pow((encoded.rgb+vec3(0.055))/vec3(1.055),vec3(2.4)),encoded.rgb>vec3(0.04045));
 return vec4(decoded_rgb,encoded.a);
}
