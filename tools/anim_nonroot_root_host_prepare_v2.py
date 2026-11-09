"""Revise uncompiled root host guards; no native execution or compilation."""
import hashlib,json,sys
from pathlib import Path
R=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(R/'tools'))
from cell_schema_probe import SourceImage
BASE=R/'target/animation/nonroot-root-host-packet-v1';OUT=R/'target/animation/nonroot-root-host-packet-v2-final'
SOURCE=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe')
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 assert sha(BASE/'host-x86.asm')=='1fe8759037111c48064b9746d377068c2b95e4af0fd9ffd7fefab8fe026aa4b4'
 assert sha(BASE/'manifest.json')=='cb2d1593941e42a96b4a3af2276cea0c8ee1ce4c9fe672e536265c10fd343acc'
 image=SourceImage(SOURCE)
 m=json.loads((BASE/'manifest.json').read_text());assert sha(BASE/'input.bin')==m['input_sha256']
 lines=(BASE/'host-x86.asm').read_text().splitlines()
 def replace(old,new):
  at=lines.index(old);assert lines.count(old)==1;lines[at:at+1]=new
 # Compare all four owner bytes, allowing only root bitmap bit 0.
 at=lines.index('CheckOwner:');assert lines[at+1:at+3]==['cmp ecx,01B8h','je NextOwner']
 lines[at+1:at+3]=['cmp ecx,01B8h','jne CompareOwner','mov eax,dword ptr [OwnerBuffer+ecx]','xor eax,dword ptr [OwnerShadow+ecx]','test eax,0FFFFFFFEh','jnz HostFailure','jmp NextOwner']
 # Conversion has its own frame upper-range check before setup writes.
 at=lines.index('mov esi,OFFSET ConvertedBuffer');lines[at:at]=['mov esi,OFFSET FrameBuffer+0800h','mov edi,OFFSET FrameShadow+0800h','mov ecx,2816','repe cmpsd','jne HostFailure']
 # Snapshot owned frame initialization before the partial accumulation.
 at=lines.index('mov dword ptr [FrameBuffer+02E0Ch],OFFSET OwnerBuffer')+1
 assert lines[at]=='mov esi,OFFSET PoseBuffer'
 lines[at:at]=['mov esi,OFFSET FrameBuffer','mov edi,OFFSET FrameShadow','mov ecx,3328','rep movsd']
 a=lines.index('CheckFrame:');b=lines.index('CompareFrame:',a)
 lines[a+1:b]=['cmp ecx,0500h','jb CompareFrame','cmp ecx,0800h','jb NextFrame',
  'cmp ecx,0854h','je NextFrame','cmp ecx,0878h','je NextFrame',
  'cmp ecx,08C0h','jb FrameCacheCheck','cmp ecx,08F0h','jb NextFrame',
  'FrameCacheCheck:','cmp ecx,0A28h','jb CompareFrame','cmp ecx,0A4Ch','jb NextFrame']
 OUT.mkdir(parents=True,exist_ok=False);(OUT/'input.bin').write_bytes((BASE/'input.bin').read_bytes());(OUT/'host-x86.asm').write_text('\n'.join(lines)+'\n')
 audit={'schema':'source-root-host-v2-static-proposal','compiled':False,'native_executed':False,'admitted_for_execution':False,
  'source_sha256':sha(SOURCE),'asm_sha256':sha(OUT/'host-x86.asm'),'input_sha256':sha(OUT/'input.bin'),
  'owner_guard':'all words compared; +1B8 dword masks only bit 0 and compares remaining 31 bits against shadow',
  'frame_conversion':'compare +800..+33FF with original shadow immediately after conversion, before owned partial setup',
  'frame_accumulation':'re-shadow after owned setup; compare all except stack scratch +500..+7FF, locals +854..857, +878..87B, +8C0..8EF, root cache +A28..A4B',
  'frame_scratch_warning':'500..7FF remains generous; exact deepest stack/write bound across all called original functions and owned pushes not yet independently established',
  'conditional_unreachable':{'0099BE70':'POS descriptor 71 excludes bit200 and flags&120==100; channel bit0 clear; no wrapper propagation bit0',
   '009A48B0':'channel bit0 clear, skipping relative branch','009A4DF0':'channel bit0 clear, skipping relative rotation branch','009B0150':'channel bit0 clear','009B0530':'channel bit0 clear','009B05E0':'channel bit0 clear','009B0AC0':'channel bit0 clear','009B75B0':'channel bit0 clear and globals 1551A70,1551A74 zero','009CDA70':'wrapper+10 bit0 zero'},
  'conditional_proof_limits':'Branch rationale derives from source flags and supplied globals only; mask 9 does not select POS71 9B0B90 B1 branch. Independent instruction-level control-flow and stack audit remains mandatory.',
  'remaining':'No dedicated sparse carrier builder or linked/runner audit exists; reproduce + verify 99CB70 original mapping and RX permissions before any execution.',
  'pins':{str(p.relative_to(R)):sha(p) for p in [Path(__file__),BASE/'manifest.json',BASE/'host-x86.asm',BASE/'input.bin']}}
 (OUT/'manifest.json').write_text(json.dumps(audit,indent=2)+'\n');print(json.dumps(audit,indent=2))
if __name__=='__main__':main()
