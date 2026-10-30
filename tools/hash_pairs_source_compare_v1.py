"""Compare bounded Rust parser output with independent projections of installed metadata.

All source payloads stay in memory; only an aggregate receipt is written under target.
Requires a separately built, caller-pinned example executable; never builds it.
"""
import argparse, collections, hashlib, json, struct, subprocess
from pathlib import Path
from cell_hierarchy_76f930_compare_v1 import TARGET, digest, expected_hash, inside_target
from cell_kind4_list_source_verify_v1 import native_rows
from refpack import _read_header, decompress

PIN='3de1933446c61546f8afcf44e22c1010a5405baf353d0d5c9259ea6071f4e7c6'
MAX_BATCH=64*1024*1024
MAX_CASE=1024*1024

def project(raw):
    """Independent strict byte-level section projection; no Rust parser logic."""
    if len(raw)<8 or struct.unpack_from('>I',raw)[0]!=4:
        raise ValueError('bad version or empty section framing')
    pos=4; sections=[]
    while pos<len(raw):
        if pos+4>len(raw): raise ValueError('truncated section count')
        n=struct.unpack_from('>I',raw,pos)[0]; pos+=4
        if n>4096 or pos+12*n>len(raw): raise ValueError('invalid section extent')
        records=[]
        for _ in range(n):
            records.append(struct.unpack_from('>III',raw,pos)); pos+=12
        sections.append(records)
        if len(sections)>4096: raise ValueError('too many sections')
    out=bytearray(struct.pack('>I',4))
    for records in sections:
        out.extend(struct.pack('>I',len(records)))
        for record in records: out.extend(struct.pack('>III',*record))
    if bytes(out)!=raw: raise ValueError('projection mismatch')
    return sections,bytes(out)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('install',type=Path); p.add_argument('--census',type=inside_target,required=True)
    p.add_argument('--example',type=inside_target,required=True); p.add_argument('--example-sha256',type=expected_hash,required=True)
    p.add_argument('--out',type=inside_target,required=True); p.add_argument('--timeout',type=float,default=120)
    a=p.parse_args(); install=a.install.resolve(strict=True)
    if not 0<a.timeout<=300: p.error('timeout must be in (0,300]')
    if not a.example.is_file() or a.example.suffix.lower()!='.exe' or digest(a.example)!=a.example_sha256: raise ValueError('example pin mismatch')
    if not a.census.is_file() or a.census.stat().st_size>128*1024*1024 or digest(a.census)!=PIN: raise ValueError('census pin mismatch')
    data=a.census.read_bytes()
    if hashlib.sha256(data).hexdigest()!=PIN: raise ValueError('census changed while reading')
    rows=[]
    for line in data.decode('utf-8').splitlines():
        r=line.split('\t')
        if r[0]=='R':
            if len(r)!=12: raise ValueError('bad census row')
            if r[4]=='02D5C9AF': rows.append(r)
    if len(rows)!=2590: raise ValueError('unexpected corpus size')
    packages={r[1]:(install/r[1]).resolve(strict=True) for r in rows}
    if len(packages)!=4 or any(not x.is_relative_to(install) or not x.is_file() for x in packages.values()): raise ValueError('invalid package paths')
    before={k:digest(v) for k,v in packages.items()}
    a.out.mkdir(parents=True,exist_ok=False)
    report={'matched':False,'scope':'bounded disk parser projection; no native runtime or registration claim', 'census_sha256_before':PIN,'example_sha256_before':a.example_sha256,'package_sha256_before':before}
    try:
        indexes={}
        for name,path in packages.items():
            live=native_rows(path); indexes[name]={x[:3]:x for x in live}
            if len(indexes[name])!=len(live): raise ValueError('duplicate native key')
        batch=bytearray(b'HPS1'+struct.pack('<I',len(rows))); expected=bytearray(b'HPO1'+struct.pack('<I',len(rows)))
        counts=collections.Counter(); totals=collections.Counter()
        for r in rows:
            path=packages[r[1]]; offset,stored,decoded=map(int,r[6:9]); key=tuple(int(r[i],16) for i in (3,4,5))
            if min(offset,stored,decoded)<0 or max(stored,decoded)>MAX_CASE or offset+stored>path.stat().st_size: raise ValueError('invalid payload bounds')
            live=indexes[r[1]].get(key)
            if live is None or live[3:7]!=(offset,stored,decoded,0xffff) or int(r[9],16)!=0xffff: raise ValueError('live native index mismatch')
            with path.open('rb') as f: f.seek(offset); encoded=f.read(stored)
            if len(encoded)!=stored: raise ValueError('truncated payload')
            _,_,declared=_read_header(encoded,0)
            if declared!=decoded: raise ValueError('RefPack size mismatch')
            raw,_,_=decompress(encoded,out_size=decoded,header=True)
            if len(raw)!=decoded: raise ValueError('decoded size mismatch')
            sections,projection=project(raw); counts[len(sections)]+=1
            for i,section in enumerate(sections): totals[i]+=len(section)
            batch.extend(struct.pack('<I',len(raw))); batch.extend(raw)
            expected.extend(struct.pack('<II',len(sections),len(projection))); expected.extend(projection)
            if max(len(batch),len(expected))>MAX_BATCH: raise ValueError('batch limit exceeded')
        if digest(a.example)!=a.example_sha256: raise ValueError('example changed before run')
        result=subprocess.run([str(a.example)],input=bytes(batch),stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=str(TARGET),timeout=a.timeout,creationflags=subprocess.CREATE_NO_WINDOW,check=False)
        if result.returncode or result.stderr or result.stdout!=expected: raise ValueError('Rust output differs or failed')
        report.update(records=len(rows),section_count_distribution=dict(sorted(counts.items())),section_record_totals=dict(sorted(totals.items())),input_sha256=hashlib.sha256(batch).hexdigest(),projection_sha256=hashlib.sha256(expected).hexdigest(),rust_output_sha256=hashlib.sha256(result.stdout).hexdigest(),matched=True)
    except Exception as e:
        report['error']=f'{type(e).__name__}: {e}'
    finally:
        try:
            report['example_sha256_after']=digest(a.example); report['census_sha256_after']=digest(a.census)
            report['package_sha256_after']={k:digest(v) for k,v in packages.items()}
            if report['example_sha256_after']!=a.example_sha256 or report['census_sha256_after']!=PIN or report['package_sha256_after']!=before:
                report['matched']=False; report['error']='input changed during comparison'
        except OSError as e:
            report['matched']=False; report['error']=f'final hash failed: {e}'
        (a.out/'report.json').write_bytes((json.dumps(report,indent=2)+'\n').encode('utf-8'))
    if not report['matched']: raise ValueError('comparison failed; see report')
    print(json.dumps(report))
if __name__=='__main__': main()
