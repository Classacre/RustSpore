"""Explicit Cargo example build through tools/build.bat; never invoked by preparation."""
import argparse, hashlib, json, os, shutil, subprocess, sys
from pathlib import Path
from cell_arbitration_native_build_v3 import sha, verify
ROOT=Path(__file__).resolve().parent.parent

def main():
 p=argparse.ArgumentParser(); p.add_argument('--prep',type=Path,required=True); p.add_argument('--prep-manifest-sha256',required=True); p.add_argument('--source-exe',type=Path,required=True)
 for name in ('cargo','rustc'):
  p.add_argument('--'+name,type=Path,required=True); p.add_argument('--'+name+'-sha256',required=True)
 a=p.parse_args(); prep=a.prep.resolve(); prep.relative_to((ROOT/'target').resolve()); mp=prep/'manifest.json'; mh=sha(mp)
 if mh!=a.prep_manifest_sha256.lower(): raise ValueError('preparation manifest mismatch')
 m=json.loads(mp.read_text()); verify(prep,m,a.source_exe)
 pins={}
 for name in ('cargo','rustc'):
  path=getattr(a,name).resolve(); digest=sha(path)
  if digest!=getattr(a,name+'_sha256').lower(): raise ValueError(name+' pin mismatch')
  pins[name]={'path':str(path),'sha256':digest}
 build_dir=prep/'cargo-target'; receipt=prep/'cargo-build-manifest.json'
 if build_dir.exists() or receipt.exists(): raise ValueError('Cargo build output exists')
 env=os.environ.copy(); env['PATH']=str(Path(pins['cargo']['path']).parent)+os.pathsep+env.get('PATH',''); env['RUSTC']=pins['rustc']['path']
 if Path(shutil.which('cargo',path=env['PATH'])).resolve()!=Path(pins['cargo']['path']): raise ValueError('Cargo PATH resolution mismatch')
 cmd=[str(Path(os.environ['ComSpec']).resolve()),'/d','/c',str(ROOT/'tools/build.bat'),'build','--locked','-p','spore-core','--example','cell_arbitration_oracle','--target','x86_64-pc-windows-msvc','--target-dir',str(build_dir)]
 versions={}
 for name in ('cargo','rustc'):
  r=subprocess.run([pins[name]['path'],'-vV' if name=='rustc' else '--version'],capture_output=True,text=True,timeout=30,env=env); versions[name]={'exit':r.returncode,'stdout':r.stdout,'stderr':r.stderr}
 r=subprocess.run(cmd,cwd=ROOT,env=env,capture_output=True,text=True,timeout=600)
 if r.returncode: raise RuntimeError('Cargo build failed: '+r.stdout+'\n'+r.stderr)
 verify(prep,m,a.source_exe)
 if sha(mp)!=mh: raise ValueError('prep manifest changed')
 for tool in pins.values():
  if sha(tool['path'])!=tool['sha256']: raise ValueError('Cargo tool changed')
 exe=build_dir/'x86_64-pc-windows-msvc/debug/examples/cell_arbitration_oracle.exe'
 data={'schema':'cell-arbitration-cargo-build-v3','prep_manifest_sha256':mh,'source_exe_path':str(a.source_exe.resolve()),'example_sha256':m['example_source_sha256'],'rust_source_sha256':m['rust_source_sha256'],'source_tree_sha256':m['source_tree_sha256'],'build_bat_sha256':m['build_bat_sha256'],'cargo_toml_sha256':m['cargo_toml_sha256'],'crate_toml_sha256':m['crate_toml_sha256'],'cargo_lock_sha256':m['cargo_lock_sha256'],'builder_sha256':m['cargo_builder_sha256'],'adapter_path':str(exe),'adapter_sha256':sha(exe),'tools':pins,'versions':versions,'command':cmd,'exit':r.returncode,'stdout':r.stdout,'stderr':r.stderr,'python_executable':str(Path(sys.executable).resolve()),'python_sha256':sha(sys.executable),'limitations':'Cargo and rustc binaries pinned; loaded libraries, rustc sysroot, MSVC/linker and SDK selected by build.bat, build helpers, configuration and environment are not pinned. Logs may identify selected MSVC/SDK. Timeout kills direct child only, descendants may survive; not sandboxed.'}
 receipt.write_text(json.dumps(data,indent=2)+'\n',encoding='utf-8'); print(json.dumps({'cargo_manifest':str(receipt),'adapter_sha256':data['adapter_sha256']}))
if __name__=='__main__': main()
