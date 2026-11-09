"""Render pinned old/new decoded texture under identical explicit CPU controls."""
import argparse,hashlib,json
from pathlib import Path
from PIL import Image,ImageChops
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('inputs',type=Path);p.add_argument('out',type=Path);a=p.parse_args();root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 expected={'legacy.png':'60d35c9602dc229f2e4fccb58b8e48cb97ed79d1a3f25b6914460229766e135b','corrected.png':'91e53f5392afb27546bca5a3190f58906b4aa3fd3c18f218811296b1ee7ab474'}
 if any(sha(a.inputs/n)!=h for n,h in expected.items()):raise ValueError('input pin')
 renders=[]
 for name in expected:
  tex=Image.open(a.inputs/name);tex.load()
  if tex.size!=(128,128) or tex.mode!='RGBA':raise ValueError('input framing')
  frame=Image.new('RGBA',(768,768),(6,23,28,255));frame.alpha_composite(tex.resize((640,640),Image.Resampling.NEAREST),(64,64));renders.append(frame)
 diff=ImageChops.difference(renders[0],renders[1]);changed=sum(x!=(0,0,0,0) for x in diff.getdata());a.out.mkdir()
 for name,frame in zip(('legacy-render.png','corrected-render.png'),renders):
  with (a.out/name).open('xb') as f:frame.save(f,format='PNG')
 report={'scope':'offline nearest-neighbor texture inspection only; no mesh, Bevy, original shader, camera or game runtime','viewport':[768,768],'texture_rect':[64,64,704,704],'background_rgba':[6,23,28,255],'inputs':expected,'changed_frame_pixels':changed,'artifacts':{n:sha(a.out/n) for n in ('legacy-render.png','corrected-render.png')}}
 with (a.out/'receipt.json').open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()
