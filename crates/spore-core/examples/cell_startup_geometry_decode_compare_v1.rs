//! Controlled candidate packed-input versus current MeshData decoding.
//! Does not establish source shader normal, transform, pose or lighting semantics.
use spore_core::{gmdl, mesh};
use std::{fs::OpenOptions, io::Write, path::Path};

fn save_new(path: &Path, bytes: &[u8]) {
    let mut f = OpenOptions::new().write(true).create_new(true).open(path).unwrap();
    f.write_all(bytes).unwrap();
}
fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    assert_eq!(args.len(), 3, "model payload and fresh output directory required");
    let raw = std::fs::read(&args[1]).unwrap();
    let output = Path::new(&args[2]);
    let model = gmdl::parse(&raw).unwrap();
    assert_eq!(model.meshes.len(), 1);
    let m = &model.meshes[0];
    let vb = &model.vertex_buffers[usize::try_from(m.vertex_index).unwrap()];
    let tri = &model.triangle_buffers[usize::try_from(m.triangle_index).unwrap()];
    let fmt = &model.vertex_formats[usize::try_from(vb.format_index).unwrap()];
    assert_eq!((vb.vertex_count, vb.stride, tri.indices.len(), tri.primitive_type), (2555, 36, 12732, 4));
    let decl: Vec<_> = fmt.elements.iter().map(|e| (e.stream,e.offset,e.element_type,e.method,e.usage,e.usage_index)).collect();
    assert_eq!(decl, vec![(0,0,2,0,0,0),(0,12,5,0,3,0),(0,16,1,0,5,0),(0,24,5,0,6,0),(0,28,5,0,2,0),(0,32,8,0,1,0)]);
    let bytes = raw.get(vb.data_offset..vb.data_offset.checked_add(vb.stride * vb.vertex_count as usize).unwrap()).unwrap();
    assert_eq!(bytes.len(), vb.buffer_size as usize);
    let index_bytes: Vec<u8> = tri.indices.iter().flat_map(|x| x.to_le_bytes()).collect();
    let decoded = mesh::from_gmdl(&raw, &model).unwrap();
    assert_eq!(decoded.indices, tri.indices);
    assert_eq!(decoded.positions.len(), vb.vertex_count as usize);
    assert!(decoded.normals_from_file);
    let mut uv_sanitised = 0usize;
    for (i, packed) in bytes.chunks_exact(vb.stride).enumerate() {
        let read = |at: usize| f32::from_le_bytes(packed[at..at+4].try_into().unwrap());
        let p = decoded.positions[i];
        for (actual, at) in [(p.x,0),(p.y,4),(p.z,8)] { assert_eq!(actual.to_bits(), read(at).to_bits(), "position {i}"); }
        let uv = [read(16),read(20)];
        let expected = if uv.iter().all(|x| x.is_finite()) { uv } else { uv_sanitised += 1; [0.0,0.0] };
        for (a,b) in decoded.uvs[i].iter().zip(expected) { assert_eq!(a.to_bits(), b.to_bits(), "uv {i}"); }
        let n = decoded.normals[i];
        assert!(n.x.is_finite() && n.y.is_finite() && n.z.is_finite(), "normal {i}");
    }
    save_new(&output.join("vertices.bin"), bytes);
    save_new(&output.join("indices-le.bin"), &index_bytes);
    println!("matched positions={} indices={} uvs={} uv_sanitised={} total_sanitised={} normals=finite_decoded_and_normalized normal_shader_equivalence=unverified basis=untransformed_source_data", decoded.positions.len(), decoded.indices.len(), decoded.uvs.len(), uv_sanitised, decoded.sanitised);
}
