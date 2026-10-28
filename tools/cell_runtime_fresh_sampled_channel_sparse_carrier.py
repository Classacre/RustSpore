"""Sparse metadata rewrite for fullsource freshsampling. Never execute/build."""
import hashlib,struct
from pathlib import Path
import cell_runtime_fresh_loop_sparse_carrier as mechanical
import cell_runtime_fresh_sampled_channel_native_probe as probe
assert hashlib.sha256(Path(mechanical.__file__).read_bytes()).hexdigest()=='20dad0c2846995ffc62ee102fccf26698be4641f11903f1aa2049917470d974e'
mechanical.probe=probe;mechanical.BASE=probe.BASE;mechanical.OWNED_BASE=probe.OWNED_BASE
original_pages=mechanical.source_pages

def pages(images,symbols):
 result=original_pages(images,symbols);out=[]
 for a,raw,flags in result:
  raw=bytearray(raw)
  if a==0x13C0000:
   for off,value in[(0xC478,symbols['MemsetProvider']),(0xC47C,symbols['MemcpyProvider']),(0xC48C,0x785A35E0)]:struct.pack_into('<I',raw,off,value)
  out.append((a,bytes(raw),flags))
 return out
mechanical.source_pages=pages
if __name__=='__main__':
 mechanical.SOURCE_TOOL_SHA=mechanical.sha(probe.__file__)
 mechanical.main()
