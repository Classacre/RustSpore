"""Bounded metadata requests; raw source words remain explicit until reader mapping gate."""
import argparse,json,struct,hashlib,csv
from pathlib import Path
from cell_runtime_curves_fixture import CONTEXTS
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def f(x):return struct.unpack('<f',struct.pack('<I',x))[0]
def bits(x):
 try:return struct.unpack('<I',struct.pack('<f',x))[0]
 except OverflowError:return 0x7f800000
def request(cw=0x027f,flags=8,frames=0x43050000,interval=0x3d088889):return struct.pack('<HHIII',cw,0,flags,frames,interval)+bytes(16)
def bounded(frames,interval):
 if not(0x3f800000<frames<0x7f800000 and 0<interval<0x7f800000):return False
 # Each SSE32 operand is exactly representable64. Product of two32 has<=48
 # significantbits; this offline domain proof excludes zero/overflow products.
 span=bits(f(frames)-1.);product=bits(f(span)*f(interval));return 0<product<0x7f800000

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());m=json.loads((a.host_dir/'manifest.json').read_text());assert sha(a.host_dir/'host-x86.asm')==m['host_source_sha256'];a.out.mkdir(parents=True,exist_ok=False)
 frames=[0x3f800001,0x40000000,0x428a0000,0x43050000,0x4b000001,0x4b800000,0x7f7fffff];intervals=[1,0x00800000,0x33800000,0x3d088889,0x3f000001,0x3f800000,0x7f7fffff];flags=[0,8,0xfffffff7,0xffffffff];pairs=[]
 for fr in frames:
  for iv in intervals:
   if bounded(fr,iv):pairs.append((fr,iv,'bounded'))
 clips=ROOT/'target/source-cell/runtime-root-data-join-v3/clips'
 sourcepins={}
 for path in sorted(clips.glob('*.anim')):
  raw=path.read_bytes();fl,iv,fr=struct.unpack_from('<III',raw,280);sourcepins[str(path)]=sha(path)
  if bounded(fr,iv):pairs.append((fr,iv,'raw_source_header_'+sha(path)))
 cases=[]
 with(a.out/'input.bin').open('xb')as binary:
  for cw in CONTEXTS:
   for fr,iv,family in pairs:
    for fl in flags:binary.write(request(cw,fl,fr,iv));cases.append(dict(case=len(cases),family=family,cw=f'{cw:04X}',flags=f'{fl:08X}',frames=f'{fr:08X}',interval=f'{iv:08X}'))
 (a.out/'cases.json').write_text(json.dumps(cases,indent=2)+'\n');protocols={}
 valid=request()
 reject={'cw_unmasked':(0,struct.pack('<H',0x027e)),'cw_pc_reserved':(0,struct.pack('<H',0x017f)),'reserved_header':(2,b'\x01\0'),'reserved_tail':(16,b'\x01'),'frames_one':(8,struct.pack('<I',0x3f800000)),'frames_nan':(8,struct.pack('<I',0x7fc12345)),'interval_zero':(12,bytes(4)),'interval_negative':(12,struct.pack('<I',0xbf800000)),'interval_inf':(12,struct.pack('<I',0x7f800000)),'product_underflow':(8,struct.pack('<II',0x3f800001,1)),'product_overflow':(8,struct.pack('<II',0x7f7fffff,0x7f7fffff))}
 for n,(at,value)in reject.items():raw=bytearray(valid);raw[at:at+len(value)]=value;(a.out/(n+'.bin')).write_bytes(raw);protocols[n]=sha(a.out/(n+'.bin'))
 pins={str(p):sha(p)for p in[Path(__file__),ROOT/'tools/cell_runtime_clip_metadata_native_probe.py',ROOT/'crates/spore-core/src/cell_runtime_clip_metadata.rs',ROOT/'crates/spore-core/examples/cell_runtime_clip_metadata_verify.rs.pending',a.host_dir/'manifest.json',a.host_dir/'host-x86.asm',ROOT/'target/rea/20261027-source-runtime-v4/38-analyze_function.json',ROOT/'target/rea/20261027-source-runtime-v4/39-analyze_function.json',ROOT/'target/rea/20261027-source-runtime-v4/48-analyze_function.json']};pins.update(sourcepins)
 (a.out/'report.json').write_text(json.dumps(dict(schema='source-cell-clip-metadata-inputs-v1',cases=len(cases),context_count=len(CONTEXTS),pairs=len(pairs),protocols=protocols,pins=pins,input_sha256=sha(a.out/'input.bin'),cases_sha256=sha(a.out/'cases.json'),source_headers='Explicit raw words only. ANIM-header-to-runtime-resource mapping not yet admitted. Flag variants synthetically exercise bit3 extraction.',domain_proof='All requests satisfy bounded finitepositive product test, no expected native projection synthesized.',no_execution=True),indent=2)+'\n');print(len(cases))
if __name__=='__main__':main()
