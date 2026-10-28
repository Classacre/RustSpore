"""Derived bounded quadratic/line/gray algorithm development check.
Original x86 census remains independent oracle. No native execution.
"""
import hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent

def flatten(points,ends,on):
 lines=[];curves=[];start=0
 def quad(a,b,c):
  error=max(abs((a[k]-2*b[k]+c[k])>>2) for k in [0,1]);depth=0
  while error>4:error>>=2;depth+=1
  assert depth<=8
  curves.append(dict(a=a,b=b,c=c,depth=depth))
  def split(a,b,c,n):
   if n==0:lines.append((a,c));return
   mid=tuple((a[k]+2*b[k]+c[k]+2)>>2 for k in [0,1]);left=tuple((a[k]+b[k]+1)>>1 for k in [0,1]);right=tuple((b[k]+c[k]+1)>>1 for k in [0,1]);split(a,left,mid,n-1);split(mid,right,c,n-1)
  split(a,b,c,depth)
 for end in ends:
  assert on[start] and all(v%64!=32 for v in points[start]);current=tuple(points[start]);i=start+1
  while True:
   if i>end:i=start
   p=tuple(points[i])
   if on[i]:
    lines.append((current,p));current=p
    if i==start:break
    i+=1
   else:
    j=i+1 if i<end else start;q=tuple(points[j])
    if on[j]:target=q;nexti=j+1
    else:target=tuple((p[k]+q[k]+1)>>1 for k in [0,1]);nexti=j
    quad(current,p,target);current=target
    if j==start and on[j]:break
    i=nexti
  start=end+1
 return lines,curves

