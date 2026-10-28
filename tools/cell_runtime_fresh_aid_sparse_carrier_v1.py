"""Fresh-aid sparse carrier preparation/build only; never executes or compiles."""
import argparse,hashlib,json,re,struct
from pathlib import Path
import cell_runtime_fresh_aid_native_probe as source
from cell_schema_probe import SourceImage
from native_crt_acos_probe import PE
ROOT=Path(__file__).resolve().parent.parent
BASE=0x7C0000
OWNED_BASE=0x27000000
SOURCE_TOOL_SHA='c25839f813107ae3210bb1504e732a8062f43f1681dd98c8e79c63602b9bcc59'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def align(n,a):return(n+a-1)&~(a-1)
def save(p,x):
 with p.open('x') as f:json.dump(x,f,indent=2);f.write('\n')
def symbols(path):
 out={}
 for line in path.read_text().splitlines():
  m=re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]+\s+(\S+)\s+([0-9A-Fa-f]{8})\b',line)
  if m:out[m[1]]=int(m[2],16)
 return out

def host_source(image):
 assert sha(source.__file__)==SOURCE_TOOL_SHA
 lines=source.host_source(image).splitlines()
 for name,n in [('VirtualAlloc',16),('VirtualProtect',16),('FlushInstructionCache',12)]:lines.remove(f'EXTERN _{name}@{n}:PROC')
 lines.insert(lines.index('.const'),'EXTERN _GetModuleHandleA@4:PROC')
 start=lines.index('HostMain PROC')+2;stop=lines.index('MainLoop:')
 check=['push 0','call _GetModuleHandleA@4',f'cmp eax,0{BASE:08X}h','jne HostFailure']
 for a,b in source.CODE:check +=[f'mov esi,OFFSET Original_{a:08X}',f'mov edi,0{a:08X}h',f'mov ecx,{b-a}','repe cmpsb','jne HostFailure']
 check +=['cmp dword ptr [014861D0h],03F800000h','jne HostFailure','push -10','call _GetStdHandle@4','mov InputHandle,eax','push -11','call _GetStdHandle@4','mov OutputHandle,eax']
 lines[start:stop]=check
 return '\n'.join(lines)+'\n'

def source_pages(image):
 pages={0x7D0000:bytearray(b'\xcc'*65536),0x9A0000:bytearray(b'\xcc'*65536),0x1480000:bytearray(65536),0x1550000:bytearray(65536)}
 for a,b in source.CODE:pages[a&~65535][a&65535:(a&65535)+b-a]=image.read(a,b-a)
 pages[0x1480000][0x61d0:0x61d4]=image.read(0x14861d0,4)
 # Explicit owned readonly zero payload anchors first section after headers. Not original bytes.
 return [(0x7C1000,bytes(0xF000),0x40000040)]+[(a,bytes(b),0x60000020 if a in (0x7D0000,0x9A0000) else 0xC0000040 if a==0x1550000 else 0x40000040) for a,b in sorted(pages.items())]

