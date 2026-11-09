from pathlib import Path
import hashlib,json,math,struct,sys
ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/'target/creature-skin-synthetic-deformation-inspection-20261109.json'
FOLDER=ROOT/'target/creature-skin-deform-directions-prep-v2-20261109'
PINS={'base':'09884d6953894b4fcc6ef5a25bba8e8bab314287889cb41de9f053ca1ae0127f','receipt':'6050374927dd93aea54e883c1ce726f584d2c73efebfcaab1719a69981d53c89','stdout':'09b8331e22b2b5318cf9a7d8defcd35d04ac89f64cd7d23f89940cb86dc11294','position':'38679f80f7ccbd81a2260c2ec20a2803317edff4b29cb72c3be8e307eeb9e0ba'}
def sha(b):return hashlib.sha256(b).hexdigest()
def compose(base,receipt,raw,position):
 for name,b in [('base',base),('receipt',receipt),('stdout',raw),('position',position)]:
  if sha(b)!=PINS[name]:raise ValueError(name+' hash mismatch')
 r=json.loads(receipt);a=json.loads(base)
 if r.get('status')!='matched' or r.get('host_exit')!=0 or r.get('host_timed_out') is not False or r.get('host_stdout_sha256')!=sha(raw):raise ValueError('invalid direction receipt')
 if len(raw)!=32024 or len(position)!=29984 or raw[:29984]!=position:raise ValueError('direction prefix mismatch')
 attr=a['conditional_eight_group_export']['attribute_words_hex']
 def pack(rows):return b''.join(struct.pack('<3I',*(int(x,16) for x in row)) for row in rows)
 if len(attr)!=4 or len(attr[1])!=85 or len(attr[3])!=85 or pack(attr[1])!=raw[22708:23728] or pack(attr[3])!=raw[24408:25428]:raise ValueError('direction source mapping mismatch')
 if pack(a['conditional_synthetic_cpu_deformation']['position_words_hex'])!=raw[28964:29984]:raise ValueError('position mapping mismatch')
 def rows(data):
  words=list(struct.iter_unpack('<3I',data))
  if len(words)!=85 or any(not math.isfinite(struct.unpack('<f',struct.pack('<I',x))[0]) for row in words for x in row):raise ValueError('invalid direction words')
  return [[f'{x:08x}' for x in row] for row in words]
 a['conditional_synthetic_cpu_directions']={'limits':'Original CPU direction deformation on retained 85 records with explicit synthetic matrices. Conditional fixture only; not a live pose, material, palette binding or renderer match. General x87 equivalence and lifecycle remain unverified.','provenance_sha256':PINS,'normal_words_hex':rows(raw[29984:31004]),'tangent_words_hex':rows(raw[31004:32024])}
 return a
if __name__=='__main__':
 if len(sys.argv)!=2:raise SystemExit('provide a new output path')
 p=Path(sys.argv[1]);out=compose(BASE.read_bytes(),(FOLDER/'result.json').read_bytes(),(FOLDER/'host-stdout.bin').read_bytes(),(ROOT/'target/creature-skin-deform-position-prep-v1-20261109/host-stdout.bin').read_bytes())
 with p.open('x',encoding='utf-8',newline='\n') as f:json.dump(out,f,indent=2,allow_nan=False);f.write('\n')
 print('wrote',p)
