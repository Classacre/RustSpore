"""Check explicit 3D baseline stability and controlled UI-region changes."""
import argparse
import hashlib
import json
import math
from pathlib import Path
from PIL import Image


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for arg in ('before_a','before_b','after','overlay','fixture'): p.add_argument(arg,type=Path)
    p.add_argument('--out',type=Path,required=True); args=p.parse_args()
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    if args.out.exists(): raise FileExistsError(args.out)
    paths=[args.before_a,args.before_b,args.after,args.overlay]
    raw=[path.read_bytes() for path in paths]; imgs=[Image.open(path).convert('RGBA') for path in paths]
    if any(img.size!=(768,768) for img in imgs): raise ValueError('unexpected capture dimensions')
    pixels=[list(img.getdata()) for img in imgs]
    if not (pixels[0]==pixels[1]==pixels[2]): raise ValueError('no-UI baseline changed')
    report=json.loads(args.fixture.read_text(encoding='utf-8'))
    calls=report['calls']
    if not calls or len(calls)!=report['call_count']: raise ValueError('bad fixture calls')
    # Conservative per-call pixel bounding boxes with 1px rasterization margin.
    boxes=[]
    for call in calls:
        l,t,r,b=call['destination']
        if not all(map(math.isfinite,(l,t,r,b))) or l>r or t>b: raise ValueError('bad rectangle')
        boxes.append((max(0,math.floor(l)-1),max(0,math.floor(t)-1),min(768,math.ceil(r)+1),min(768,math.ceil(b)+1)))
    inside=outside=0
    for n,(base,overlay) in enumerate(zip(pixels[2],pixels[3])):
        if base==overlay: continue
        x=n%768; y=n//768
        if any(l<=x<r and t<=y<b for l,t,r,b in boxes): inside+=1
        else: outside+=1
    if inside==0 or outside: raise ValueError(f'UI pixel boundary failed: inside={inside}, outside={outside}')
    background=(0x18,0x20,0x2e,0xff)
    nonbackground=sum(p!=background for p in pixels[2])
    if nonbackground<1000: raise ValueError('3D baseline lacks expected nonbackground content')
    data={'scope':'single-frame explicit source UI fixture; synthetic geometry/state; no retail HUD claim',
          'size':[768,768], 'pre_and_post_baselines_rgba_equal':True,
          'png_sha256':[hashlib.sha256(x).hexdigest() for x in raw],
          'fixture_sha256':hashlib.sha256(args.fixture.read_bytes()).hexdigest(),
          'baseline_nonbackground_pixels':nonbackground,'inside_changed':inside,'outside_changed':outside,
          'rectangle_union_boxes':boxes,'raster_margin_pixels':1}
    args.out.parent.mkdir(parents=True,exist_ok=True)
    with args.out.open('x',encoding='utf-8') as out: json.dump(data,out,indent=2);out.write('\n')
    print(json.dumps(data,indent=2))


if __name__=='__main__': main()