def build(owned,mapping,image,audit):
 """Rewrite PE metadata only. Preserve absolute owned VAs and all native bytes."""
 pe=PE(owned);assert pe.base==OWNED_BASE
 assert {name for dll,name in pe.imports().values() if dll.lower()=='kernel32.dll'}=={'GetStdHandle','ReadFile','WriteFile','ExitProcess','GetModuleHandleA','GetLastError'}
 assert len(pe.imports())==6
 raw=bytearray(pe.data);nt=struct.unpack_from('<I',raw,0x3c)[0];opt=nt+24
 n=struct.unpack_from('<H',raw,nt+6)[0];sz=struct.unpack_from('<H',raw,nt+20)[0]
 assert struct.unpack_from('<H',raw,opt)[0]==0x10b and sz==224
 sa,fa=struct.unpack_from('<II',raw,opt+32);assert sa==4096 and fa==512
 sy=symbols(mapping);assert sy['HostMain']>=OWNED_BASE
 delta=OWNED_BASE-BASE
 sections=[]
 for i in range(n):
  h=opt+sz+i*40;name=bytes(raw[h:h+8]);vs,rva,rs,fo=struct.unpack_from('<IIII',raw,h+8);flags=struct.unpack_from('<I',raw,h+36)[0]
  assert not flags&0x2000000,'No discardable/relocation sections admitted'
  sections.append(dict(name=name,vs=vs,rva=rva+delta,data=bytearray(raw[fo:fo+rs]),flags=flags,old_rva=rva))
 def locate(rva,n):
  for sec in sections:
   if sec['old_rva']<=rva and rva+n<=sec['old_rva']+len(sec['data']):return sec['data'],rva-sec['old_rva']
  raise AssertionError(hex(rva))
 edits=[]
 def adjust(rva):
  data,at=locate(rva,4);v=struct.unpack_from('<I',data,at)[0]
  if v:struct.pack_into('<I',data,at,v+delta);edits.append(rva)
 # No unknown RVA-bearing structures silently retained.
 dirs=[struct.unpack_from('<II',raw,opt+96+8*i) for i in range(16)]
 for i,(rva,size) in enumerate(dirs):
  if i not in (1,5,6,12):assert (rva,size)==(0,0),(i,rva,size)
 imp,impsz=dirs[1];assert imp
 cursor=imp
 while True:
  d,o=locate(cursor,20);fields=struct.unpack_from('<5I',d,o)
  if not any(fields):break
  oft,stamp,chain,name,iat=fields;assert stamp==0 and chain==0 and oft and iat
  for table in (oft,iat):
   t=table
   while True:
    b,j=locate(t,4);v=struct.unpack_from('<I',b,j)[0]
    if not v:break
    assert not v&0x80000000,'Ordinal imports not admitted'
    adjust(t);t+=4
  for at in (cursor,cursor+12,cursor+16):adjust(at)
  cursor+=20
 native=source_pages(image)
 for i,(address,data,flags) in enumerate(native):
  assert all(address+len(data)<=BASE+s['rva'] or BASE+s['rva']+align(s['vs'],sa)<=address for s in sections)
  skip=4096 if address==BASE else 0
  # The first 4KiB belongs to PE headers, not any approved source span.
  if skip:assert all(not(BASE<=a<BASE+skip) for a,b in source.CODE)
  sections.append(dict(name=f'.n{i:02d}'.encode().ljust(8,b'\0'),vs=len(data)-skip,rva=address-BASE+skip,data=bytearray(data[skip:]),flags=flags))
 sections.sort(key=lambda s:s['rva'])
 # Microsoft PE Format, Section Table: section VAs must be ascending AND adjacent.
 # https://learn.microsoft.com/en-us/windows/win32/debug/pe-format#section-table-section-headers
 # Zero-raw virtual sections express holes without writable/executable payload.
 contiguous=[];gaps=[]
 for sec in sections:
  if contiguous:
   start=contiguous[-1]['rva']+align(contiguous[-1]['vs'],sa)
   assert start<=sec['rva']
   if start<sec['rva']:
    gap=dict(name=f'.g{len(gaps):02d}'.encode().ljust(8,b'\0'),vs=sec['rva']-start,rva=start,data=bytearray(),flags=0x80)
    contiguous.append(gap);gaps.append(dict(rva=f'{start:08X}',bytes=gap['vs'],flags='00000080',raw_bytes=0))
  contiguous.append(sec)
 sections=contiguous
 assert all(a['rva']+align(a['vs'],sa)==b['rva'] for a,b in zip(sections,sections[1:]))
 hdr=bytearray(raw[:opt+sz]);struct.pack_into('<H',hdr,nt+6,len(sections))
 # Strip symbols, relocation directory and debug metadata; fixed absolute image.
 struct.pack_into('<II',hdr,nt+12,0,0);flags=struct.unpack_from('<H',hdr,nt+22)[0]|1;struct.pack_into('<H',hdr,nt+22,flags)
 struct.pack_into('<I',hdr,opt+28,BASE);struct.pack_into('<I',hdr,opt+16,sy['HostMain']-BASE)
 for at in (20,24):
  v=struct.unpack_from('<I',hdr,opt+at)[0]
  if v:struct.pack_into('<I',hdr,opt+at,v+delta)
 struct.pack_into('<H',hdr,opt+70,struct.unpack_from('<H',hdr,opt+70)[0]&~0x40)
 for i,(rva,size) in enumerate(dirs):
  struct.pack_into('<II',hdr,opt+96+i*8,*( (rva+delta,size) if i in (1,12) and rva else (0,0)))
 headers=align(len(hdr)+40*len(sections),fa);assert headers<=4096;body=bytearray();table=bytearray()
 for sec in sections:
  blob=sec['data'];rs=align(len(blob),fa);pos=headers+len(body) if rs else 0
  table+=struct.pack('<8sIIIIIIHHI',sec['name'],sec['vs'],sec['rva'],rs,pos,0,0,0,0,sec['flags'])
  body+=blob+bytes(rs-len(blob))
 struct.pack_into('<I',hdr,opt+56,align(max(s['rva']+s['vs'] for s in sections),sa))
 assert sections[0]['rva']==align(headers,sa)
 struct.pack_into('<I',hdr,opt+60,headers);struct.pack_into('<I',hdr,opt+64,0)
 for at,mask in ((4,0x20),(8,0x40),(12,0x80)):
  struct.pack_into('<I',hdr,opt+at,sum(align(sec['vs'],sa) if mask==0x80 else align(len(sec['data']),fa) for sec in sections if sec['flags']&mask))
 result=hdr+table+bytes(headers-len(hdr)-len(table))+body
 # Every owned instruction byte survives; edits must remain metadata only.
 first,last=sy['OwnedStart'],sy['OwnedEnd']
 assert all(not(first-OWNED_BASE<=at<last-OWNED_BASE) for at in edits)
 for sec in sections:
  if 'old_rva' not in sec:continue
  expected=bytearray(pe.read(OWNED_BASE+sec['old_rva'],len(sec['data'])))
  for at in edits:
   if sec['old_rva']<=at<sec['old_rva']+len(expected):
    j=at-sec['old_rva'];struct.pack_into('<I',expected,j,struct.unpack_from('<I',expected,j)[0]+delta)
  assert expected==sec['data']
 return bytes(result),dict(owned_exe_sha256=sha(owned),owned_map_sha256=sha(mapping),metadata_rva_edits=edits,owned_absolute_addresses_preserved=True,source_pages=[dict(address=f'{a:08X}',sha256=hashlib.sha256(b).hexdigest(),flags=f'{f:08X}') for a,b,f in native],sections=len(sections),virtual_gap_sections=gaps,section_virtual_ranges_adjacent=True,pe_spec='https://learn.microsoft.com/en-us/windows/win32/debug/pe-format#section-table-section-headers',size_of_image=struct.unpack_from('<I',hdr,opt+56)[0],imports=sorted(pe.imports().values()),entry=f'{sy["HostMain"]:08X}',global_serial='015514E0')

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--owned-exe',type=Path);p.add_argument('--owned-map',type=Path);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());image=SourceImage(a.source_exe);proof=source.audit_source(image)
 if a.owned_exe:
  assert a.owned_map and (a.out/'host-x86.asm').read_text()==host_source(image)
  raw,report=build(a.owned_exe,a.owned_map,image,proof)
  with (a.out/'carrier-x86.exe').open('xb') as f:f.write(raw)
  report.update(native_executed=False,independent_linked_audit_required=True,carrier_sha256=sha(a.out/'carrier-x86.exe'));save(a.out/'carrier-build-review.json',report);return
 a.out.mkdir(parents=True,exist_ok=False);(a.out/'host-x86.asm').write_text(host_source(image));save(a.out/'manifest.json',dict(schema='fresh-aid-sparse-carrier-preparation-v1',tool_sha256=sha(__file__),source_tool_sha256=SOURCE_TOOL_SHA,asm_sha256=sha(a.out/'host-x86.asm'),source_audit=proof,source_bytes=1049,primary_base='007C0000',owned_compile_base='27000000',native_executed=False,compiled=False,owned_padding='007C1000..007CFFFF readonly zero payload; no original source',serial_policy='01550000..0155FFFF owned mutable zero initial page; per-record full shadow except source serial DWORD retained',limits='Old exit10 is preinput mapping failure only. Actual linked independent audit required. Existing input/model/RunOnce unchanged.'))
if __name__=='__main__':main()
