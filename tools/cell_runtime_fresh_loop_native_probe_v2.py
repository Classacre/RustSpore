"""Version2 formatting-only wrapper: split PUBLIC declarations for MASM limits."""
import argparse,hashlib,json
from pathlib import Path
import cell_runtime_fresh_loop_native_probe as base
BASE_TOOL_SHA='8ecef145eff94aeba577fb8c85dbbc9d1e49a23ce12539692fdc41d260cc30ef'
assert hashlib.sha256(Path(base.__file__).read_bytes()).hexdigest()==BASE_TOOL_SHA
for _name in dir(base):
 if not _name.startswith('_') and _name not in globals():globals()[_name]=getattr(base,_name)

def public_symbols(text):
 return [symbol.strip()for line in text.splitlines()if line.startswith('PUBLIC ')for symbol in line[7:].split(',')]

def host_source(images):
 original=base.host_source(images);lines=[]
 for line in original.splitlines():
  if line.startswith('PUBLIC '):lines.extend('PUBLIC '+symbol.strip()for symbol in line[7:].split(','))
  else:lines.append(line)
 result='\n'.join(lines)+'\n'
 assert public_symbols(original)==public_symbols(result)
 assert [x for x in original.splitlines()if not x.startswith('PUBLIC ')]==[x for x in result.splitlines()if not x.startswith('PUBLIC ')]
 assert max(map(len,result.splitlines()))<512
 return result

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--source-crt',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll'));p.add_argument('--audit-exe',type=Path);p.add_argument('--audit-map',type=Path);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());images=arithmetic.Images(a.source_exe,a.source_crt);asm=host_source(images)
 if a.audit_exe:
  assert a.audit_map and(a.out/'host-x86.asm').read_text()==asm;r=linked_audit(a.audit_exe,a.audit_map,images)
  with(a.out/'owned-linked-audit.json').open('x')as f:json.dump(r,f,indent=2)
  print('Ownedlinked static audit PASS; no execution');return
 prior=ROOT/'target/source-cell/native-runtime-fresh-loop-hybrid-host-v3';m=json.loads((prior/'manifest.json').read_text());assert sha((prior/'host-x86.asm').read_bytes())==m['host_source_sha256'];assert(prior/'host-x86.asm').read_text()==base.host_source(images)
 a.out.mkdir(parents=True,exist_ok=False);(a.out/'host-x86.asm').write_text(asm)
 m.update(schema='fresh-full-activation-loop-hybrid-public-format-v2',host_source_sha256=sha((a.out/'host-x86.asm').read_bytes()),preparation_tool_sha256=sha(Path(__file__).read_bytes()),base_preparation_tool_sha256=BASE_TOOL_SHA,format_only='Each PUBLIC symbol now has its own short declaration; all non-PUBLIC lines byte-identical in normalized text, ordered PUBLIC symbols identical.',prior_manifest_sha256=sha((prior/'manifest.json').read_bytes()),prior_asm_sha256=sha((prior/'host-x86.asm').read_bytes()),maximum_line_length=max(map(len,asm.splitlines())),compiled=False,native_executed=False)
 (a.out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(m['host_source_sha256'])
if __name__=='__main__':main()
