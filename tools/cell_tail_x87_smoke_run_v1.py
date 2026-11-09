"""Pinned, one-shot wrapper smoke runner. Review before execution."""
import argparse, hashlib, json, re, subprocess
from pathlib import Path
import pefile

ROOT=Path(__file__).resolve().parent.parent
HOST=ROOT/'target/cell-tail-x87-smoke-review-v2/host.exe'
DLL=ROOT/'target/cell-tail-x87-wrapper-link-review-v1/wrapper.dll'
HOST_RECEIPT=ROOT/'target/cell-tail-x87-smoke-review-v2/build-observation.json'
DLL_RECEIPT=ROOT/'target/cell-tail-x87-wrapper-link-review-v1/build-observation.json'
PINS={HOST:'cc73645170fd09e40c4153dc60d792316096184c8b44a52d70931f028e734a84',
 DLL:'9afd99d82133e7e0c973ef0597ddc0d8bf4aef84849af5a5a5fd9c2f9795d236',
 HOST_RECEIPT:'5cc87d86ea3a9be3aedeed0b5c6538474793d5d8f465d7e2866372eed1163c7c',
 DLL_RECEIPT:'c0470fb3eb712f8e26dae8febf53bce0f4f720a939841bc1c9d4a1f521edd289'}
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def verify():
    for path,digest in PINS.items():
        if not path.is_absolute() or sha(path)!=digest: raise ValueError('pin mismatch: '+str(path))
    for receipt in (HOST_RECEIPT,DLL_RECEIPT):
        doc=json.loads(receipt.read_text())
        for entry in doc['files']:
            path=Path(entry['path'])
            if not path.is_absolute() or sha(path)!=entry['sha256']: raise ValueError('receipt file mismatch: '+str(path))
    host=pefile.PE(str(HOST)); dll=pefile.PE(str(DLL))
    if host.FILE_HEADER.Machine!=0x14c or dll.FILE_HEADER.Machine!=0x14c: raise ValueError('not x86')
    if dll.OPTIONAL_HEADER.AddressOfEntryPoint!=0 or getattr(dll,'DIRECTORY_ENTRY_IMPORT',[]): raise ValueError('unexpected DLL entry/imports')
    if not getattr(dll,'DIRECTORY_ENTRY_BASERELOC',[]): raise ValueError('missing DLL relocations')
    exports=[e.name for e in dll.DIRECTORY_ENTRY_EXPORT.symbols]
    if exports!=[b'envelope']: raise ValueError('unexpected DLL exports')

def main():
    p=argparse.ArgumentParser(); p.add_argument('--out',type=Path,required=True); args=p.parse_args()
    out=args.out.resolve(); out.relative_to((ROOT/'target').resolve())
    # If unique-directory creation itself fails, no receipt can be created;
    # propagate that failure without touching an existing directory.
    out.mkdir(parents=True,exist_ok=False)
    receipt={'schema':'x87-wrapper-smoke-one-shot-v1','status':'in_progress',
             'host':str(HOST.resolve()),'dll':str(DLL.resolve()),'pins':{str(k):v for k,v in PINS.items()},
             'limits':'normal return and selected FXSAVE fields only; no AVX/fault safety; post-build receipts; stable-file admission; no sandbox; output size checked after capture'}
    def write(): (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    write()
    try:
        runner_hash=sha(Path(__file__).resolve())
        receipt['runner_sha256']=runner_hash
        verify()
        if sha(Path(__file__).resolve())!=runner_hash: raise ValueError('runner changed before launch')
        try:
            r=subprocess.run([str(HOST.resolve()),str(DLL.resolve())],capture_output=True,timeout=10)
            (out/'stdout.bin').write_bytes(r.stdout); (out/'stderr.bin').write_bytes(r.stderr)
            receipt['exit_code']=r.returncode
        except subprocess.TimeoutExpired as e:
            (out/'stdout.bin').write_bytes(e.stdout or b''); (out/'stderr.bin').write_bytes(e.stderr or b'')
            receipt['timeout']=True; raise
        if len(r.stdout)>4096 or len(r.stderr)>1024: raise ValueError('output exceeds protocol limits')
        if r.returncode!=0 or r.stderr: raise ValueError('host failure')
        lines=r.stdout.decode('ascii').splitlines()
        if len(lines)!=5 or lines[4]!='x87_wrapper_smoke=pass': raise ValueError('unexpected output lines')
        pattern=re.compile(r'image([0-3]) fcw=([0-9a-f]{4}) fsw=([0-9a-f]{4}) ftw=([0-9a-f]{2}) mxcsr=([0-9a-f]{8}) xmm=([0-9a-f]{256})')
        snapshots=[]
        for n,line in enumerate(lines[:4]):
            m=pattern.fullmatch(line)
            if not m or int(m[1])!=n: raise ValueError('invalid snapshot')
            snapshots.append(m.groups()[1:])
        if snapshots[1][:4]!=('027f','0000','00','00001f80'): raise ValueError('invalid initial fixture')
        if snapshots[0]!=snapshots[3]: raise ValueError('saved/restored field mismatch')
        if snapshots[2][0]!='027f' or snapshots[2][1]!='0000' or snapshots[2][2]!='00' or (int(snapshots[2][3],16)&~0x3f)!=0x1f80:
            raise ValueError('unexpected post-thunk control/status')
        verify()
        if sha(Path(__file__).resolve())!=runner_hash: raise ValueError('runner changed during run')
        receipt['stdout_sha256']=sha(out/'stdout.bin'); receipt['stderr_sha256']=sha(out/'stderr.bin'); receipt['status']='passed'
    except Exception as exc:
        receipt['status']='failed'; receipt['error']=repr(exc); write(); raise
    write(); print(json.dumps(receipt))
if __name__=='__main__': main()
