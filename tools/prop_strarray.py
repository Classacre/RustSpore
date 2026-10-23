"""Brute-force the string-array layout in the desyncing 0x0001ffff prop."""
import struct, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress

REC=28; PREFIX=4; PROP_TYPE=0x00B1B104
def parse_index(path):
    with path.open("rb") as fh:
        head=fh.read(0x60); count=struct.unpack_from("<I",head,0x24)[0]
        idx_off=struct.unpack_from("<I",head,0x40)[0]; fh.seek(idx_off+PREFIX); blob=fh.read(count*REC)
    return [(g,off,sz&0x7FFFFFFF,dec) for (g,t,ihi,ilo,off,sz,dec) in [struct.unpack_from("<IIIIIII",blob,i*REC) for i in range(count)] if t==PROP_TYPE]

pkg=Path("D:/Games/SPORE Collection/Data/PatchData.package")
target=1665
d=None
for g,off,stored,dec in parse_index(pkg):
    if g!=0x0001ffff: continue
    with pkg.open("rb") as fh:
        fh.seek(off); raw=fh.read(stored)
    dd=decompress(raw,out_size=None,header=True)[0] if dec>stored else raw
    if len(dd)==target:
        d=dd; break
assert d is not None

start=0x19c  # type 0x0013, spec 0x009c
typ,spec=struct.unpack_from(">HH",d,start)
cnt,stride=struct.unpack_from(">II",d,start+4)
print(f"at 0x{start:04x}: type=0x{typ:04x} spec=0x{spec:04x} count={cnt} stride={stride}")

def decode_region(p, n, stride, mode):
    out=[]
    for i in range(n):
        if mode=="fixed":
            seg=d[p:p+stride]
            # find length prefix (first 4 bytes BE) then string
            ln=struct.unpack_from(">I",seg,0)[0]
            w=2
            s=seg[4:4+ln*w].decode("utf-16-le","replace")
            out.append((ln,s,seg.hex()))
            p+=stride
        else:  # length-prefixed
            if p+4>len(d): out.append(("OVER",None,d[p:].hex())); break
            ln=struct.unpack_from(">I",d,p)[0]
            if ln>1000 or p+4+ln*2>len(d): out.append(("BADLEN",ln,d[p:p+20].hex())); break
            s=d[p+4:p+4+ln*2].decode("utf-16-le","replace")
            out.append((ln,s))
            p+=4+ln*2
    return p,out

for mode in ("fixed","lenprefix"):
    p=start+12
    end,out=decode_region(p,cnt,stride,mode)
    print(f"\n=== mode={mode} ends at 0x{end:04x} ({(len(d)-end)} bytes remain) ===")
    for i,(ln,s,*rest) in enumerate(out[:cnt]):
        print(f"  [{i}] len={ln} str={s!r}")
    # what follows
    if end+8<=len(d):
        nid,nty,nsp=struct.unpack_from(">IHH",d,end)
        print(f"  FOLLOWING: id=0x{nid:08x} type=0x{nty:04x} spec=0x{nsp:04x}")
