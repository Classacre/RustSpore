//! Decode the controlled candidate BC3 mip and emit 16 supplied fixture texels.
use std::io::Write;
fn main(){
 let args:Vec<_>=std::env::args_os().collect();assert_eq!(args.len(),2,"pinned 512x512 BC3 mip required");
 let data=std::fs::read(&args[1]).unwrap();assert_eq!(data.len(),262144);
 let rgba=spore_core::raster::decode_pixels(spore_core::raster::FORMAT_DXT5,512,512,&data).unwrap();assert_eq!(rgba.len(),512*512*4);
 let coords=[(0,0),(511,0),(0,511),(511,511),(1,1),(510,510),(255,255),(256,256),(127,127),(384,384),(64,320),(320,64),(123,456),(456,123),(300,200),(200,300)];
 let mut out=Vec::with_capacity(64);for (x,y) in coords {let at=(y*512+x)*4;out.extend_from_slice(&rgba[at..at+4]);}
 std::io::stdout().write_all(&out).unwrap();
}
