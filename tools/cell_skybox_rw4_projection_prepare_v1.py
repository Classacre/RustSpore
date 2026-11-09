"""Prepare a review-only fixed RW4 instruction fixture; never execute it."""
import argparse,hashlib,json
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from gmdl_skin_probe import indexes
from dbpf_extract import payload
from rw4_validate import validate
def sha(b):return hashlib.sha256(b).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('package',type=Path);p.add_argument('out',type=Path);a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output parent outside target')
 image=SourceImage(a.exe); before=sha(a.package.read_bytes());rows=[r for r in indexes(a.package) if r['type']==0x2f4e681b and r['instance']==0xbd08bd8a]
 if len(rows)!=1:raise ValueError('candidate key not unique')
 data=payload(a.package,rows[0]);after=sha(a.package.read_bytes())
 if before!=after or sha(data)!='5705df1ab03e94c58c8a890383026e68c68dfe64ba488d4d767f2706d8ad8aec' or validate(data)!='ok':raise ValueError('candidate mismatch')
 relocation=image.read(0x11e3afe,0x1b);lookup=image.read(0x11eae90,0x25)
 if relocation.hex()!='81fa300001008b08894804720881fa3f00010076082bcb03ce8908' or lookup.hex()!='8b51048d41088d14928d4c91083bc1740f8b5424043910740983c0143bc175f533c0c20400':raise ValueError('source slice mismatch')
 a.out.mkdir();(a.out/'candidate.rw4').write_bytes(data)
 def arr(n,b):return 'static const unsigned char '+n+'[]={'+','.join(f'0x{x:02x}' for x in b)+'};\n'
 header='#pragma once\n'+arr('relocation',relocation)+arr('registry_lookup',lookup);(a.out/'source_bytes.h').write_text(header,encoding='ascii')
 host=Path(__file__).with_name('cell_skybox_rw4_projection_host_review_v1.c')
 r={'scope':'review only; no original instruction execution', 'source_sha256':SOURCE_SHA256,'package_sha256_before_after':before,'payload_sha256':sha(data),
    'relocation_range':'11E3AFE..11E3B18','relocation_sha256':sha(relocation),'lookup_range':'11EAE90..11EAEB4','lookup_sha256':sha(lookup),
    'header_sha256':sha(header.encode()),'host_source_sha256':sha(host.read_bytes()),'candidate_bytes':len(data),
    'admissions':'fixed non-base-resource type 20003, old base zero, synthetic registry and stable trusted paths; no provider/callback/device'}
 (a.out/'manifest.json').write_text(json.dumps(r,indent=2)+'\n');print('manifest sha256',sha((a.out/'manifest.json').read_bytes()),'relocation',sha(relocation),'lookup',sha(lookup))
if __name__=='__main__':main()
