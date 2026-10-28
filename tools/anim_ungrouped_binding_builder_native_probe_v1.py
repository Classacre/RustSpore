"""Prepare guarded full9A1E00 entry, exact217 ungrouped source-domain. No builds/runs."""
import argparse,json,struct,hashlib
from pathlib import Path
import anim_nonroot_selector_z_native_probe_v1 as combined
import anim_nonroot_selector_native_probe_v2 as selector
import anim_root_pose_sparse_carrier_v3 as carrier
from cell_schema_probe import SourceImage
from cell_native_math_probe import db
ROOT=Path(__file__).resolve().parent.parent
BASE=0x670000;CAPACITY=7140;MODEL=114284;REPLY=MODEL+8
SIZES=dict(combined.SIZES,Payload=94492,Reply=REPLY,Clip=348,Channels=14*228,Descriptors=14*24*32,ChannelPtrs=56,Output=40,Records=CAPACITY*16)
TRAPS={0x67c860:'secondarytopologyerrorcount0',0x99c5e0:'secondarytopologycount0',0x99ed80:'requiredmask0andheadergate0success',0x9a0980:'7140capacity',0x9a1980:'AF0',0x9a1b80:'AF0',0xf47290:'allgroupstatepointerszero',0xf634b0:'7140capacityandresize0',0x11e0a70:'secondarytopologycount0',0x9b03c0:'hierarchy0or1000',0x9b2ef0:'hierarchy0or1000'}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def memset_import(image):
 nt=struct.unpack_from('<I',image.data,60)[0];op=nt+24;base=struct.unpack_from('<I',image.data,op+28)[0];rva,size=struct.unpack_from('<II',image.data,op+104)
 def string(a):
  b=bytearray()
  for k in range(256):
   c=image.read(a+k,1)[0]
   if not c:return b.decode('ascii')
   b.append(c)
  raise AssertionError('unboundedimportname')
 for i in range(size//20):
  oft,stamp,chain,name,ft=struct.unpack('<5I',image.read(base+rva+i*20,20))
  if not name:break
  for j in range(2048):
   value=struct.unpack('<I',image.read(base+(oft or ft)+j*4,4))[0]
   if not value:break
   if base+ft+j*4!=0x13cc478:continue
   assert not value&0x80000000;dll,symbol=string(base+name),string(base+value+2);assert(dll,symbol)==('MSVCR90.dll','memset');return dict(IAT='013CC478',dll=dll,symbol=symbol,targeted_original_import_verified=True)
 raise AssertionError('memsetIATnotfound')

def evidence(n):
 p=ROOT/f'target/rea/20261027-source-runtime-v4/{n}-analyze_function.json';e=json.loads(json.loads(p.read_text())['content'][0]['text']);return p,e,e['raw_result']['value']
def proof(image,out):
 p=combined.proof(image,out)
 def add(entry,body,origin):
  spans=[]
  for r in body['ranges']:
   a,b=int(r['start'],16),int(r['end'],16)+1;raw=image.read(a,b-a);name=f'source-{a:08X}-{b:08X}.bin';(out/name).write_bytes(raw);spans.append(dict(start=f'{a:08X}',end_exclusive=f'{b:08X}',bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest(),blob=name))
  assert sum(x['bytes']for x in spans)==body['total_bytes'];p['functions'].append(dict(entry=f'{entry:08X}',ranges=spans,origin=origin));p['source_bytes']+=body['total_bytes']
 for oldquery,entry in[(875,0x9a1e00),(878,0x9a0e30)]:
  f=ROOT/f'target/rea/20261022-source-runtime-v3/{oldquery}-analyze_function.json';e=json.loads(json.loads(f.read_text())['content'][0]['text']);v=e['raw_result']['value'];assert int(v['procedure']['address'],16)==entry;add(entry,v['procedure']['body'],dict(query=str(f),query_sha256=sha(f),evidence_id=e['evidence_id']))
  if oldquery==875:
   for c in v['callees']:
    if int(c['address'],16)in(0x99c3e0,0x99c480,0x99f5e0):add(int(c['address'],16),c['body'],dict(parent_exact_callee_metadata=str(f),query_sha256=sha(f),evidence_id=e['evidence_id']))
 for n,entry in[(73,0x9a0dd0),(74,0x890020),(75,0x7f1320),(76,0x11e04f0)]:
  f,e,v=evidence(n);assert int(v['procedure']['address'],16)==entry;add(entry,v['procedure']['body'],dict(query=str(f),query_sha256=sha(f),evidence_id=e['evidence_id']))
 fresh=ROOT/'target/source-cell/native-runtime-fresh-loop-hybrid-host-v3/manifest.json';r=next(x for x in json.loads(fresh.read_text())['code_allowlist']if x['start']=='011E04D0');assert r['bytes']==31 and hashlib.sha256(image.read(0x11e04d0,31)).hexdigest()==r['sha256'];add(0x11e04d0,dict(ranges=[dict(start='0x11e04d0',end='0x11e04ee')],total_bytes=31),dict(accepted_allowlist=str(fresh),sha256=sha(fresh)))
 mirror=ROOT/'target/animation/root-selector-binding-source-prep-v1/root-data-mirror-source-domain-v1.json';m=json.loads(mirror.read_text());assert hashlib.sha256(image.read(0x99c340,97)).hexdigest()==m['code']['sha256'];add(0x99c340,dict(ranges=[dict(start='0x99c340',end='0x99c3a0')],total_bytes=97),dict(accepted_case_closed_source=str(mirror),sha256=sha(mirror)))
 add(0x11e052e,dict(ranges=[dict(start='0x11e052e',end='0x11e0533')],total_bytes=6),dict(kind='exactoriginalimportthunk',IAT='013CC478',dll='MSVCR90.dll',symbol='memset'))
 assert struct.unpack_from('<I',image.read(0x11e052e,6),2)[0]==0x13cc478
 by={int(x['address'],16):x for x in p['data']}
 for a,n in[(0x99c3a4,24),(0x99c3bc,32),(0x1485e24,4)]:by[a]=dict(address=f'{a:08X}',bytes=n,sha256=hashlib.sha256(image.read(a,n)).hexdigest());(out/f'data-{a:08X}.bin').write_bytes(image.read(a,n))
 p['data']=[by[x]for x in sorted(by)];p['unreachable_calls']={f'{a:08X}':why for a,why in TRAPS.items()};p['original_memset_import']=memset_import(image)
 p['owned_memset_contract']=dict(thunk='011E052E',IAT='013CC478',provider='MemsetProvider',exact_return='009A1EDF',destination='original EBP-1954h == WholeStackTop-19D4h',value=255,bytes=2295,calls=2)
 p['owned_trap_page']='00670000 first4KiBPEheaders/restINT3, no proprietarybytes there'
 p['scope']='Full9A1E00 entry + original reset/helper/stackprobe/shift/constructor/emptyfallback/secondarysetup; sourceAF0,AC&4=0,secondary2,7140preparedcapacity. Originalproducer before entry. No whole9C48C0/actor/allocation claim.'
 assert p['source_bytes']==13380;return p

def assembly(out,p):
 saved=combined.SIZES;combined.SIZES=SIZES
 try:text=combined.assembly(out,p)
 finally:combined.SIZES=saved
 text=text.replace(',RunQuery','')
 text=text.replace('Expected BYTE 1032 DUP(0)',f'Expected BYTE {MODEL} DUP(0)')
 text=text.replace('PUBLIC ProducerStackLow,','PUBLIC WholeStackLow,WholeStackTop,MemcpyDestination,WholeReturn,WholeCount,MemsetCalls,CurrentChannel,ParseCursor,DescriptorSum\nWholeStackLow DWORD 0\nWholeStackTop DWORD 0\nMemcpyDestination DWORD 0\nWholeReturn DWORD 0\nWholeCount DWORD 0\nMemsetCalls DWORD 0\nCurrentChannel DWORD 0\nParseCursor DWORD 0\nDescriptorSum DWORD 0\nPUBLIC ProducerStackLow,',1)
 text=text.replace('cmp eax,0670000h','cmp eax,0670000h') # module-base correction below
 text=text.replace('cmp eax,000990000h','cmp eax,000670000h')
 # Drop owned isolated query execution; retain its conformed validation/provider.
 a=text.index('RunQuery PROC');b=text.index('RunQuery ENDP')+len('RunQuery ENDP');text=text[:a]+text[b:]
 # New provider destination is original fullbuilder worklist, not an owned input.
 a=text.index('MemcpyProvider PROC');b=text.index('MemcpyProvider ENDP')+len('MemcpyProvider ENDP');part=text[a:b]
 part=part.replace('cmp dword ptr [esp+4],OFFSET IndicesBuffer','mov eax,MemcpyDestination\ncmp dword ptr [esp+4],eax').replace('mov edx,OFFSET IndicesBuffer','mov edx,MemcpyDestination').replace('cmp eax,OFFSET IndicesBuffer','cmp eax,MemcpyDestination').replace('cmp ProviderCalls,1','mov edx,dword ptr [HeaderBuffer+8]\nshl edx,1\ncmp ProviderCalls,edx');text=text[:a]+part+text[b:]
 # Runtime header is a complete prepared-case field snapshot, not a clip loader.
 a=text.index('ValidateHeader PROC');b=text.index('ValidateHeader ENDP')+len('ValidateHeader ENDP')
 lines=['ValidateHeader PROC','cmp dword ptr [HeaderBuffer],031444C42h','jne HostFailure','cmp dword ptr [HeaderBuffer+4],1','jb HostFailure','cmp dword ptr [HeaderBuffer+4],255','ja HostFailure','cmp dword ptr [HeaderBuffer+8],1','jb HostFailure','cmp dword ptr [HeaderBuffer+8],14','ja HostFailure','cmp dword ptr [HeaderBuffer+12],027Fh','jne HostFailure','cmp dword ptr [HeaderBuffer+28],0','jne HostFailure','mov eax,dword ptr [HeaderBuffer+8]','cmp dword ptr [HeaderBuffer+24],eax','jb HostFailure','imul eax,eax,24','cmp dword ptr [HeaderBuffer+24],eax','ja HostFailure','ret','ValidateHeader ENDP'];text=text[:a]+'\n'.join(lines)+text[b:]
 # Startup keeps every actual source/data/thunk/exit/IAT byte check. Replace its
 # low module-base predicate and loop/transport after handles only.
 text=text.replace('cmp eax,000990000h','cmp eax,000670000h')
 a=text.index('NextGroup:');b=text.index('HostMain ENDP',a)
 loop=['NextCase:','mov Remaining,32','mov Cursor,OFFSET HeaderBuffer','mov AllowEnd,1','call ReadExact','call ValidateHeader','mov edi,OFFSET PayloadBuffer','mov ecx,94492','xor eax,eax','rep stosb','mov eax,dword ptr [HeaderBuffer+4]','imul eax,eax,324','mov edx,dword ptr [HeaderBuffer+8]','imul edx,edx,80','add eax,edx','mov edx,dword ptr [HeaderBuffer+24]','shl edx,5','add eax,edx','cmp eax,94492','ja HostFailure','mov NeedBytes,eax','mov Remaining,eax','mov Cursor,OFFSET PayloadBuffer','mov AllowEnd,0','call ReadExact','call PrepareCase','call PrepareBody','mov Poison,0','call RunProducer','mov esi,OFFSET ProducerBuffer','mov edi,OFFSET Expected','mov ecx,44','rep movsb','mov Poison,-1','call RunProducer','mov esi,OFFSET ProducerBuffer','mov edi,OFFSET Expected','mov ecx,44','repe cmpsb','jne HostFailure']
 for name,n in [('Definition',1024),('State',16),('Producer',52),('Query',32),('Indices',1020)]:loop +=[f'mov esi,OFFSET {name}Buffer',f'mov edi,OFFSET {name}Shadow',f'mov ecx,{n}','rep movsb']
 loop +=['mov Poison,0','call RunWhole','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected',f'mov ecx,{MODEL}','rep movsb','mov Poison,-1','call RunWhole','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected',f'mov ecx,{MODEL}','repe cmpsb','jne HostFailure',f'mov Remaining,{REPLY}','mov Cursor,OFFSET ReplyBuffer','WriteWhole:','push 0','push OFFSET Transferred','push Remaining','push Cursor','push OutputHandle','call _WriteFile@20','test eax,eax','jz HostFailure','mov eax,Transferred','test eax,eax','jz HostFailure','cmp eax,Remaining','ja HostFailure','sub Remaining,eax','add Cursor,eax','cmp Remaining,0','jne WriteWhole','jmp NextCase']
 text=text[:a]+'\n'.join(loop)+'\n'+text[b:]
 # Add the new original memset IAT check before reading any case.
 old='cmp dword ptr [013CC47Ch],OFFSET MemcpyProvider';assert text.count(old)==1;text=text.replace(old,'cmp dword ptr [013CC478h],OFFSET MemsetProvider\njne HostFailure\n'+old)
 # Keep PrepareBody, RunProducer, ValidateQuery and provider boundaries. Additional
 # routines are owned, and all proprietary bodies remain immutable target blobs.
 where=text.index('HostMain PROC');text=text[:where]+owned_routines()+'\n'+text[where:]
 return text

def owned_routines():
 g=['PUBLIC PrepareCase,RunWhole,MemsetProvider','PrepareCase PROC']
 for name in('Clip','Channels','Descriptors','ChannelPtrs','Query','Indices'):
  g +=[f'mov edi,OFFSET {name}Buffer',f'mov ecx,{SIZES[name]}','mov eax,0A5h','rep stosb']
 g +=['mov dword ptr [ClipBuffer+144h],0','mov dword ptr [ClipBuffer+148h],OFFSET ChannelPtrsBuffer','mov dword ptr [ClipBuffer+154h],0','mov dword ptr [ClipBuffer+158h],0','mov eax,dword ptr [HeaderBuffer+8]','mov dword ptr [ClipBuffer+144h],eax','mov eax,dword ptr [HeaderBuffer+4]','imul eax,eax,324','add eax,OFFSET PayloadBuffer','mov ParseCursor,eax','mov CurrentChannel,0','mov DescriptorSum,0','PrepareChannel:','mov eax,CurrentChannel','imul edi,eax,0E4h','add edi,OFFSET ChannelsBuffer','mov dword ptr [ChannelPtrsBuffer+eax*4],edi','mov esi,ParseCursor','push edi','add edi,088h','mov ecx,19','rep movsd','pop edi','mov eax,dword ptr [esi]','cmp eax,1','jb HostFailure','cmp eax,24','ja HostFailure','add DescriptorSum,eax','mov dword ptr [edi+0DCh],eax','add esi,4','mov ParseCursor,esi','movzx eax,word ptr [edi+0ACh]','cmp eax,1','je ChannelACValid','cmp eax,9','je ChannelACValid','cmp eax,011h','jne HostFailure','ChannelACValid:','cmp byte ptr [edi+0AFh],0','jne HostFailure','cmp dword ptr [edi+09Ch],2','jne HostFailure']
 for at in(0xa0,0xa4,0xa8):g +=[f'cmp dword ptr [edi+0{at:X}h],0','jne HostFailure']
 g +=['mov eax,dword ptr [edi+08Ch]','cmp eax,1','je RootPrimary','cmp eax,5','je RootPrimary','push edi','lea esi,[edi+08Ch]','mov edi,OFFSET QueryBuffer','mov ecx,4','rep movsd','mov dword ptr [QueryBuffer+16],0','mov dword ptr [QueryBuffer+20],255','mov dword ptr [QueryBuffer+24],0','call ValidateQuery','pop edi','jmp PrimaryChecked','RootPrimary:']
 for at in(0x90,0x94,0x98):g +=[f'cmp dword ptr [edi+0{at:X}h],0','jne HostFailure']
 g +=['PrimaryChecked:','mov eax,CurrentChannel','imul eax,eax,768','add eax,OFFSET DescriptorsBuffer','mov dword ptr [edi+0E0h],eax','mov ecx,dword ptr [edi+0DCh]','shl ecx,5','push edi','mov edi,eax','mov esi,ParseCursor','rep movsb','mov ParseCursor,esi','pop edi','mov ecx,dword ptr [edi+0DCh]','mov esi,dword ptr [edi+0E0h]','CheckDescriptor:','cmp dword ptr [esi+10h],9','jae HostFailure','mov eax,dword ptr [esi]','and eax,0Fh','cmp eax,3','jne NoPseudoDescriptor','cmp dword ptr [esi+4],070E47545h','je HostFailure','NoPseudoDescriptor:','add esi,32','loop CheckDescriptor','inc CurrentChannel','mov eax,CurrentChannel','cmp eax,dword ptr [HeaderBuffer+8]','jb PrepareChannel','mov eax,DescriptorSum','cmp eax,dword ptr [HeaderBuffer+24]','jne HostFailure','mov eax,OFFSET PayloadBuffer','add eax,NeedBytes','cmp eax,ParseCursor','jne HostFailure','mov dword ptr [QueryBuffer+20],255','ret','PrepareCase ENDP',
 'MemsetProvider PROC','cmp dword ptr [esp],009A1EDFh','jne HostFailure','mov eax,WholeStackTop','sub eax,080h','cmp ebp,eax','jne HostFailure','lea eax,[ebp-01954h]','cmp dword ptr [esp+4],eax','jne HostFailure','cmp dword ptr [esp+8],0FFh','jne HostFailure','cmp dword ptr [esp+12],08F7h','jne HostFailure','inc MemsetCalls','cmp MemsetCalls,2','ja HostFailure','pushfd','pop eax','test eax,0400h','jnz HostFailure','push edi','mov edi,dword ptr [esp+8]','mov ecx,dword ptr [esp+16]','mov eax,0FFh','rep stosb','mov eax,dword ptr [esp+8]','pop edi','ret','MemsetProvider ENDP',
 'RunWhole PROC','mov DriverStack,esp','mov edi,OFFSET OutputBuffer','mov ecx,40','xor eax,eax','rep stosb','mov dword ptr [OutputBuffer+1Ch],OFFSET RecordsBuffer','mov dword ptr [OutputBuffer+20h],OFFSET RecordsBuffer',f'mov dword ptr [OutputBuffer+24h],OFFSET RecordsBuffer+{CAPACITY*16}','mov edi,OFFSET RecordsBuffer',f'mov ecx,{CAPACITY*16}','mov eax,0A5h','rep stosb','mov MemsetCalls,0','mov ProviderCalls,0','fninit','fldcw word ptr [HeaderBuffer+12]','ldmxcsr RequestedMxcsr','movd xmm0,Poison','pshufd xmm0,xmm0,0']
 for i in range(1,8):g +=[f'movaps xmm{i},xmm0']
 g +=['mov ecx,5','WholeProbePage:','sub esp,01000h','test dword ptr [esp],eax','loop WholeProbePage','sub esp,020h','test dword ptr [esp],eax','mov WholeStackLow,esp','mov edi,esp','mov ecx,05000h','mov eax,0A5h','rep stosb','mov eax,WholeStackLow','add eax,04F00h','mov WholeStackTop,eax','sub eax,01DE8h','mov MemcpyDestination,eax','mov esp,WholeStackTop','push OFFSET OutputBuffer','push OFFSET DefinitionBuffer','push OFFSET ClipBuffer','mov eax,009A1E00h','call eax','mov WholeReturn,eax','add esp,12','cmp esp,WholeStackTop','jne HostFailure','mov esp,DriverStack','cmp WholeReturn,1','jne HostFailure','cmp MemsetCalls,2','jne HostFailure','mov esi,WholeStackLow']
 for at in range(0,16,4):g +=[f'cmp dword ptr [esi+{at}],0A5A5A5A5h','jne HostFailure']
 g +=['mov esi,WholeStackTop','mov edi,WholeStackLow','add edi,05000h','WholeUpperStack:','cmp esi,edi','jae WholeUpperDone','cmp byte ptr [esi],0A5h','jne HostFailure','inc esi','jmp WholeUpperStack','WholeUpperDone:','cmp dword ptr [OutputBuffer+1Ch],OFFSET RecordsBuffer','jne HostFailure',f'cmp dword ptr [OutputBuffer+24h],OFFSET RecordsBuffer+{CAPACITY*16}','jne HostFailure','mov eax,dword ptr [OutputBuffer+20h]','sub eax,OFFSET RecordsBuffer','jb HostFailure','test eax,15','jnz HostFailure','shr eax,4',f'cmp eax,{CAPACITY}','ja HostFailure','mov WholeCount,eax','cmp dword ptr [OutputBuffer+10h],0','jne HostFailure','cmp dword ptr [OutputBuffer+18h],1','jne HostFailure','test eax,eax','jz WholeEmpty','cmp dword ptr [OutputBuffer+14h],2','jne HostFailure','jmp WholeCountChecked','WholeEmpty:','cmp dword ptr [OutputBuffer+14h],1','jne HostFailure','WholeCountChecked:','mov eax,dword ptr [OutputBuffer+20h]','mov edi,OFFSET RecordsBuffer',f'add edi,{CAPACITY*16}','WholeUnusedRecords:','cmp eax,edi','jae WholeUnusedDone','cmp byte ptr [eax],0A5h','jne HostFailure','inc eax','jmp WholeUnusedRecords','WholeUnusedDone:']
 for name,n in SIZES.items():
  if name in('Output','Records','Reply'):continue
  g +=[f'mov esi,OFFSET {name}Buffer',f'mov edi,OFFSET {name}Shadow',f'mov ecx,{n}','repe cmpsb','jne HostFailure']
 for name in SIZES:
  for side in('Before','After'):
   for at in range(0,16,4):g +=[f'cmp dword ptr [{name}{side}+{at}],0A5A5A5A5h','jne HostFailure']
 g +=['pushfd','pop eax','test eax,0400h','jnz HostFailure','mov edi,OFFSET ReplyBuffer',f'mov ecx,{MODEL}','mov eax,0FFh','rep stosb','mov eax,dword ptr [HeaderBuffer+16]','mov dword ptr [ReplyBuffer],eax','mov eax,dword ptr [HeaderBuffer+20]','mov dword ptr [ReplyBuffer+4],eax','mov eax,WholeReturn','mov dword ptr [ReplyBuffer+8],eax','mov eax,WholeCount','mov dword ptr [ReplyBuffer+12],eax','mov esi,OFFSET OutputBuffer','mov edi,OFFSET ReplyBuffer+16','mov ecx,28','rep movsb','mov esi,OFFSET RecordsBuffer','mov ecx,WholeCount','shl ecx,4','rep movsb',f'fnstcw word ptr [ReplyBuffer+{MODEL}]',f'fnstsw word ptr [ReplyBuffer+{MODEL+2}]',f'stmxcsr dword ptr [ReplyBuffer+{MODEL+4}]','mov ax,word ptr [HeaderBuffer+12]',f'cmp ax,word ptr [ReplyBuffer+{MODEL}]','jne HostFailure',f'test word ptr [ReplyBuffer+{MODEL+2}],03800h','jnz HostFailure',f'mov eax,dword ptr [ReplyBuffer+{MODEL+4}]','and eax,0FFFFFFC0h','cmp eax,01F80h','jne HostFailure','ret','RunWhole ENDP']
 return'\n'.join(g)+'\n'

def pages(image,p,unused,mapping):
 sy=selector.carrier_symbols(mapping);pages={BASE:bytearray(b'\xcc'*65536)};code={BASE}
 for f in p['functions']:
  for r in f['ranges']:
   a=int(r['start'],16);page=a&~0xffff;code.add(page);pages.setdefault(page,bytearray(b'\xcc'*65536))[a&0xffff:(a&0xffff)+r['bytes']]=image.read(a,r['bytes'])
 pages.setdefault(0x11e0000,bytearray(b'\xcc'*65536))[0x534:0x53a]=image.read(0x11e0534,6);code.add(0x11e0000)
 for a in TRAPS:code.add(a&~0xffff);pages.setdefault(a&~0xffff,bytearray(b'\xcc'*65536))
 for d in p['data']:
  a=int(d['address'],16);pages.setdefault(a&~0xffff,bytearray(65536))[a&0xffff:(a&0xffff)+d['bytes']]=image.read(a,d['bytes'])
 pages.setdefault(0x13c0000,bytearray(65536));struct.pack_into('<I',pages[0x13c0000],0xc478,sy['MemsetProvider']);struct.pack_into('<I',pages[0x13c0000],0xc47c,sy['MemcpyProvider']);pages[0x9c0000][0x4992:0x4999]=b'\xb8'+struct.pack('<I',sy['ProducerDone'])+b'\xff\xe0'
 return[(a,bytes(b),0x60000020 if a in code else 0x40000040)for a,b in sorted(pages.items())]
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--owned-exe',type=Path);p.add_argument('--owned-map',type=Path);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());image=SourceImage(a.source_exe)
 if a.owned_exe:
  source=json.loads((a.out/'source-closure.json').read_text());old=carrier.BASE,carrier.source_pages
  try:carrier.BASE=BASE;carrier.source_pages=lambda im,pr,d:pages(im,pr,d,a.owned_map);blob,r=carrier.build(a.owned_exe,a.owned_map,image,source)
  finally:carrier.BASE,carrier.source_pages=old
  (a.out/'builder-carrier-x86.exe').write_bytes(blob);r.update(native_executed=False,execution_admitted=False,carrier_sha256=sha(a.out/'builder-carrier-x86.exe'));(a.out/'carrier-build-review.json').write_text(json.dumps(r,indent=2)+'\n');return
 a.out.mkdir(parents=True,exist_ok=False);source=proof(image,a.out);(a.out/'source-closure.json').write_text(json.dumps(source,indent=2)+'\n');(a.out/'host-x86.asm').write_text(assembly(a.out,source));m=dict(schema='full-original9A1E00-prepared-ungrouped-source-entry-host-v1',tool_sha256=sha(__file__),asm_sha256=sha(a.out/'host-x86.asm'),source_bytes=13380,buffers=SIZES,projection_bytes=MODEL,reply_bytes=REPLY,capacity_records=CAPACITY,source_entry='009A1E00 full3491B',native_producer_entry='009C48DB partialoriginalcaller plusfull9B6920,notfull9C48C0',stack='Ownedreal20KBstackwindow probedpagebypage beforetouch; original43B11E04F0 receivesEAX1DD4 andretainsitsown ESP/returnsemantics. SourceTop-80 originalEBP; sourceworklistTop-1DE8 andclaimsTop-19D4. Windows errorsrestoreDriverStack.',source_allocation_claim=False,source_field_scope='Preparedsource217clipheaders/channels/descriptors andactualfinalizedbodyfields. GlobalDefinition350/354/otheropaqueunusedfields retaindeclaredcanaries/zero underrawmask154=0/expected158=0,notreconstructedclassmetadata.',compiled=False,native_executed=False,execution_admitted=False,compile_commands=json.loads((ROOT/'target/animation/nonroot-selector-Z-native-v1-complete/manifest.json').read_text())['compile_commands']);(a.out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
if __name__=='__main__':main()
