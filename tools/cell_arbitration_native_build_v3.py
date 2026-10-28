"""Build only the x86 host and bind a separately built Cargo adapter."""
import argparse, hashlib, json, os, subprocess, sys
from pathlib import Path
from cell_arbitration_native_prepare_v3 import source_tree_sha
ROOT=Path(__file__).resolve().parent.parent
FILES={'generator_sha256':'tools/cell_arbitration_native_prepare_v3.py','builder_sha256':'tools/cell_arbitration_native_build_v3.py','runner_sha256':'tools/cell_arbitration_native_compare_v3.py','cargo_builder_sha256':'tools/cell_arbitration_cargo_build_v3.py','rust_source_sha256':'crates/spore-core/src/cell_feeding_contact_arbitration_e59b40_v1.rs','example_source_sha256':'crates/spore-core/examples/cell_arbitration_oracle.rs','build_bat_sha256':'tools/build.bat','cargo_toml_sha256':'Cargo.toml','crate_toml_sha256':'crates/spore-core/Cargo.toml','cargo_lock_sha256':'Cargo.lock'}
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def verify(prep,m,exe):
 if m.get('schema')!='cell-arbitration-prep-v3': raise ValueError('not v3')
 for path,key in [(exe,'source_sha256'),(prep/'host.c','host_sha256')]+[(ROOT/path,key) for key,path in FILES.items()]:
  if sha(path)!=m[key]: raise ValueError(f'{key} mismatch: {path}')
 if source_tree_sha()!=m['source_tree_sha256']: raise ValueError('spore-core Rust source tree changed')
def verify_cargo(prep,m,mh,receipt_path,receipt_hash,adapter,adapter_hash):
 receipt_path.resolve().relative_to((ROOT/'target').resolve()); adapter.resolve().relative_to((ROOT/'target').resolve())
 if sha(receipt_path)!=receipt_hash: raise ValueError('Cargo manifest mismatch')
 r=json.loads(receipt_path.read_text())
 if r.get('schema')!='cell-arbitration-cargo-build-v3' or r['prep_manifest_sha256']!=mh: raise ValueError('Cargo provenance mismatch')
 for key in ('rust_source_sha256','source_tree_sha256','build_bat_sha256','cargo_toml_sha256','crate_toml_sha256','cargo_lock_sha256'):
  if r[key]!=m[key]: raise ValueError('Cargo source provenance mismatch: '+key)
 if r['example_sha256']!=m['example_source_sha256'] or r['builder_sha256']!=m['cargo_builder_sha256']: raise ValueError('Cargo source mismatch')
 if Path(r['adapter_path']).resolve()!=adapter.resolve() or r['adapter_sha256']!=adapter_hash or sha(adapter)!=adapter_hash: raise ValueError('Cargo adapter mismatch')
 for info in r['tools'].values():
  if sha(info['path'])!=info['sha256']: raise ValueError('Cargo tool changed')
 if sha(r['python_executable'])!=r['python_sha256']: raise ValueError('Cargo build Python changed')
 return r
def main():
 p=argparse.ArgumentParser(); p.add_argument('--prep',type=Path,required=True); p.add_argument('--prep-manifest-sha256',required=True); p.add_argument('--source-exe',type=Path,required=True)
 for name in ('cl','link','adapter','cargo-manifest'):
  dest=name.replace('-','_'); p.add_argument('--'+name,dest=dest,type=Path,required=True); p.add_argument('--'+name+'-sha256',dest=dest+'_sha256',required=True)
 a=p.parse_args(); prep=a.prep.resolve(); prep.relative_to((ROOT/'target').resolve()); mp=prep/'manifest.json'; mh=sha(mp)
 if mh!=a.prep_manifest_sha256.lower(): raise ValueError('prep manifest mismatch')
 m=json.loads(mp.read_text()); verify(prep,m,a.source_exe)
 receipt_hash=a.cargo_manifest_sha256.lower(); adapter_hash=a.adapter_sha256.lower()
 verify_cargo(prep,m,mh,a.cargo_manifest,receipt_hash,a.adapter,adapter_hash)
 pins={}
 for name in ('cl','link'):
  path=getattr(a,name).resolve(); digest=sha(path)
  if digest!=getattr(a,name+'_sha256').lower(): raise ValueError(name+' mismatch')
  pins[name]={'path':str(path),'sha256':digest}
 if Path(pins['cl']['path']).parent!=Path(pins['link']['path']).parent: raise ValueError('cl and link must share directory')
 for name in ('host.exe','host.obj','build-manifest.json'):
  if (prep/name).exists(): raise ValueError('existing output: '+name)
 env=os.environ.copy(); env['PATH']=str(Path(pins['link']['path']).parent)+os.pathsep+env.get('PATH','')
 envhash={k:hashlib.sha256(env.get(k,'').encode()).hexdigest() for k in ('PATH','INCLUDE','LIB','LIBPATH')}
 versions={}
 for name in ('cl','link'):
  r=subprocess.run([pins[name]['path']],cwd=prep,env=env,capture_output=True,text=True,timeout=30); versions[name]={'exit':r.returncode,'stdout':r.stdout,'stderr':r.stderr}
 cmd=[pins['cl']['path'],'/nologo','/W4','/O2','/arch:SSE2','/Fohost.obj','/Fehost.exe','host.c','/link','/DYNAMICBASE:NO','/BASE:0x00400000']
 r=subprocess.run(cmd,cwd=prep,env=env,capture_output=True,text=True,timeout=180)
 if r.returncode: raise RuntimeError('native host build failed: '+r.stdout+'\n'+r.stderr)
 verify(prep,m,a.source_exe); verify_cargo(prep,m,mh,a.cargo_manifest,receipt_hash,a.adapter,adapter_hash)
 if sha(mp)!=mh: raise ValueError('manifest changed')
 for tool in pins.values():
  if sha(tool['path'])!=tool['sha256']: raise ValueError('tool changed')
 result={'schema':'cell-arbitration-build-v3','prep_manifest_sha256':mh,'source_exe_path':str(a.source_exe.resolve()),'tools':pins,'tool_versions':versions,'command':cmd,'stdout':r.stdout,'stderr':r.stderr,'environment_hashes':envhash,'python_executable':str(Path(sys.executable).resolve()),'python_sha256':sha(sys.executable),'host_sha256':sha(prep/'host.exe'),'adapter_path':str(a.adapter.resolve()),'adapter_sha256':adapter_hash,'cargo_manifest_path':str(a.cargo_manifest.resolve()),'cargo_manifest_sha256':receipt_hash,'limitations':'Native compiler DLLs, SDK and runtime dependencies are unpinned; Cargo provenance has its own limitations. Timeout kills direct child only; no descendant containment.'}
 (prep/'build-manifest.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8'); print(json.dumps({'build_manifest':str(prep/'build-manifest.json'),'host_sha256':result['host_sha256'],'adapter_sha256':adapter_hash}))
if __name__=='__main__': main()
