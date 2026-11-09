from pathlib import Path
import hashlib,json,math,struct,sys
ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/'target/creature-skin-synthetic-deformation-inspection-20261109.json'
FOLDER=ROOT/'target/creature-skin-quaternion-product-deform-prep-v4-20261109'
PINS={'base':'09884d6953894b4fcc6ef5a25bba8e8bab314287889cb41de9f053ca1ae0127f','receipt':'4da5fd8744fe5208a7441a8ab5d53fb09493d0d1239ecf1b2d3a87ae1250ac5c','stdout':'6ffcfe6dad216cf13f4128a9b4022a36fd09cf1be046ac81571332b218fa446d','prior':'09b8331e22b2b5318cf9a7d8defcd35d04ac89f64cd7d23f89940cb86dc11294','manifest':'47a0ca23c4af08569b106964c98e81fac2471a48ffeabc9d0d03521d47de50a0'}
def sha(b):return hashlib.sha256(b).hexdigest()
def compose(base,receipt,raw,prior,manifest,position_model,direction_model,palette):
 for name,b in [('base',base),('receipt',receipt),('stdout',raw),('prior',prior),('manifest',manifest)]:
  if sha(b)!=PINS[name]:raise ValueError(name+' hash mismatch')
 m=json.loads(manifest)
 if sha(position_model)!=m['deform-model.bin'] or sha(direction_model)!=m['directions-model.bin'] or sha(palette)!=m['synthetic-palette.bin'] or len(palette)!=480:raise ValueError('model hash mismatch')
 r=json.loads(receipt);a=json.loads(base)
 if r.get('status')!='matched' or r.get('host_exit')!=0 or r.get('host_timed_out') is not False or r.get('host_stdout_sha256')!=sha(raw):raise ValueError('invalid receipt')
 if len(raw)!=32024 or len(prior)!=32024 or raw[:28964]!=prior[:28964] or raw[28964:29984]!=position_model or raw[29984:]!=direction_model or raw[28964:]==prior[28964:]:raise ValueError('prefix or model mismatch')
 attr=a['conditional_eight_group_export']['attribute_words_hex'];skin=a['conditional_same_object_export']
 def pack3(rows):return b''.join(struct.pack('<3I',*(int(x,16) for x in row)) for row in rows)
 if len(attr)!=4 or any(len(attr[i])!=85 for i in (0,1,3)) or pack3(attr[0])!=raw[21688:22708] or pack3(attr[1])!=raw[22708:23728] or pack3(attr[3])!=raw[24408:25428]:raise ValueError('attribute mapping mismatch')
 if b''.join(struct.pack('<4H',*(int(x,16) for x in row)) for row in skin['index_offset_words_hex'])!=raw[25428:26108] or b''.join(struct.pack('<4I',*(int(x,16) for x in row)) for row in skin['weight_words_hex'])!=raw[26108:27468]:raise ValueError('skin mapping mismatch')
 def rows(data):
  words=list(struct.iter_unpack('<3I',data))
  if len(words)!=85 or any(not math.isfinite(struct.unpack('<f',struct.pack('<I',x))[0]) for row in words for x in row):raise ValueError('nonfinite output')
  return [[f'{x:08x}' for x in row] for row in words]
 a['conditional_synthetic_cpu_deformation']={'limits':'Conditional original CPU positions with alternating synthetic identity and 180-degree composite rotations; no live pose, palette, material or identity.','provenance_sha256':PINS,'synthetic_palette_words_hex':[f'{x[0]:08x}' for x in struct.iter_unpack('<I',palette)],'position_words_hex':rows(raw[28964:29984])}
 a['conditional_synthetic_cpu_directions']={'limits':'Conditional original CPU directions with alternating synthetic identity and 180-degree composite rotations; not a live pose or material, and not general x87 or animation proof.','provenance_sha256':PINS,'normal_words_hex':rows(raw[29984:31004]),'tangent_words_hex':rows(raw[31004:32024])}
 return a
if __name__=='__main__':
 if len(sys.argv)!=2:raise SystemExit('provide a new target output path')
 path=Path(sys.argv[1]);parent=path.parent.resolve();root=(ROOT/'target').resolve()
 if not parent.is_relative_to(root):raise ValueError('output must be inside target')
 out=compose(BASE.read_bytes(),(FOLDER/'result.json').read_bytes(),(FOLDER/'host-stdout.bin').read_bytes(),(ROOT/'target/creature-skin-quaternion-product-deform-prep-v3-20261109/host-stdout.bin').read_bytes(),(FOLDER/'manifest.json').read_bytes(),(FOLDER/'deform-model.bin').read_bytes(),(FOLDER/'directions-model.bin').read_bytes(),(FOLDER/'synthetic-palette.bin').read_bytes())
 with path.open('x',encoding='utf-8',newline='\n') as f:json.dump(out,f,indent=2,allow_nan=False);f.write('\n')
 print('wrote',path)
