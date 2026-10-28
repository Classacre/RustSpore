"""Independent actual selector peer, static only; no native execution."""
import hashlib,json,struct,sys
from pathlib import Path
from types import SimpleNamespace
import anim_nonroot_selector_audit_v2 as auditor
import anim_nonroot_selector_native_probe_v2 as probe
from cell_schema_probe import SourceImage
from native_crt_acos_probe import decoder
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(out):
 h=ROOT/'target/animation/nonroot-selector-native-root-v2';inp=ROOT/'target/animation/nonroot-selector-inputs-v3';source=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe');image=SourceImage(source)
 hashes={'host-x86.exe':'708df109bb9a5e3ef47a5955652f11ef6841b3d93dd269406afbac5fbccb304b','selector-carrier-x86.exe':'9c7b990c155a12c1ce1db9226c17953226b9d132072f26b906d203a2e18402a3','host-x86.map':'43ff9288ee505d6752d1ae388378de2a05879f14387b451b4b676a5506f9eb0e'}
 assert all(sha(h/n)==v for n,v in hashes.items());fresh=auditor.audit(SimpleNamespace(carrier=h/'selector-carrier-x86.exe',owned_exe=h/'host-x86.exe',owned_map=h/'host-x86.map',prep=h,source_exe=source));assert fresh==json.loads((h/'root-linked-audit-v2.json').read_text()) and fresh['source_bytes']==4693
 proof=json.loads((h/'source-closure.json').read_text());asm=(h/'host-x86.asm').read_text();assert asm==probe.host_source(h,proof)
 # Independent source cdecl memcpy frame derivation: P before pushcount; src=P+150.
 rows=list(decoder().disasm(image.read(0x9b2205,0x19),0x9b2205));assert [(r.mnemonic,r.op_str) for r in rows]==[('lea','edx, [esi*4]'),('push','edx'),('lea','ecx, [esp + 0x154]'),('push','ecx'),('push','eax'),('call','0x11e0534'),('add','esp, 0xc')]
 assert 0x154-4+16==0x160 and rows[-1].address==0x9b221b
 # Source full function saves caller frame and restores it before ordinary RET.
 assert image.read(0x9b2150,12)==bytes.fromhex('558bec83e4f081ec44050000') and image.read(0x9b2223,4)==bytes.fromhex('8be55dc3')
 assert 'mov dword ptr [DefinitionBuffer+360h],07FC0CAFEh' in asm and 'mov dword ptr [DefinitionBuffer+36Ch],07FC0CAFEh' in asm
 assert 'mov dword ptr [edi+210h],0DEADFA11h' in asm and 'cmp dword ptr [QueryBuffer+8],0\njne HostFailure' in asm
 assert 'cmp dword ptr [QueryBuffer+24],0\njne HostFailure' in asm
 m=json.loads((inp/'manifest.json').read_text());pins=m['pins'].copy()
 for name,value in list(pins.items()):assert sha(ROOT/name)==value,name
 for name,value in m['files'].items():assert sha(inp/name)==value,name;pins[str((inp/name).relative_to(ROOT))]=value
 aliases=json.loads((inp/'selector-aliases.json').read_text());selectors=json.loads((inp/'selector-table.json').read_text());bodies=json.loads((inp/'body-index.json').read_text());assert len(aliases)==1644 and sum(x['admitted'] for x in aliases)==1641 and len(selectors)==59 and len(bodies)==1322
 assert all(not x['actual_actor_association'] and x['finalized_blocks']==x['source_rows']+x['generated_blocks'] for x in bodies);assert sum(x['generated_blocks'] for x in bodies)==4
 assert all(x[2:]==[0,0] and x[0]&0x40==0 for x in selectors)
 count=blocks=queries=0
 with (inp/'input.bin').open('rb') as f:
  while raw:=f.read(32):
   magic,n,q,cw,bid,r0,r1,r2=struct.unpack('<8I',raw);assert magic==0x314c4553 and 1<=n<=255 and q==118 and bid==count and (r0,r1,r2)==(0,0,0);assert cw&0xfffff0ff==0x7f and cw&0x300!=0x100
   payload=f.read(n*324);assert len(payload)==n*324 and sha_bytes(payload)==bodies[bid]['body_records_sha256']
   assert n==bodies[bid]['finalized_blocks'];qs=f.read(q*32);assert len(qs)==q*32
   got=[]
   for i in range(q):
    words=struct.unpack_from('<8I',qs,i*32);assert words[2:4]==(0,0) and words[4] in (0,1) and words[5]<=255 and words[6]==0 and words[7]==i;got.append((tuple(words[:4]),words[4]))
   assert set(got)=={(tuple(s),mirror) for s in selectors for mirror in (0,1)}
   count+=1;blocks+=n;queries+=q
 assert (count,blocks,queries)==(1322,46256,155996)
 paths=[Path(__file__),Path(auditor.__file__),Path(probe.__file__),ROOT/'tools/anim_root_pose_sparse_carrier_audit_v2.py',inp/'manifest.json',*[h/n for n in ['host-x86.asm','host-x86.exe','host-x86.map','selector-carrier-x86.exe','root-linked-audit-v2.json','source-closure.json','manifest.json']]];pins.update({str(p.relative_to(ROOT)):sha(p) for p in paths})
 report=dict(schema='actual-nonroot-selector-independent-font-peer-v1',no_blocking_static_issue_found=True,native_executed=False,host_executed=False,actual_linked_audit_reproduced=True,original_spans_exact=True,provider_ABI_and_bounds_closed=True,full_entry_frame_and_returns_closed=True,guarded_memory_bounds_closed=True,owned_control_and_failure_stack_closed=True,source_query_body_join_closed=True,pins=pins,linked_audit=fresh,bodies=count,blocks=blocks,queries=queries,source_aliases=1644,admitted_aliases=1641,Z_exclusions=3,provider_proof='At sourceP beforepushlen: pushlen->P-4,LEAESP154->P150,pushsrc/pushdest/CALL->providerESP=P-16;src=entryESP160. Originalreturn9B221B and cdecl cleanup12. Size=sourceESI*4 checkedpositive/aligned<=1020/body/capacity; destinationownedIndices; bounded disjoint source stack; onecopy; ESI/EDI preserved,EBX/EBP untouched. CRTthunk6B unchanged,readonlyIATprovider binding explicit.',limits='No native/protocol/Rust acceptance. Mode0/noZ/classmask0 only. Definition360/36C quietNaNs can be read in discarded noZ float calculations; maskedFP/exceptions not claimed as producer data. Block210opaque canary is not reconstructedclass data. 1641sourcepolicies cross actual1322finalizedbodies,not clip-to-actor association. No zerochild surrogate,constructor/ownership/pruning/livebinding admission.')
 with Path(out).open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(sha(out))
def sha_bytes(b):return hashlib.sha256(b).hexdigest()
if __name__=='__main__':main(sys.argv[1])
