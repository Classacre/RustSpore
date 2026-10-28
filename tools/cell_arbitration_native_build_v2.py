"""Build the reviewed oracle with explicit compiler binary pins. Opt-in execution."""
import argparse, hashlib, json, os, subprocess, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
RUST=ROOT/'crates/spore-core/src/cell_feeding_contact_arbitration_e59b40_v1.rs'
SCRIPTS={'generator_sha256':'tools/cell_arbitration_native_prepare_v2.py','builder_sha256':'tools/cell_arbitration_native_build_v2.py','runner_sha256':'tools/cell_arbitration_native_compare_v2.py'}
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def verify(prep, manifest, exe):
 checks=[(exe,'source_sha256'),(prep/'host.c','host_sha256'),(prep/'adapter.rs','adapter_sha256'),(RUST,'rust_source_sha256')]+[(ROOT/path,key) for key,path in SCRIPTS.items()]
 for path,key in checks:
  if sha(path)!=manifest[key]: raise ValueError(f'{key} mismatch: {path}')
def invoke(argv,cwd,env,timeout):
 return subprocess.run(argv,cwd=cwd,env=env,capture_output=True,text=True,timeout=timeout)
def main():
 p=argparse.ArgumentParser(); p.add_argument('--prep',type=Path,required=True); p.add_argument('--prep-manifest-sha256',required=True); p.add_argument('--source-exe',type=Path,required=True)
 for n in ('cl','link','rustc','rust_link'):
  p.add_argument('--'+n.replace('_','-'),dest=n,type=Path,required=True); p.add_argument('--'+n.replace('_','-')+'-sha256',dest=n+'_sha256',required=True)
 p.add_argument('--rust-lib',required=True,help='x64 LIB search path from the x64 developer environment'); a=p.parse_args(); prep=a.prep.resolve(); prep.relative_to((ROOT/'target').resolve())
 manifest_path=prep/'manifest.json'; m=json.loads(manifest_path.read_text()); manifest_hash=sha(manifest_path)
 if manifest_hash!=a.prep_manifest_sha256.lower(): raise ValueError('preparation manifest pin mismatch')
 if m.get('schema')!='cell-arbitration-prep-v2': raise ValueError('not v2 preparation')
 verify(prep,m,a.source_exe); pins={}
 for name in ('cl','link','rustc','rust_link'):
  path=getattr(a,name).resolve(); expected=getattr(a,name+'_sha256').lower(); actual=sha(path)
  if len(expected)!=64 or actual!=expected: raise ValueError(name+' pin mismatch')
  pins[name]={'path':str(path),'sha256':actual}
 if Path(pins['cl']['path']).parent!=Path(pins['link']['path']).parent: raise ValueError('cl and link must be in same directory')
 for name in ('host.exe','adapter.exe','host.obj','adapter.obj','build-manifest.json'):
  if (prep/name).exists(): raise ValueError('existing output: '+name)
 env=os.environ.copy(); env['PATH']=str(Path(pins['link']['path']).parent)+os.pathsep+env.get('PATH','')
 # Retain hashes only; environment may contain credentials. Runtime, headers, libraries,
 # linker dependencies and Rust sysroot remain outside the binary hash pins.
 rust_env=env.copy(); rust_env['LIB']=a.rust_lib
 envhash={k:hashlib.sha256(env.get(k,'').encode()).hexdigest() for k in ('PATH','INCLUDE','LIB','LIBPATH')}
 envhash['rust_LIB']=hashlib.sha256(rust_env['LIB'].encode()).hexdigest()
 cmds=[[pins['cl']['path'],'/nologo','/W4','/O2','/arch:SSE2','/Fohost.obj','/Fehost.exe','host.c','/link','/DYNAMICBASE:NO','/BASE:0x00400000'],
       [pins['rustc']['path'],'--target','x86_64-pc-windows-msvc','-O','-C','linker='+pins['rust_link']['path'],'-o','adapter.exe','adapter.rs']]
 versions={}
 for name,cmd in [('cl',[pins['cl']['path']]),('link',[pins['link']['path']]),('rustc',[pins['rustc']['path'],'-vV']),('rust_link',[pins['rust_link']['path']])]:
  r=invoke(cmd,prep,env,30); versions[name]={'exit':r.returncode,'stdout':r.stdout,'stderr':r.stderr}
 logs=[]
 for index,cmd in enumerate(cmds):
  r=invoke(cmd,prep,env if index==0 else rust_env,180); logs.append({'argv':cmd,'exit':r.returncode,'stdout':r.stdout,'stderr':r.stderr})
  if r.returncode: raise RuntimeError('build failed: '+json.dumps(logs))
 verify(prep,m,a.source_exe)
 if sha(manifest_path)!=manifest_hash: raise ValueError('preparation manifest changed during build')
 for name,info in pins.items():
  if sha(info['path'])!=info['sha256']: raise ValueError(name+' changed during build')
 result={'schema':'cell-arbitration-build-v2','prep_manifest_sha256':manifest_hash,'source_exe_path':str(a.source_exe.resolve()),'tools':pins,'tool_versions':versions,'commands':logs,'environment_hashes':envhash,'python_executable':str(Path(sys.executable).resolve()),'python_sha256':sha(sys.executable),'host_sha256':sha(prep/'host.exe'),'adapter_sha256':sha(prep/'adapter.exe'),'limitations':'Compiler DLLs, SDK headers and libraries, Rust sysroot and runtime dependencies are not hash-pinned; environment hashes do not attest their contents. Each tool invocation is trusted and may launch descendants. subprocess timeout kills direct child only.'}
 (prep/'build-manifest.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8'); print(json.dumps({'build_manifest':str(prep/'build-manifest.json'),'host_sha256':result['host_sha256'],'adapter_sha256':result['adapter_sha256']}))
if __name__=='__main__': main()
