"""Conditional retained-definition/en-us locale join; no game execution."""
import argparse,collections,hashlib,json,struct
from pathlib import Path
from locale_table_probe import tokenize
DEF_SHA='f0a940e9c6a2a6baa994f5df3b2b21b19bd316920a43cc9e77649547ed168673'
LOCALE_SHA='a6fbed4f4e2954689735149d2cf86cf1b1785eb43d4780d655fbad8bf3d4641c'
def main():
 p=argparse.ArgumentParser();p.add_argument('--definitions',type=Path,required=True);p.add_argument('--locale',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
 out=a.out.resolve();out.relative_to((Path(__file__).resolve().parent.parent/'target').resolve());out.mkdir(parents=True,exist_ok=False)
 data=(a.definitions/'corpus.bin').read_bytes();loc=a.locale.read_bytes();inv=json.loads((a.definitions/'inventory.json').read_text())
 assert hashlib.sha256(data).hexdigest()==DEF_SHA and hashlib.sha256(loc).hexdigest()==LOCALE_SHA and data[:8]==b'RSCELLD1'
 entries,_,_=tokenize(loc);mapping={i:txt for i,_,_,txt in entries};count=struct.unpack_from('<I',data,8)[0];assert count==len(inv['records'])==489
 rows=[];totals=collections.Counter();pos=12
 for i,item in enumerate(inv['records']):
  key,size,_=struct.unpack_from('<QIQ',data,pos);pos+=20;raw=data[pos:pos+size];pos+=size
  assert hashlib.sha256(raw).hexdigest()==item['sha256'] and size==796
  units=struct.unpack_from('<80H',raw,4);ident=struct.unpack_from('<I',raw,0xa4)[0]
  embedded=bool(units[0]);terminated=0 in units
  text=mapping.get(ident,b'');nonempty=bool(text and text[:2]!=b'\x00\x00')
  category='locale_nonempty' if nonempty else ('embedded_fallback' if embedded else 'creation_fallback_required')
  totals[category]+=1
  if embedded:assert terminated
  rows.append({'ordinal':i,'package':item['package'],'key':item['key'],'definition_sha256':item['sha256'],
               'legacy_fixture_key':f'{key:016X}','locale_id':f'{ident:08X}','locale_nonempty':nonempty,
               'embedded_nonempty':embedded,'embedded_terminated_within_80':terminated,'category':category})
 assert pos==len(data)
 result={'definitions_sha256':DEF_SHA,'locale_sha256':LOCALE_SHA,'table_id':'04DC2F94','locale':'en-us',
         'source_executed':False,'resources':count,'counts':dict(totals),'rows':rows,
         'limits':'retained source copies and selected en-us table only; actual overlay, provider state and live actor binding not established'}
 (out/'report.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf8');print(json.dumps({'resources':count,'counts':dict(totals),'source_executed':False}))
if __name__=='__main__':main()
