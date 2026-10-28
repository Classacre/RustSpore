"""Audit-only literal .const layout resolution for unchanged sampling host v5."""
import hashlib,json,re,struct
from pathlib import Path
import cell_runtime_fresh_sampled_channel_native_probe as original
from cell_runtime_fresh_sampled_channel_native_probe import *
from anim_root_pose_sparse_carrier_audit_v2 import Image,syms
assert sha(original.__file__)=='76ca25f38fb7aaf657d2e931683bf2aa32233ef670372cc0d5b01c7757a9fc36'
# Sparse page copying must split the full sampler at009A0000; assigning an
# overlong Python bytearray slice would silently enlarge page00990000.
APP_CODE=[]
for lo,hi in original.APP_CODE:
 while lo<hi:
  end=min(hi,(lo&~65535)+65536);APP_CODE.append((lo,end));lo=end
assert sum(b-a for a,b in APP_CODE)==sum(b-a for a,b in original.APP_CODE)

def coff_symbols(path):
 b=path.read_bytes();machine,n,stamp,ptr,count,optional,flags=struct.unpack_from('<HHIIIHH',b);assert machine==0x14c and optional==0
 strings=b[ptr+18*count:];assert len(strings)>=4 and struct.unpack_from('<I',strings)[0]<=len(strings)
 result={};i=0
 while i<count:
  at=ptr+18*i;name=b[at:at+8];value,section,kind,storage,aux=struct.unpack_from('<IhHBB',b,at+8)
  if name[:4]==bytes(4):off=struct.unpack_from('<I',name,4)[0];name=strings[off:strings.index(0,off)]
  else:name=name.split(b'\0')[0]
  name=name.decode('ascii');assert name not in result or result[name]==(section,value)
  if section>0:result[name]=(section,value)
  i+=1+aux
 assert i==count;return result

def linked_audit(exe,mapping,images):
 obj=mapping.with_suffix('.obj');locals=coff_symbols(obj);public=syms(mapping);bases={}
 for name,address in public.items():
  if name not in locals:continue
  section,offset=locals[name];base=address-offset
  assert section not in bases or bases[section]==base,(name,section)
  bases[section]=base
 # MASM omits unused non-PUBLIC labels from COFF. Decode the entire literal
 # .const block with a strict grammar, verify every public label and every byte.
 asm=(mapping.parent/'host-x86.asm').read_text();assert asm==original.host_source(images)
 block=asm.split('.const\n',1)[1].split('.data\n',1)[0];layout={};literal=bytearray()
 for line in block.splitlines():
  if line.startswith('PUBLIC '):continue
  m=re.fullmatch(r'(\w+) LABEL BYTE',line)
  if m:layout[m[1]]=len(literal);continue
  m=re.fullmatch(r'(\w+) BYTE (\d+) DUP\(0\)',line)
  if m:layout[m[1]]=len(literal);literal+=bytes(int(m[2]));continue
  assert line.startswith('DB '),line
  values=line[3:].split(',');assert all(re.fullmatch(r'[0-9A-F]{3}h',v)for v in values)
  literal+=bytes(int(v[:-1],16)for v in values)
 anchor=public['Original_00401930'];assert layout['Original_00401930']==0
 pe=Image(exe);assert pe.read(anchor,len(literal))==literal and pe.perm(anchor,len(literal))==0x40000000
 for name,offset in layout.items():
  if name in public:assert public[name]==anchor+offset
 added={};constants=[]
 for address,size in DATA:
  name=f'Original_{address:08X}'
  if name in public:continue
  offset=layout[name];resolved=anchor+offset
  assert pe.read(resolved,size)==images.read(address,size) and pe.perm(resolved,size)==0x40000000
  added[name]=resolved;constants.append(dict(name=name,source_address=f'{address:08X}',owned_address=f'{resolved:08X}',bytes=size,sha256=hashlib.sha256(images.read(address,size)).hexdigest(),literal_const_offset=offset,readonly_exact=True))
 assert added and all(name.startswith('Original_')for name in added)
 augmented=mapping.parent/'audit-local-constant-symbols-v2.map'
 text=mapping.read_text()+''.join(f'\n 0000:00000000 {name} {value:08X}\n'for name,value in sorted(added.items()))
 if augmented.exists():assert augmented.read_text()==text
 else:augmented.write_text(text)
 # Every prior linked assertion runs unchanged, over the same actual EXE.
 result=original.linked_audit(exe,augmented,images)
 result.update(audit_symbol_resolution='Strict literal .const layout; entire block byte/permission exact; every public label agrees; all public COFF/MAP section anchors agree; no executable edits',literal_const_bytes=len(literal),literal_const_sha256=hashlib.sha256(literal).hexdigest(),original_map_sha256=sha(mapping),object_sha256=sha(obj),augmented_map_sha256=sha(augmented),resolved_local_constants=constants,all_original_linked_checks_retained=True,unchanged_v5_preparation_tool_sha256=sha(original.__file__))
 return result
