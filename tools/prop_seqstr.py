import struct, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress
REC=28;PREFIX=4;PROP_TYPE=0x00B1B104
def parse_index(path):
    with path.open('rb') as fh:
        head=fh.read(0x60);count=struct.unpack_from('<I',head,0x24)[0]
        idx_off=struct.unpack_from('<I',head,0x40)[0];fh.seek(idx_off+PREFIX);blob=fh.read(count*REC)
    return [(g,off,sz&0x7FFFFFFF,dec) for (g,t,ihi,ilo,off,sz,dec) in [struct.unpack_from('<IIIIIII',blob,i*REC) for i in range(count)] if t==PROP_TYPE]
pkg=Path('D:/Games/SPORE Collection/Data/Spore_Game.package')
for g,off,stored,dec in parse_index(pkg):
    if g!=0x0001ffff: continue
    with pkg.open('rb') as fh: fh.seek(off);raw=fh.read(stored)
    d=decompress(raw,out_size=None,header=True)[0] if dec>stored else raw
    if len(d)!=1044: continue
    p=0xb4
    n=0
    while p+4<=len(d):
        ln=struct.unpack_from('>I',d,p)[0]
        if ln>200 or p+4+ln*2>len(d):
            print(f'BREAK at 0x{p:x} ln={ln} bytes_left={len(d)-p}'); break
        s=d[p+4:p+4+ln*2].decode('utf-16-le','replace')
        print(f'  str[{n}] len={ln} @{p:#x}: {s!r}')
        p+=4+ln*2; n+=1
    print(f'total strings={n}, end=0x{p:x}, prop_end=0x{len(d):x}')
    break
