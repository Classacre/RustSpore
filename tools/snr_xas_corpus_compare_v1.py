#!/usr/bin/env python3
"""Compare the strict XAS1 RAM subset with vgmstream; writes only to target.

Usage from repository root:
  python tools/snr_xas_corpus_compare_v1.py PACKAGE VGMSTREAM_CLI NATIVE_VERIFY NEW_TARGET_DIR [MAX_ADMITTED]

The native verifier is built with tools/build.bat build -p spore-core --example
snr_xas_verify. The vgmstream CLI is an independently obtained external tool;
this comparison checks a derived port, not independent algorithm fidelity.
"""
import sys,struct,collections,json,hashlib,subprocess,wave
from pathlib import Path
sys.path.insert(0,'tools'); from dbpf_extract import parse_index
from refpack import decompress
if len(sys.argv) not in (5, 6): raise SystemExit(__doc__)
limit = int(sys.argv[5]) if len(sys.argv) == 6 else None
if limit is not None and limit <= 0: raise SystemExit('MAX_ADMITTED must be positive')
p=Path(sys.argv[1]).resolve(strict=True)
vgm=Path(sys.argv[2]).resolve(strict=True)
native=Path(sys.argv[3]).resolve(strict=True)
target=Path('target').resolve(strict=True)
requested=Path(sys.argv[4])
parent=requested.parent.resolve(strict=True)
if not parent.is_relative_to(target): raise SystemExit('output must be under target')
out=parent/requested.name
out.mkdir(exist_ok=False)
def digest(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for chunk in iter(lambda:f.read(1024*1024),b''): h.update(chunk)
 return h.hexdigest()
before=digest(p)
entries,_=parse_index(p)
log=(out/'results.jsonl').open('x',encoding='utf8')
counts=collections.Counter(); failures=[]; mismatches=0
with p.open('rb') as f:
 for e in entries:
  if e['type']!=0x01a527db:continue
  f.seek(e['offset']);raw=f.read(e['stored'])
  try:
   d=decompress(raw,out_size=None,header=True)[0] if e['dec']>e['stored'] else raw
   if len(d)<8: counts['short']+=1;continue
   h,n=struct.unpack_from('>II',d);codec=h>>24&15;ch=(h>>18&63)+1;rate=h&0x3ffff;samples=n&0x1fffffff
   if h>>28!=0 or codec!=4: counts[f'other version/codec {h>>28}/{codec}']+=1;continue
   if ch>2 or rate==0 or rate>200000 or not samples: counts['invalid channels/rate/empty']+=1;continue
   if n>>29: counts['stream or loop']+=1;continue
   if len(d)<16: counts['short block']+=1;continue
   block,bs=struct.unpack_from('>II',d,8);size=block&0xffffff;frames=(samples+127)//128
   if block>>24 not in (0,128) or 8+size!=len(d) or bs!=samples or size<8 or frames*ch*76!=size-8: counts['other block framing']+=1;continue
   predictors={struct.unpack_from('<I',d,16+frame*ch*76+c*76+g*4)[0]&15 for frame in range(frames) for c in range(ch) for g in range(4)}
   if max(predictors, default=0)>3: counts['unsupported predictor']+=1;continue
   counts['admissible']+=1
   snr=out/'current.snr'; wav=out/'current.wav'; pcm=out/'current.pcm'
   for path in (snr,wav,pcm): path.unlink(missing_ok=True)
   snr.write_bytes(d)
   a=subprocess.run([str(vgm),'-i','-o',str(wav),str(snr)],capture_output=True,text=True,timeout=120)
   b=subprocess.run([str(native),str(snr),str(pcm)],capture_output=True,text=True,timeout=120)
   result={'instance':f"{e['instance']:016X}",'source_sha256':hashlib.sha256(d).hexdigest(),'rate':rate,'channels':ch,'samples':samples,'vgm_exit':a.returncode,'native_exit':b.returncode}
   try:
    if a.returncode or b.returncode: raise ValueError('decode failure '+a.stderr+' '+b.stderr)
    with wave.open(str(wav),'rb') as w:
     if (w.getnchannels(),w.getframerate(),w.getsampwidth(),w.getnframes()) != (ch,rate,2,samples): raise ValueError('reference WAV format mismatch')
     ref=w.readframes(samples)
    actual=pcm.read_bytes();result.update(equal=actual==ref,native_sha256=hashlib.sha256(actual).hexdigest(),reference_sha256=hashlib.sha256(ref).hexdigest())
   except Exception as ex: result['error']=str(ex)
   if not result.get('equal', False): mismatches += 1
   log.write(json.dumps(result)+'\n');log.flush()
   for path in (snr,wav,pcm): path.unlink(missing_ok=True)
   if counts['admissible'] % 100 == 0: print('checked',counts['admissible'],flush=True)
   if limit is not None and counts['admissible'] >= limit: break
  except Exception as ex:
   failures.append((e['instance'],str(ex)))
   log.write(json.dumps({'instance':f"{e['instance']:016X}",'error':str(ex)})+'\n');log.flush()
print('counts',dict(counts),'failures',failures)
after=digest(p)
(out/'summary.json').write_text(json.dumps({'counts':dict(counts),'resource_failures':failures,'package_sha256_before':before,'package_sha256_after':after,'vgmstream_sha256':digest(vgm),'native_verifier_sha256':digest(native),'mismatches':mismatches,'limit':limit},indent=2),encoding='utf8')
log.close()
if failures or mismatches or before != after: raise SystemExit(1)
