"""Prepare uncompiled root host bitmap-guard revision; never execute native code."""
import hashlib,json
from pathlib import Path
R=Path(__file__).resolve().parent.parent
BASE=R/'target/animation/nonroot-root-host-packet-v2-final';OUT=R/'target/animation/nonroot-root-host-packet-v3'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 assert sha(BASE/'host-x86.asm')=='5cba9971b2290a7a17002740fe952a9f500f541603e111ba6a7ee04aa8351a23'
 assert sha(BASE/'input.bin')=='bd1d0f0180699b4443e709515d76d5e6d3181e72df2b2bf96b601a9b52187ad9'
 source=(BASE/'host-x86.asm').read_text().splitlines(); needle='mov eax,dword ptr [OwnerBuffer+ecx]'
 indexes=[i for i,line in enumerate(source) if line==needle];assert len(indexes)==2
 i=indexes[0];assert source[i+1]=='xor eax,dword ptr [OwnerShadow+ecx]'
 source[i+1:i+1]=['cmp eax,1','jne HostFailure']
 OUT.mkdir(parents=True,exist_ok=False);(OUT/'host-x86.asm').write_text('\n'.join(source)+'\n');(OUT/'input.bin').write_bytes((BASE/'input.bin').read_bytes())
 report={'schema':'source-root-host-v3-bitmap-guard-proposal','compiled':False,'native_executed':False,'admitted_for_execution':False,'asm_sha256':sha(OUT/'host-x86.asm'),'input_sha256':sha(OUT/'input.bin'),'bitmap_guard':'owner +1B8 dword must equal 1; it is initialized zero, source two admitted positive weighted ordinary components both OR bit0','source_comparison':'all remaining 31 bits in the dword additionally checked against owner shadow','stack_scratch':'unchanged +500..7FF; conditional computed low +6D8 leaves 472 bytes +500..6D7 unchecked','limitations':'No complete reachable CFG or full stack/register/write proof. Dedicated sparse builder, source/linked review, runner audit and independent numeric comparison are absent; no native execution is authorized by this packet.','pins':{str(p.relative_to(R)):sha(p) for p in [Path(__file__),BASE/'manifest.json',BASE/'host-x86.asm',BASE/'input.bin']}}
 (OUT/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
