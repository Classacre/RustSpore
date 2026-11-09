//! Explicit source-only PBR plane inspection. No runtime effect or camera claim.
#[path="../src/offscreen.rs"] mod offscreen;
#[path="../src/cell_effectmap_plane_v1.rs"] mod cell_effectmap_plane_v1;
use bevy::{asset::RenderAssetUsages,camera::ScalingMode,image::{ImageAddressMode,ImageFilterMode,ImageSampler,ImageSamplerDescriptor},mesh::Indices,prelude::*,render::render_resource::{Extent3d,PrimitiveTopology,TextureDimension,TextureFormat},render::{renderer::RenderAdapterInfo,view::screenshot::save_to_disk}};
use spore_core::mesh;
use std::path::{Path,PathBuf};

#[derive(Resource)] struct Fixture { geometry:mesh::MeshData, pixels:Vec<u8>, shot:PathBuf, scale:f32, camera_z:f32, ambient:f32, textured:bool }
#[derive(Resource,Default)] struct Capture { frame:u32, requested:bool }
fn load(install:&Path,shot:PathBuf,scale:f32,camera_z:f32,ambient:f32,textured:bool)->Result<Fixture,String>{
 if !scale.is_finite()||!(0.01..=100.).contains(&scale)||!camera_z.is_finite()||!(0.1..=1000.).contains(&camera_z)||!ambient.is_finite()||!(0.0..=10000.).contains(&ambient){return Err("scale, camera z or ambient outside admitted range".into())}
 if shot.exists(){return Err("output already exists".into())}
 let art=cell_effectmap_plane_v1::load(install)?;
 println!("CELL_EFFECTMAP_PLANE inspection=true source_id=44C489B7 shader_saved=80000033 original_shader=false pbr=true runtime_selection=false runtime_camera=false translation=zero rotation=identity scale={scale} camera_z={camera_z} ambient={ambient} atlas_srgb=assumed sampler=wrap_linear_nearest_mip levels=9 textured={textured}");
 Ok(Fixture{geometry:art.geometry,pixels:art.pixels,shot,scale,camera_z,ambient,textured})
}

fn setup(mut commands:Commands,f:Res<Fixture>,adapter:Res<RenderAdapterInfo>,mut meshes:ResMut<Assets<Mesh>>,mut materials:ResMut<Assets<StandardMaterial>>,mut images:ResMut<Assets<Image>>,mut ready:ResMut<offscreen::OffscreenReadiness>){
 println!("CELL_EFFECTMAP_PLANE_GPU name={:?} backend={:?} type={:?}",adapter.name,adapter.backend,adapter.device_type);
 let mut m=Mesh::new(PrimitiveTopology::TriangleList,RenderAssetUsages::default());m.insert_attribute(Mesh::ATTRIBUTE_POSITION,f.geometry.positions.iter().map(|p|[p.x,p.y,p.z]).collect::<Vec<_>>());m.insert_attribute(Mesh::ATTRIBUTE_NORMAL,f.geometry.normals.iter().map(|p|[p.x,p.y,p.z]).collect::<Vec<_>>());m.insert_attribute(Mesh::ATTRIBUTE_UV_0,f.geometry.uvs.clone());m.insert_indices(Indices::U32(f.geometry.indices.clone()));
 let mut tex=Image::new(Extent3d{width:256,height:256,depth_or_array_layers:1},TextureDimension::D2,f.pixels[..256*256*4].to_vec(),TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::default());tex.texture_descriptor.mip_level_count=9;tex.data=Some(f.pixels.clone());tex.sampler=ImageSampler::Descriptor(ImageSamplerDescriptor{address_mode_u:ImageAddressMode::Repeat,address_mode_v:ImageAddressMode::Repeat,mag_filter:ImageFilterMode::Linear,min_filter:ImageFilterMode::Linear,mipmap_filter:ImageFilterMode::Nearest,lod_max_clamp:8.,..default()});let image=images.add(tex);if f.textured{ready.require_image(&image);}
 let material=materials.add(StandardMaterial{base_color:Color::WHITE,base_color_texture:f.textured.then_some(image),cull_mode:None,double_sided:true,..default()});commands.spawn((Mesh3d(meshes.add(m)),MeshMaterial3d(material),Transform::from_scale(Vec3::splat(f.scale))));
 commands.spawn((Camera3d::default(),Projection::Orthographic(OrthographicProjection{scaling_mode:ScalingMode::FixedVertical{viewport_height:2.0*f.scale},..OrthographicProjection::default_3d()}),Transform::from_xyz(0.,0.,f.camera_z),AmbientLight{color:Color::WHITE,brightness:f.ambient,..default()}));
}
fn capture(mut commands:Commands,f:Res<Fixture>,target:Res<offscreen::OffscreenTarget>,ready:Res<offscreen::OffscreenReadiness>,mut c:ResMut<Capture>,time:Res<Time>){
 if let Some(e)=ready.failure(){eprintln!("plane pipeline error: {e}");std::process::exit(1)}
 if time.elapsed_secs()>120.{eprintln!("plane capture timeout");std::process::exit(1)}
 if !c.requested&&ready.ready(){commands.spawn(target.screenshot()).observe(save_to_disk(f.shot.clone()));c.requested=true;}
 if c.requested{c.frame+=1;if c.frame>10&&f.shot.exists(){std::process::exit(0)}}
}
fn main()->Result<(),Box<dyn std::error::Error>>{let a:Vec<_>=std::env::args().skip(1).collect();if a.len()!=6{return Err("usage: cell_effectmap_plane_inspection_v1 <install> <new.png> <plane-scale> <camera-z> <ambient> <textured|white-control>".into())}let textured=match a[5].as_str(){"textured"=>true,"white-control"=>false,_=>return Err("mode must be textured or white-control".into())};let f=load(Path::new(&a[0]),PathBuf::from(&a[1]),a[2].parse()?,a[3].parse()?,a[4].parse()?,textured).map_err(|e|format!("plane admission: {e}"))?;let mut app=App::new();offscreen::configure(&mut app,768,768);app.insert_resource(f).init_resource::<Capture>().add_systems(Startup,setup).add_systems(Update,capture);app.run();Ok(())}
