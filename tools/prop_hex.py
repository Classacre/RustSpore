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

pkg=Path(sys.argv[1]); g0=int(sys.argv[2],16); start=int(sys.argv[3]); end=int(sys.argv[4])
for g,off,stored,dec in parse_index(pkg):
    if g!=g0: continue
    with pkg.open("rb") as fh:
        fh.seek(off); raw=fh.read(stored)
    d=decompress(raw,out_size=None,header=True)[0] if dec>stored else raw
    if len(d)<end: continue
    for i in range(start,end,16):
        chunk=d[i:i+16]
        print(f"{i:04x}  "+" ".join(f"{b:02x}" for b in chunk)+"   "+"".join(chr(b) if 32<=b<127 else "." for b in chunk))
    break
