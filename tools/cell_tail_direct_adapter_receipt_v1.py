"""Record adapter and build inputs after build; no claim of hermetic provenance."""
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent.parent
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
exe=root/'target/debug/examples/cell_tail_direct_oracle_v1.exe'
if sha(exe)!='2eed486019e365698992cd7eee8697e0e5ae57a16113307e934d1d3a77fcfe68': raise ValueError('unexpected adapter')
rust=Path.home() / '.rustup' / 'toolchains' / '1.93.1-x86_64-pc-windows-msvc' / 'bin'
paths=[root/'crates/spore-core/examples/cell_tail_direct_oracle_v1.rs',
 root/'crates/spore-core/src/cell_feeding_tail_e6a2b0_v1.rs',
 root/'crates/spore-core/src/cell_feeding_contact_arbitration_e59b40_v1.rs',
 root/'crates/spore-core/src/lib.rs',root/'crates/spore-core/Cargo.toml',root/'Cargo.toml',root/'Cargo.lock',root/'tools/build.bat',
 rust/'rustc.exe',rust/'cargo.exe',Path(__file__).resolve()]
r={'schema':'cell-tail-direct-adapter-postbuild-observation-v1','adapter_sha256':sha(exe),
 'command':'tools/build.bat build --locked -p spore-core --example cell_tail_direct_oracle_v1',
 'pinned_files':[{'path':str(p.resolve()),'sha256':sha(p)} for p in paths],
 'limitations':'Post-build hashes do not prove build invocation or rule out transient replacement; other Rust source, compiler DLLs, sysroot, SDK, environment and build dependencies are unpinned.'}
out=root/'target/cell-tail-direct-review-v3/adapter-receipt.json'
with out.open('x',encoding='utf-8') as f: json.dump(r,f,indent=2); f.write('\n')
print(sha(out))
