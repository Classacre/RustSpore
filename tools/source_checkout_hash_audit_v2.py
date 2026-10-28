"""Audit exact Git checkout bytes for every publishable file, including plans.

Writes only unreachable content-addressed Git blobs and an ignored report.
Does not stage, modify source, change configuration or contact the remote.
"""
import argparse,hashlib,json
from collections import Counter
from pathlib import Path
from source_checkout_hash_audit import ROOT,git

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args();output=args.out.resolve()
    output.relative_to((ROOT/'target').resolve());assert not output.exists()
    names=set(git('ls-files','--cached','--others','--exclude-standard','-z').decode().split('\0'))
    rows=[];counts=Counter();failures=[]
    for name in sorted(names):
        path=ROOT/name
        if not path.is_file():continue
        raw=path.read_bytes();lf=raw.count(b'\n');crlf=raw.count(b'\r\n')
        style='mixed'if 0<crlf<lf else 'crlf'if crlf else 'lf'
        blob=git('hash-object','-w','--stdin','--path='+name,data=raw).decode().strip()
        restored=git('cat-file','--filters','--path='+name,blob)
        exact=raw==restored
        if not exact:failures.append(name)
        counts[style]+=1
        rows.append(dict(path=name,style=style,working_sha256=hashlib.sha256(raw).hexdigest(),git_blob=blob,checkout_sha256=hashlib.sha256(restored).hexdigest(),exact=exact))
    output.parent.mkdir(parents=True,exist_ok=True)
    with output.open('x')as report:json.dump(dict(schema='all-publishable-files-git-checkout-audit-v2',files=len(rows),exact=not failures,failures=failures,styles=dict(counts),rows=rows,index_HEAD_source_config_remote_unchanged=True),report,indent=2);report.write('\n')
    print(json.dumps(dict(files=len(rows),exact=not failures,failures=failures,styles=dict(counts))))
    if failures:raise SystemExit(1)
if __name__=='__main__':main()