def trunc(a,b):return (abs(a)//b)*(-1 if a<0 else 1)
def crossings(points,ends,on):
 horizontal={s:[] for s in range((min(p[1] for p in points)-64)>>6,((max(p[1] for p in points)+64)>>6)+1)};vertical={s:[] for s in range((min(p[0] for p in points)-64)>>6,((max(p[0] for p in points)+64)>>6)+1)};lines,curves=flatten(points,ends,on);previous=[None,None]
 for a,b in lines:
  if a[0]!=b[0]:
   direction=int(b[0]>a[0]);lo,hi=(a,b) if direction else (b,a);low,high=lo[0],hi[0];sample=((low+31)&~63)+32
   if a[0]%64==32:
    assert previous[0] is not None
    if direction and previous[0]<a[0]:sample+=64
    if not direction and previous[0]>a[0]:high=a[0]-1
   while sample<=high:
    value=lo[1]+trunc((sample-low)*(hi[1]-lo[1]),hi[0]-lo[0]);vertical[sample>>6].append(value*2|direction);sample+=64
  if a[1]!=b[1]:
   direction=int(b[1]>a[1]);lo,hi=(a,b) if direction else (b,a);low,high=lo[1],hi[1];sample=((low+31)&~63)+32
   if a[1]%64==32:
    assert previous[1] is not None
    if direction and previous[1]<a[1]:sample+=64
    if not direction and previous[1]>a[1]:high=a[1]-1
   while sample<=high:
    den=hi[1]-lo[1];num=(sample-low)*(hi[0]-lo[0]);value=lo[0]+((num+den-1)//den if direction else num//den);horizontal[sample>>6].append(value*2|direction);sample+=64
  elif a[1]%64==32 and a[0]!=b[0]:horizontal[a[1]>>6].extend([min(a[0],b[0])*2|1,max(a[0],b[0])*2])
  for axis in [0,1]:
   if a[axis]!=b[axis]:previous[axis]=a[axis]
 for rows in [horizontal,vertical]:
  for values in rows.values():values.sort()
 return dict(horizontal=[dict(row=s,events=v) for s,v in horizontal.items()],vertical=[dict(row=s,events=v) for s,v in vertical.items()]),lines,curves

def intervals(events):
 result=[];i=0
 while i<len(events):
  j=i+1;assert j<len(events);w=(1 if events[i]&1 else -1)+(1 if events[j]&1 else -1)
  while w:
   j+=1;assert j<len(events);w+=1 if events[j]&1 else -1
  result.append((events[i]>>1,events[j]>>1));i=j+1
 return result

def bitmap(cross):
 h=[r for r in cross['horizontal'] if r['events']];v=[r for r in cross['vertical'] if r['events']];assert h and v
 xmin=min([r['events'][0]>>1 for r in h]+[v[0]['row']*64+32]);xmax=max([r['events'][-1]>>1 for r in h]+[v[-1]['row']*64+32]);ymin=min([h[0]['row']*64+32]+[r['events'][0]>>1 for r in v]);ymax=max([h[-1]['row']*64+32]+[r['events'][-1]>>1 for r in v]);left=xmin;top=ymax+64;w=((xmax+64)>>6)-(xmin>>6);height=((ymax+64)>>6)-(ymin>>6);assert 0<w<=64 and 0<height<=64;alpha=[0]*(w*height)
 def at(x,y):
  col=x-(left>>6);row=(top>>6)-y-1;assert 0<=col<w and 0<=row<height,(x,y,col,row,w,height);return row*w+col
 def blend(a,b):
  assert 0<=a<=126 and 0<=b<=126,(a,b);wa=3970-(63-a)**2;wb=3970-(63-b)**2;return(a*wa+b*wb)//(wa+wb)
 for r in h:
  for a,b in intervals(r['events']):
   first,last=a>>6,b>>6;i=at(first,r['row'])
   if first==last:alpha[i]=(alpha[i]+2*((b&63)-(a&63)))&255
   else:
    alpha[i]=(alpha[i]+2*(63-(a&63)))&255
    for x in range(first+1,last):alpha[at(x,r['row'])]=126
    alpha[at(last,r['row'])]=2*(b&63)
 for r in v:
  pending=None
  for a,b in intervals(r['events']):
   first,last=a>>6,b>>6;i=at(r['row'],first);carry=0
   if pending:
    if pending[0]==i:carry=pending[1]
    else:alpha[pending[0]]=blend(alpha[pending[0]],pending[1])
   if first==last:pending=(i,(carry+2*((b&63)-(a&63)))&255)
   else:
    alpha[i]=blend(alpha[i],(carry+2*(63-(a&63)))&255);pending=(at(r['row'],last),2*(b&63))
  if pending:alpha[pending[0]]=blend(alpha[pending[0]],pending[1])
 return dict(width=w,height=height,pitch=w,left=left,top=top,raw_bounds=[xmin,xmax,ymin,ymax],alpha_hex=bytes(alpha).hex())

def main(out):
 out=Path(out);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False);rows=[]
 for gid in [37,49,54]:
  hp=ROOT/f'target/source-ui/font-simple-census-v3/gid{gid}-hint.json';rp=ROOT/f'target/source-ui/font-simple-raster-census-v1/gid{gid}-raster.json';h=json.loads(hp.read_text());r=json.loads(rp.read_text());g=h['glyph'];n=g['real_points'];data=bytes.fromhex(g['data_hex']);on=[bool(v&1) for v in data[(n+4)*20:(n+4)*21]][:n];cross,lines,curves=crossings(g['output_points'][:n],g['contour_ends'],on);b=bitmap(cross);differences=[]
  for kind in ['horizontal','vertical']:
   for actual,expected in zip(cross[kind],r['crossings'][kind]):
    if actual!=expected:differences.append(dict(kind=kind,actual=actual,expected=expected))
  report=dict(gid=gid,crossings_exact=cross==r['crossings'],alpha_exact=b['alpha_hex']==r['alpha_hex'],bitmap=b,lines=lines,curves=curves,differences=differences,source_hint_sha256=hashlib.sha256(hp.read_bytes()).hexdigest(),source_raster_sha256=hashlib.sha256(rp.read_bytes()).hexdigest());(out/f'gid{gid}.json').write_text(json.dumps(report,indent=2)+'\n');rows.append({k:report[k] for k in ['gid','crossings_exact','alpha_exact']});print(rows[-1],len(differences),len(lines),len(curves),flush=True)
 (out/'manifest.json').write_text(json.dumps(dict(schema='curved-derived-algorithm-check-v1',rows=rows,new_native_execution=False,rust_execution=False),indent=2)+'\n')
if __name__=='__main__':main(sys.argv[1])
