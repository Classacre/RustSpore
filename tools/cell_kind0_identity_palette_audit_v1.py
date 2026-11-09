"""Audit stored three-weight preconditions for the controlled kind-0 fixture."""
import argparse,hashlib,json,struct
from pathlib import Path
from dbpf_extract import payload
from gmdl_skin_probe import indexes
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(4*1024*1024),b''):h.update(b)
 return h.hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('package',type=Path);p.add_argument('out',type=Path);p.add_argument('--admit-stable-path',required=True,action='store_true');a=p.parse_args();root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 before=sha(a.package)
 if before!='e01bb86d8ddf02c56d52394b4e1db7181c1e1ebe1878bf6fb72c4ee523fcedd1':raise ValueError('package pin')
 es=[e for e in indexes(a.package) if (e['type'],e['instance'])==(0x2f4e681b,0xdae8ce17)]
 if len(es)!=1:raise ValueError('key')
 b=payload(a.package,es[0]);h=hashlib.sha256(b).hexdigest()
 if h!='bbd99a741d41a3606a310f242e14e5f85dc7b49af454972ff02124b16e318592':raise ValueError('RW4 pin')
 table=struct.unpack_from('<I',b,48)[0];base=struct.unpack_from('<I',b,68)[0];v=struct.unpack_from('<6I',b,table+3*24)
 if v[5]!=0x10030 or v[2]!=454*36:raise ValueError('vertex source framing')
 data=b[base+v[0]:base+v[0]+v[2]];rows=[data[i:i+36] for i in range(0,len(data),36)]
 report={'package_sha256':before,'rw4_sha256':h,'vertices':len(rows),'fourth_nonzero':sum(r[35]!=0 for r in rows),'first_three_not_255':sum(sum(r[32:35])!=255 for r in rows),'indices_invalid':sum(any(x>24 or x%3 for x in r[28:32]) for r in rows),'scope':'stored vertex bytes only; identity-palette fixture admission, not runtime weights or FP equivalence'}
 if sha(a.package)!=before:raise ValueError('package changed')
 with a.out.open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()
