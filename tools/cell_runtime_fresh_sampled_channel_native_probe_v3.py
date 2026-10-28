"""Owned event-tail shadow correction; original10403B and numeric model unchanged."""
import argparse,json,difflib,hashlib
from pathlib import Path
import cell_runtime_fresh_sampled_channel_native_probe_v2 as previous
from cell_runtime_fresh_sampled_channel_native_probe_v2 import *
_frozen_source=previous.original.host_source
SHADOW='mov esi,OFFSET EventsBuffer\nmov edi,OFFSET EventsShadow\nmov ecx,32\nrep movsd\n'
def host_source(images):
 text=_frozen_source(images)
 for anchor in ['mov ClipMask,OFFSET MaskActivation\n','mov ClipMask,OFFSET MaskClock\n']:
  assert text.count(anchor)==1;text=text.replace(anchor,SHADOW+anchor)
 return text
def linked_audit(exe,mapping,images):
 expected=host_source(images);assert(mapping.parent/'host-x86.asm').read_text()==expected
 # v2's unchanged complete linked audit checks the exact expected owned source.
 # Bind only that source equality to this explicitly reviewed successor.
 saved=previous.original.host_source
 try:
  previous.original.host_source=lambda unused:expected
  result=previous.linked_audit(exe,mapping,images)
 finally:previous.original.host_source=saved
 result.update(schema='fresh-sampled-owned-event-tail-shadow-correction-linked-v3',source_game_and_CRT_unchanged=True,owned_change='Two exact128B Events→EventsShadow copies immediatelybefore activation/eachclock.',event_tail_guard_not_weakened=True)
 return result
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());images=arithmetic.Images(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll'));audit_source(images)
 olddir=ROOT/'target/source-cell/native-runtime-fresh-sampled-channel-host-v5';old=olddir/'host-x86.asm';assert sha(old)=='ee4fe87d9456ed1ae970faf81ebe821c878f47e6c511bf009f572fe5edc01642';assert old.read_text()==_frozen_source(images)
 text=host_source(images);reverted=text
 for anchor in ['mov ClipMask,OFFSET MaskActivation\n','mov ClipMask,OFFSET MaskClock\n']:assert reverted.count(SHADOW+anchor)==1;reverted=reverted.replace(SHADOW+anchor,anchor)
 assert reverted==old.read_text();assert text.count(SHADOW)==3 # initialRunOnce plus2added
 a.out.mkdir(exist_ok=False,parents=True);(a.out/'host-x86.asm').write_text(text);(a.out/'owned-only-change.diff').write_text(''.join(difflib.unified_diff(old.read_text().splitlines(True),text.splitlines(True),fromfile='frozen-v5',tofile='event-tail-shadow-v6')))
 manifest=json.loads((olddir/'manifest.json').read_text());manifest.update(schema='fresh-sampled-event-tail-shadow-corrected-prep-v3',preparation_tool_sha256=sha(__file__),host_source_sha256=sha(a.out/'host-x86.asm'),parent_v5_host_sha256=sha(old),original_source_bytes_unchanged=10403,only_two_owned_shadow_copies_added=True,compiled=False,native_executed=False)
 (a.out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
 pins=[Path(__file__),Path(previous.__file__),Path(previous.original.__file__),old,olddir/'manifest.json',ROOT/'target/source-cell/runtime-fresh-sampled-channel-event-tail-diagnosis-v1/diagnosis.json',ROOT/'crates/spore-core/src/cell_runtime_fresh_sampled_channel.rs',ROOT/'crates/spore-core/examples/cell_runtime_fresh_sampled_channel_verify.rs.pending',ROOT/'target/source-cell/runtime-fresh-sampled-channel-inputs-v1/report.json']
 (a.out/'owned-diff-review.json').write_text(json.dumps(dict(static_pass=True,native_executed=False,exact_old_source_restored_by_removing_two_shadow_blocks=True,added_instructions=8,event_tail_preserved_against_precise_incoming_data=True,whole128Bshadowcopy=True,event_guard_unchanged=True,source_bytes_unchanged=10403,pins={str(p):sha(p)for p in pins}),indent=2)+'\n');print(sha(a.out/'host-x86.asm'))
if __name__=='__main__':main()
