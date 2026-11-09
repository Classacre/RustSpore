//! Compare bounded request store/provider projection to retained source rows.
use spore_core::cell_camera_request_store_v1::project;
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let mut a=std::env::args().skip(1);let b=std::fs::read(a.next().expect("native stdout")).unwrap();let map=std::fs::read_to_string(a.next().expect("host map")).unwrap();assert_eq!(b.len(),144);
 let lines:Vec<_>=map.lines().filter(|l|l.split_whitespace().nth(1)==Some("Request")).collect();assert_eq!(lines.len(),1);let request=u32::from_str_radix(lines[0].split_whitespace().nth(2).unwrap(),16).unwrap();
 for i in 0..4{let first=u32::from(i!=0);let typ=if i==1{0x11111111}else{0x3d97a8e4};let factory=if i>=2{Some(if i==3{request}else{0})}else{None};let x=project([first,typ,0x40616200],0,factory,0x10000,first,0,true).unwrap();let o=i*36;
  assert_eq!((w(&b,o),w(&b,o+4),w(&b,o+8),w(&b,o+12)),(x.provider_return,x.component_request,u32::from(x.factory_called),first));let fp=w(&b,o+16);assert_eq!(fp&0xffff,0x027f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,o+20)&!0x3f,0x1f80);assert_eq!((w(&b,o+24),w(&b,o+28),w(&b,o+32)),(0xc0dec0de,0xa5a5a5a5,0x10000));
 }println!("matched four controlled request-store/provider rows; factory and ownership supplied");}
