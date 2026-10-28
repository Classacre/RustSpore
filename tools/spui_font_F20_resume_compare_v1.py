"""Compare ROOT Rust outputs to independently source-derived canonical rows."""
import hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main(actual,expected,output):
 a,e,o=map(Path,[actual,expected,output]);o.resolve().relative_to((ROOT/'target').resolve());assert not o.exists()
 raw=a.read_bytes();ref=e.read_bytes();assert b'\r' not in ref
 got=raw.decode().splitlines();want=ref.decode().splitlines();statuses={};numeric=[]
 for row in got:
  v=row.split('\t')
  if len(v)>2 and v[1]=='status':
   assert v[0] not in statuses;statuses[v[0]]=v[2:]
  else:numeric.append(row)
 # Full-frame batch expected omits statuses; compare exact complete-glyph projection.
 if statuses and not any('\tstatus\t' in row for row in want):
  complete={k for k,v in statuses.items()if v==['complete']};projected=[row for row in want if row.split('\t',1)[0].split(':',1)[0]in complete];assert numeric==projected,'numeric projection mismatch'
 else:assert got==want,'exact source comparison mismatch'
 report=dict(schema='font-F20-root-source-comparison-v1',actual_sha256=sha(a),source_sha256=sha(e),numeric_rows=len(numeric),statuses=statuses,complete=sum(v==['complete']for v in statuses.values()),rows_equal=True,scope='Actual whole program results only. Status exclusions retained. No completion inferred from first stops.')
 o.write_bytes((json.dumps(report,indent=2)+'\n').encode());print(json.dumps({k:v for k,v in report.items()if k!='statuses'},indent=2))
if __name__=='__main__':main(*sys.argv[1:])
