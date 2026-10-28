"""Independent original opcode setter proof and two-field reference correction.
Only source x86 is interpreted; no Rust or native executable runs.
"""
import hashlib,json,struct,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
from spui_font_visible_raster_scout import load_source
from spui_font_visible_hint_scout import PrepMachine,FRAME
from spui_font_scaling_probe import STACK

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(out):
 out=Path(out).resolve();out.relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False);image,frontier,ins=load_source();setters=[]
 for opcode,entry,axis in [(0,0x8abee0,1),(1,0x8abf20,0)]:
  assert int.from_bytes(image.read(0x1432f30+opcode*4,4),'little')==entry
  row=next(r for r in frontier['functions'] if int(r['entry'],16)==entry);original=b''.join(image.read(int(a,16),int(b,16)-int(a,16)) for a,b in row['ranges']);assert hashlib.sha256(original).hexdigest()==row['sha256']
  m=PrepMachine(ins);m.region(STACK-128,bytes(256));m.region(FRAME,bytes([0xa5])*256);m.reg('esp',STACK);m.word(STACK,value=0xffffffff);m.word(STACK+4,value=FRAME);m.run(entry,0xffffffff,100);assert m.reg('esp')==STACK+4
  raw=m.bytes(FRAME,256);vectors=struct.unpack_from('<4H',raw,0x14);callbacks=tuple(struct.unpack_from('<I',raw,at)[0] for at in [0x44,0x48,0x4c]);allowed=set(range(0x14,0x1c))|set(range(0x40,0x42))|set(range(0x44,0x50));assert all(v==0xa5 for i,v in enumerate(raw) if i not in allowed)
  expected=((0,0x4000,0,0x4000),(0x8ab9a0,0x8aba40,0x8aba40)) if axis else ((0x4000,0,0x4000,0),(0x8ab970,0x8aba30,0x8aba30));assert(vectors,callbacks)==expected and struct.unpack_from('<H',raw,0x40)[0]==0x4000
  setters.append(dict(opcode=f'{opcode:02X}',dispatch_table_slot=f'{0x1432f30+opcode*4:08X}',entry=f'{entry:08X}',source_record=row,original_instructions_interpreted=True,normalized_axis=axis,projection_freedom_words=[f'{v:04X}' for v in vectors],move_projection_dual_callbacks=[f'{v:08X}' for v in callbacks],all_other_frame_bytes_unchanged=True))
 old=ROOT/'target/source-ui/font-curved-fixtures-v2/expected.tsv';new=ROOT/'target/source-ui/font-curved-fixtures-v3/expected.tsv';rust=ROOT/'target/source-ui/font-curved-rust-v1.tsv';a=old.read_text().splitlines();b=new.read_text().splitlines();assert len(a)==len(b)==2291;diff=[]
 for i,(before,after) in enumerate(zip(a,b)):
  if before==after:continue
  x,y=before.split('\t'),after.split('\t');assert x[0] in ['gid49','gid54'] and x[1]=='final' and x[3]=='0' and y[3]=='1' and x[:3]==y[:3] and x[4:]==y[4:];diff.append(dict(zero_based_row=i,before=before,after=after))
 assert len(diff)==2
 frameproof=[]
 for gid in [37,49,54]:
  p=ROOT/f'target/source-ui/font-curved-fixtures-v3/fresh-hint/gid{gid}-hint.json';h=json.loads(p.read_text());frame=bytes.fromhex(h['frame_hex']);vectors=struct.unpack_from('<4H',frame,0x14);callbacks=tuple(struct.unpack_from('<I',frame,o)[0] for o in [0x44,0x48,0x4c]);last=next(t for t in reversed(h['trace']) if t['stream']=='glyph0' and t['opcode'] in [0,1]);setter=next(s for s in setters if int(s['opcode'],16)==last['opcode']);assert list(vectors)==[int(v,16) for v in setter['projection_freedom_words']] and list(callbacks)==[int(v,16) for v in setter['move_projection_dual_callbacks']];frameproof.append(dict(gid=gid,last_source_axis_opcode=last,normalized_axis=setter['normalized_axis'],frame_vectors=list(vectors),frame_callbacks=list(callbacks),hint_sha256=sha(p)))
 assert new.read_bytes()==rust.read_bytes()
 paths=[Path(__file__),ROOT/'tools/spui_font_curved_fixtures_v3.py',ROOT/'tools/spui_font_curved_hint_reference_v1.py',ROOT/'tools/spui_font_rectilinear_fixtures_v3.py',ROOT/'target/source-ui/font-curved-rust-review-packet-v1.json',old,new,rust,ROOT/'target/source-ui/font-fixture-frontier-v7.json']
 report=dict(schema='curved-final-axis-source-reference-correction-v3',new_native_execution=False,rust_execution=False,source_opcode_setter_proof=setters,actual_source_final_frame_proof=frameproof,canonical_differences=diff,canonical_rows=2291,old_reference_sha256=sha(old),new_reference_sha256=sha(new),retained_first_rust_sha256=sha(rust),new_reference_equals_retained_first_rust=True,rust_numeric_model_changed=False,expected_generated_from_rust=False,reason='Historical inherited formatter hardcoded normalized axis0; original P/U frame and opcode00 prove Yaxis1. All other2291-row fields unchanged. Source frames are source-instruction evidence,not native full-frame observations.',pins={str(p.relative_to(ROOT)):sha(p) for p in paths});p=out/'report.json';p.write_text(json.dumps(report,indent=2)+'\n');print(sha(p))
if __name__=='__main__':main(sys.argv[1])
