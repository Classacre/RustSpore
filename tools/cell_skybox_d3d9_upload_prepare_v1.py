"""Extract two bounded original-instruction fragments for review; never execute them."""
import hashlib, json
from pathlib import Path
from cell_schema_probe import SourceImage, SOURCE_SHA256

def sha(b): return hashlib.sha256(b).hexdigest()
def main():
    import argparse
    p=argparse.ArgumentParser(); p.add_argument('exe',type=Path); p.add_argument('out',type=Path); a=p.parse_args()
    root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
    if not a.out.parent.resolve(strict=True).is_relative_to(root): raise ValueError('output outside target')
    image=SourceImage(a.exe)
    expected_sha256=['9817b7f10322262926a1bc9048caeb1bf746f471ac627452d3d1af3d246b5ae3', '38fff9dcd84f790b69a280ef81961b094c3a50f5b210e4351a2b6ff778deb5a8']
    blocks=[image.read(0x11efde0,135),image.read(0x11f00bd,16)]
    if [sha(x) for x in blocks]!=expected_sha256: raise ValueError('source instruction mismatch')
    a.out.mkdir()
    header='#pragma once\n'+''.join('static const unsigned char %s[]={%s};\n'%(name,','.join('0x%02x'%b for b in data)) for name,data in zip(('size_bytes','copy_bytes'),blocks))
    (a.out/'source_bytes.h').write_bytes(header.encode('ascii'))
    host=Path(__file__).with_name('cell_skybox_d3d9_upload_host_review_v1.c')
    manifest={'source_sha256':SOURCE_SHA256,'size_range':'011EFDE0..011EFE66','copy_range':'011F00BD..011F00CC','slice_sha256':[sha(x) for x in blocks],'header_sha256':sha(header.encode()),'host_source_sha256':sha(host.read_bytes()),'scope':'review only; no source execution; fixed DXT5 input; excluded size branch traps'}
    (a.out/'manifest.json').write_bytes((json.dumps(manifest,indent=2)+'\n').encode())
    print(json.dumps(manifest,indent=2))
if __name__=='__main__': main()
