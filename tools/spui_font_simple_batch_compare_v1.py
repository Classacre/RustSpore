"""Compare batch completed glyph groups against independent retained source targets."""
import argparse,collections,hashlib,json
from pathlib import Path
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('fixture',type=Path);p.add_argument('actual',type=Path);p.add_argument('out',type=Path);a=p.parse_args();m=json.loads((a.fixture/'manifest.json').read_text());assert sha(a.fixture/'expected.tsv')==m['canonical_sha256'];expected=collections.defaultdict(list)
 for l in (a.fixture/'expected.tsv').read_text().splitlines():expected[l.split('\t')[0].split(':')[0]].append(l)
 actual=collections.defaultdict(list);status={}
 for l in a.actual.read_text().splitlines():
  f=l.split('\t');key=f[0].split(':')[0]
  if len(f)>1 and f[1]=='status':assert key not in status;status[key]=f[2:]
  else:actual[key].append(l)
 assert set(status)=={f'gid{i}' for i in range(578)}
 results=[];errors=collections.Counter()
 for key,value in status.items():
  if value==['complete']:
   want=expected[key];got=actual[key];diff=next((i for i,(x,y) in enumerate(zip(want,got)) if x!=y),None);assert len(want)==len(got) and diff is None,(key,len(want),len(got),diff, None if diff is None else [want[diff],got[diff]])
   results.append(key)
  else:assert value[0]=='excluded' and not actual[key];errors[value[1]]+=1
 report=dict(schema='font0-simple-batch-rust-comparison-v1',all_completed_exact=True,completed_glyphs=len(results),completed=results,typed_exclusions=dict(errors),fixture_sha256=sha(a.fixture/'manifest.json'),actual_sha256=sha(a.actual),native_executed=False,scope='Successful bounded fresh15ppem font0 whole state/raw pixels/final origins only; excluded glyphs unaccepted; no full text/UI/cache/lifetime admission.')
 with a.out.open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(len(results),dict(errors))
if __name__=='__main__':main()
