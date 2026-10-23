"""Find first desyncing prop in a group (optionally with min length) and hex-dump it."""
import struct, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress

REC=28; PREFIX=4; PROP_TYPE=0x00B1B104
FIXED={0x09:4,0x0a:4,0x0d:4,0x30:8,0x31:12,0x32:12,0x33:16,0x34:16,0x38:64,0x39:24}
SINGLE_PADDING={0x30:4,0x31:4,0x32:4,0x33:4,0x38:4,0x39:4}
ARRAY_SPECIFIERS=frozenset({0x009C,0x001C})
KNOWN_SPECS=frozenset({0x0000,0x001C,0x009C}|set(FIXED))
KNOWN_TYPES=frozenset(set(FIXED)|{0x01,0x20,0x00,0x12,0x13,0x22})
SEP_LOW_WORDS=(b"\x12\x00",b"\xa0\x00"); CONST_SEP=b"\x01\x00\x00\x00"
def esize(d,p,t,s):
    if s in ARRAY_SPECIFIERS:
        if p+8>len(d): return None
        n,st=struct.unpack_from(">II",d,p)
        if n>200000 or st>16384 or p+8+n*st>len(d): return None
        return 8+n*st
    if t in (0x12,0x13):
        if p+4>len(d): return None
        ln=struct.unpack_from(">I",d,p)[0]; w=1 if t==0x12 else 2
        if p+4+ln*w>len(d): return None
        return 4+ln*w
    if t==0x22:
        if p+4>len(d): return None
        n=struct.unpack_from(">I",d,p)[0]; pp=p+4
        for _ in range(n):
            if pp+12>len(d): return None
            sz,_,_=struct.unpack_from(">III",pp); pp+=12
            if pp+sz>len(d): return None
            pp+=sz
        return pp-p
    if t==0x01: ss=1
    elif t in FIXED: ss=FIXED[t]+(0 if s in ARRAY_SPECIFIERS else SINGLE_PADDING.get(t,0))
    elif t==0x20: ss=16
    elif t==0x00: ss=s
    else: ss=None
    if ss is None or p+ss>len(d): return None
    return ss
def is_sep(d,p):
    if d[p:p+4]==CONST_SEP: return True
    return d[p+2:p+4] in SEP_LOW_WORDS
def parse_index(path):
    with path.open("rb") as fh:
        head=fh.read(0x60); count=struct.unpack_from("<I",head,0x24)[0]
        idx_off=struct.unpack_from("<I",head,0x40)[0]; fh.seek(idx_off+PREFIX); blob=fh.read(count*REC)
    return [(g,off,sz&0x7FFFFFFF,dec) for (g,t,ihi,ilo,off,sz,dec) in [struct.unpack_from("<IIIIIII",blob,i*REC) for i in range(count)] if t==PROP_TYPE]

pkg=Path(sys.argv[1]); g0=int(sys.argv[2],16); minlen=int(sys.argv[3]) if len(sys.argv)>3 else 0
chosen=None; cd=None
for g,off,stored,dec in parse_index(pkg):
    if g!=g0: continue
    with pkg.open("rb") as fh:
        fh.seek(off); raw=fh.read(stored)
    d=decompress(raw,out_size=None,header=True)[0] if dec>stored else raw
    if len(d)<minlen: continue
    p=0; cnt,p=struct.unpack_from(">I",d,p)[0],p+4; ok=True; dp=0
    while p+8<=len(d):
        if is_sep(d,p): p+=4; continue
        ident,typ,spec=struct.unpack_from(">IHH",d,p); sz=esize(d,p+8,typ,spec)
        if sz is not None: p+=8+sz; continue
        typ2,spec2=struct.unpack_from(">HH",d,p); sz2=esize(d,p+4,typ2,spec2)
        if typ2 in KNOWN_TYPES and sz2 is not None and spec2 in KNOWN_SPECS: p+=4+sz2; continue
        ok=False; dp=p; break
    if is_sep(d,p): p+=4
    if not ok or p!=len(d):
        chosen=(g,off); cd=d; break
if chosen is None:
    print("no desyncing prop found"); raise SystemExit(0)
print(f"prop group=0x{g0:08x} len={len(cd)} desync@{dp}")
for i in range(0,len(cd),16):
    chunk=cd[i:i+16]
    mark="<<" if i<=dp<i+16 else "  "
    print(f"{mark} {i:04x}  "+" ".join(f"{b:02x}" for b in chunk)+"   "+"".join(chr(b) if 32<=b<127 else "." for b in chunk))
