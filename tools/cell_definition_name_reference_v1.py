"""Independent retained-definition/en-US conditional name reference (no source execution)."""
import hashlib,struct
from pathlib import Path
from locale_table_probe import tokenize
root=Path('target/source-cell/definition-corpus-v4/corpus.bin');locale=Path('target/source-locale/corpus-v2/sources/01169.locale')
data=root.read_bytes();loc=locale.read_bytes()
assert hashlib.sha256(data).hexdigest()=='f0a940e9c6a2a6baa994f5df3b2b21b19bd316920a43cc9e77649547ed168673'
assert hashlib.sha256(loc).hexdigest()=='a6fbed4f4e2954689735149d2cf86cf1b1785eb43d4780d655fbad8bf3d4641c'
entries,_,_=tokenize(loc);table={i:text for i,_,_,text in entries};assert data[:8]==b'RSCELLD1'
count=struct.unpack_from('<I',data,8)[0];assert count==489
pos=12;rows=[];counts={'locale':0,'embedded':0,'creation':0}
for i in range(count):
 key,size,_=struct.unpack_from('<QIQ',data,pos);pos+=20;raw=data[pos:pos+size];pos+=size;assert len(raw)==size==796
 ident=struct.unpack_from('<I',raw,0xa4)[0];text=table.get(ident,b'')
 if text and text[:2]!=b'\x00\x00':category='locale';units=struct.unpack('<'+'H'*(len(text)//2),text)
 else:
  label=struct.unpack_from('<80H',raw,4)
  if label[0]:category='embedded';assert 0 in label;units=label[:label.index(0)]
  else:category='creation';units=()
 counts[category]+=1;rows.append(f'{i}\t{ident:08x}\t{category}\t'+''.join(f'{u:04x}' for u in units)+'\n')
assert pos==len(data)
out=Path('target/source-cell/definition-name-compare-v1');out.mkdir(parents=True,exist_ok=False)
b=''.join(rows).encode();(out/'python-expected.tsv').write_bytes(b)
print('count',count,'categories',counts,'sha256',hashlib.sha256(b).hexdigest())
