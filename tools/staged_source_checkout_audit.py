"""Verify that the complete index will check out as the measured local files."""
import argparse,hashlib,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args();out=args.out.resolve()
    out.relative_to((ROOT/'target').resolve());assert not out.exists()
    rows=[];failures=[]
    for entry in git('ls-files','--stage','-z').split(b'\0'):
        if not entry:continue
        meta,name=entry.split(b'\t',1);mode,blob,stage=meta.decode().split()
        assert stage=='0'and mode in('100644','100755'),(name,meta)
        name=name.decode();path=ROOT/name;assert path.is_file(),name
        before=path.read_bytes();restored=git('cat-file','--filters','--path='+name,blob)
        exact=before==restored
        if not exact:failures.append(name)
        rows.append(dict(path=name,blob=blob,working_sha256=hashlib.sha256(before).hexdigest(),checkout_sha256=hashlib.sha256(restored).hexdigest(),exact=exact))
    with out.open('x')as handle:json.dump(dict(schema='complete-index-checkout-hash-audit-v1',files=len(rows),exact=not failures,failures=failures,rows=rows,index_and_source_not_modified=True),handle,indent=2);handle.write('\n')
    print(json.dumps(dict(files=len(rows),exact=not failures,failures=failures)))
    if failures:raise SystemExit(1)
if __name__=='__main__':main()
