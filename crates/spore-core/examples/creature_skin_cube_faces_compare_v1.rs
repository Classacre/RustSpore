use spore_core::creature_skin_cube_walk_v1::face_records;use std::io::Write;
fn main(){let mut out=Vec::new();for face in face_records(){out.extend(face);}std::io::stdout().write_all(&out).unwrap();}
