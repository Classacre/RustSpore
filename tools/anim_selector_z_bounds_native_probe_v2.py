"""Owned-only Z publisher stack guard correction; frozen v1 remains unchanged."""
import sys,json,hashlib
from pathlib import Path
import anim_selector_z_bounds_native_probe_v1 as base
BASE_SHA='ec92dab04cd562b2819c42f712097b9c6118450201900adaa5a8617daf6b8ece'
OLD_ASM='b1f5d344c573b5c2f2c7a00b4b183865f7dc43afa0b71c5eab71fc534ae31e75'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 assert sha(base.__file__)==BASE_SHA
 original=base.assembly
 def corrected(out,p):
  text=original(out,p);old='mov esi,SourceStack\nadd esi,24\nmov edi,StackWindow\n';new='mov esi,SourceStack\nadd esi,36\nmov edi,StackWindow\n';assert text.count(old)==1
  assert hashlib.sha256(text.replace(chr(10),chr(13)+chr(10)).encode()).hexdigest()==OLD_ASM
  return text.replace(old,new)
 base.assembly=corrected
 try:base.main()
 finally:base.assembly=original
 args=sys.argv[1:];out=Path(args[args.index('--out')+1])
 if '--owned-exe'in args:return
 manifest=json.loads((out/'manifest.json').read_text());manifest.update(schema='original-Z-producer-fieldwriter-native-preparation-v2',tool_sha256=sha(__file__),base_tool_sha256=BASE_SHA,base_asm_sha256=OLD_ASM,asm_sha256=sha(out/'host-x86.asm'),stack_scope='RealStackWindow768B+two16Bguards; SourceS=window+180h. Original caller writes S+4..35; untouched upper guard begins S+36. Exit ESP=S-12; retainedx87 TOP7 captured80 then TOP0. BelowS source callee scratch allowed; S..3 remains untouched. Failure restores DriverStack before Windows API.',owned_change='One ADD ESI immediate24->36 in CheckStackUpper setup. All original/source/data/input/model/ABI instructions otherwise identical.')
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
 review=dict(schema='Z-publisher-owned-stack-guard-correction-v2',base_tool_sha256=BASE_SHA,base_asm_sha256=OLD_ASM,tool_sha256=sha(__file__),asm_sha256=sha(out/'host-x86.asm'),source_bytes=3396,source_model_inputs_unchanged=True,old_preparation_preserved=True,native_executed=False,compiled=False,execution_admitted=False,argument_derivation=['entryESP=S','PUSH ESI => S-4','LEA EAX,[ESP+1Ch] => outMax=S+24','PUSH EAX => S-8','LEA ECX,[ESP+14h] => outMin=S+12','PUSH ECX => S-12','helper writes outMin S+12..23 and outMax S+24..35','caller scratch Y/Z S+4..11','untouched above-S bytes are S..3 and S+36..end'],owned_change='Exactly one ADD ESI,24 becomes ADD ESI,36; no original instruction changes',open_gates=['fresh root build','independent actual linked and partial-entry operation-order peer','new runner pins actual v2 identity','native and Rust comparison'])
 (out/'owned-guard-correction.json').write_text(json.dumps(review,indent=2)+'\n')
if __name__=='__main__':main()

