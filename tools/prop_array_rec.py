"""Test whether 0x0013 array element is a single string or a 2-string record."""
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
pkg=Path("D:/Games/SPORE Collection/Data/Spore_Game.package")
d=None
for g,off,stored,dec in parse_index(pkg):
    if g!=0x0001ffff: continue
    with pkg.open("rb") as fh:
        fh.seek(off); raw=fh.read(stored)
    dd=decompress(raw,out_size=None,header=True)[0] if dec>stored else raw
    if len(dd)==1044:
        d=dd; break
assert d is not None
# entry 1 header at 0xa4: id, type 0x0013, spec 0x009c, then count, stride
start=0xa4+8  # value start
cnt,stride=struct.unpack_from(">II",d,start)
print(f"count={cnt} stride={stride} value@0x{start+8:x}")
base=start+8
def strings_after(p,n,per):
    out=[]; ok=True
    for _ in range(n):
        for _2 in range(per):
            if p+4>len(d): ok=False; break
            ln=struct.unpack_from(">I",d,p)[0]
            if ln>200 or p+4+ln*2>len(d): ok=False; break
            s=d[p+4:p+4+ln*2].decode("utf-16-le","replace"); p+=4+ln*2
            out.append(s)
        if not ok: break
    return p,ok,out
for per,label in ((1,"single-string"),(2,"2-string record")):
    p,ok,out=strings_after(base,cnt,per)
    remaining=len(d)-p
    print(f"\n[{label}] consumed to 0x{p:x}, remaining={remaining}, ok={ok}")
    print("  sample strings:", out[:8])
    # does the landing point look like a valid entry header?
    if p+8<=len(d):
        nid,nty,nsp=struct.unpack_from(">IHH",d,p)
        print(f"  FOLLOWING entry: id=0x{nid:08x} type=0x{nty:04x} spec=0x{nsp:04x}")
