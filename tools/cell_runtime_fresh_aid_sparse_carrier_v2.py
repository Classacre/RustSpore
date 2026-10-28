"""PUBLIC-only map visibility correction over frozen sparse carrier v1; never execute."""
import argparse,json
from pathlib import Path
import cell_runtime_fresh_aid_sparse_carrier_v1 as v1
from cell_schema_probe import SourceImage
ROOT=v1.ROOT
V1_SHA='b44122a51221d4b69945afb4a2312519b6b06133622b28a9f2574a452e092996'
def host_source(image):
 assert v1.sha(v1.__file__)==V1_SHA
 text=v1.host_source(image);return text.replace('.code\n','PUBLIC InputShadow,ClipShadow,EventsShadow,FrameShadow,SerialShadow,NativeStack\n.code\n',1)
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--owned-exe',type=Path);p.add_argument('--owned-map',type=Path);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());image=SourceImage(a.source_exe);proof=v1.source.audit_source(image)
 if a.owned_exe:
  assert a.owned_map and (a.out/'host-x86.asm').read_text()==host_source(image)
  raw,report=v1.build(a.owned_exe,a.owned_map,image,proof)
  with (a.out/'carrier-x86.exe').open('xb') as f:f.write(raw)
  report.update(native_executed=False,independent_linked_audit_required=True,carrier_sha256=v1.sha(a.out/'carrier-x86.exe'));v1.save(a.out/'carrier-build-review.json',report);return
 a.out.mkdir(parents=True,exist_ok=False);(a.out/'host-x86.asm').write_text(host_source(image));v1.save(a.out/'manifest.json',dict(schema='fresh-aid-sparse-carrier-preparation-v2',tool_sha256=v1.sha(__file__),v1_tool_sha256=V1_SHA,asm_sha256=v1.sha(a.out/'host-x86.asm'),source_audit=proof,source_bytes=1049,primary_base='007C0000',owned_compile_base='27000000',native_executed=False,compiled=False,change='PUBLIC-only map visibility for four shadows,SerialShadow,NativeStack; all instructions/source/RunOnce/inputs unchanged',actual_linked_audit_required=True))
if __name__=='__main__':main()
