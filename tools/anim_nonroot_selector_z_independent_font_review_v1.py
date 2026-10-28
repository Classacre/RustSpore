"""Independent actual combined producer/all-policy selector peer; never native."""
import hashlib,json,struct,sys
from pathlib import Path
from types import SimpleNamespace
import anim_nonroot_selector_z_audit_v1 as audit
import anim_nonroot_selector_z_native_probe_v1 as probe
from anim_selector_z_bounds_symbolic_review_v1 import check
from cell_schema_probe import SourceImage
from anim_root_pose_sparse_carrier_audit_v2 import Image,syms
from native_crt_acos_probe import decoder
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(output):
 h=ROOT/'target/animation/nonroot-selector-Z-native-root-v1';inp=ROOT/'target/animation/nonroot-selector-Z-inputs-v1-complete';source=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe');im=SourceImage(source)
 for n,v in [('selector-Z-carrier-x86.exe','f3fdf07bd8700a3dafbcc4d93010f7b29800f4e5b2efa90f9515873246505698'),('host-x86.exe','75bbb23cc381951ade1718b3dc14b21436b83b84552ddd19c8c90f29b20f8807'),('host-x86.map','4a2c5dcac421ade7a54e7c901126803fc8d2057f9491fc66ac8de5b335666a4a')]:assert sha(h/n)==v
 fresh=audit.audit(SimpleNamespace(carrier=h/'selector-Z-carrier-x86.exe',owned_exe=h/'host-x86.exe',owned_map=h/'host-x86.map',prep=h,source_exe=source));assert fresh==json.loads((h/'root-linked-audit-v1.json').read_text()) and fresh['source_bytes']==8089
 p=json.loads((h/'source-closure.json').read_text());asm=(h/'host-x86.asm').read_text();assert asm==probe.assembly(h,p)
 assert next(d['bytes']for d in p['data'] if d['address']=='013EB8B0')==16
 seq='call PrepareBody\nmov Poison,0\ncall RunProducer';assert seq in asm and 'mov Poison,-1\ncall RunProducer' in asm
 baseline=asm.index('mov esi,OFFSET DefinitionBuffer\nmov edi,OFFSET DefinitionShadow',asm.index(seq));assert baseline>asm.index('mov Poison,-1\ncall RunProducer') and baseline<asm.index('mov CurrentQuery,0',baseline)
 part=asm[asm.index('RunProducer PROC'):asm.index('RunProducer ENDP')];assert 'mov esi,ProducerSourceStack\nadd esi,36\nmov edi,ProducerStackWindow' in part and 'cmp ecx,0358h\njb CompareDefinition\ncmp ecx,0370h\njb NextDefinition' in part
 assert all(f'movaps xmm{i},xmm0' in part for i in range(1,8));assert 'mov esi,OFFSET DefinitionBuffer' in part and 'mov eax,009C48DBh\njmp eax' in part and 'fstp tbyte ptr [StateBuffer]' in part and 'mov esp,DriverStack' in part
 # Original provider/source-call proof inherited byte-exact,independently retain exact frame derivation.
 assert im.read(0x9b220c,1)==b'\x52' and im.read(0x9b2216,5)==bytes.fromhex('e819e38200')
 symbolic=check(im)
 m=json.loads((inp/'manifest.json').read_text());assert (m['bodies'],m['blocks'],m['queries'],m['source_aliases'],m['admitted_source_aliases'],m['unique_selector_words'])==(1322,46256,161284,1644,1644,61);assert not m['retained_Z_exclusions'] and m['six_published_fields_identical_across12CWs']
 pins=m['pins'].copy();assert all(sha(ROOT/k)==v for k,v in pins.items())
 for name,v in m['files'].items():assert sha(inp/name)==v;pins[str((inp/name).relative_to(ROOT))]=v
 selectors=json.loads((inp/'selector-table.json').read_text());aliases=json.loads((inp/'selector-aliases.json').read_text());idx=json.loads((inp/'body-index.json').read_text());assert len(selectors)==61 and len(aliases)==1644 and {tuple(x['primary'])for x in aliases}=={tuple(x)for x in selectors};old=ROOT/'target/animation/nonroot-selector-inputs-v3';oldselectors=json.loads((old/'selector-table.json').read_text());oldmap={tuple(s):i for i,s in enumerate(oldselectors)};z=ROOT/'target/animation/selector-Z-producer-native-run-v2/native-v1/native-projection44.bin';controls=[0x7f|pc|rc for pc in (0,0x200,0x300)for rc in(0,0x400,0x800,0xc00)];count=blocks=preserved=0
 with (inp/'input.bin').open('rb')as f,(old/'input.bin').open('rb')as previous,(inp/'expected1032.bin').open('rb')as expected,(old/'expected1032.bin').open('rb')as oldexpected,z.open('rb')as native:
  for bi,b in enumerate(idx):
   hraw=f.read(32);magic,n,q,cw,bid,r0,r1,r2=struct.unpack('<8I',hraw);assert magic==0x314c4553 and q==122 and cw==0x27f and bid==bi and (r0,r1,r2)==(0,0,0)
   records=f.read(n*324);uh=struct.unpack('<8I',previous.read(32));ur=previous.read(uh[1]*324);previous.read(uh[2]*32);assert records==ur and hashlib.sha256(records).hexdigest()==b['body_records_sha256'];oldgold=[oldexpected.read(1032) for _ in range(118)]
   fields=None;chosen=None
   for policy in controls:
    v=native.read(44);assert len(v)==44 and struct.unpack_from('<2I',v)==(bi,policy)
    if fields is None:fields=v[8:32]
    assert fields==v[8:32]
    if policy==0x27f:chosen=v
   assert chosen.hex()==b['native_bounds_projection44_hex']
   for si,s in enumerate(selectors):
    for mirror in (0,1):
     query=f.read(32);assert query==struct.pack('<8I',*s,mirror,255,0,si*2+mirror);gold=expected.read(1032);assert struct.unpack_from('<2I',gold)==(bi,si*2+mirror)
     if tuple(s)in oldmap:assert gold[8:]==oldgold[oldmap[tuple(s)]*2+mirror][8:];preserved+=1
     count+=1
   blocks+=n
  assert not f.read(1)and not previous.read(1)and not expected.read(1)and not oldexpected.read(1)and not native.read(1)
 assert (count,blocks,preserved)==(161284,46256,155996)
 paths=[Path(__file__),Path(audit.__file__),Path(probe.__file__),ROOT/'tools/anim_selector_z_bounds_symbolic_review_v1.py',ROOT/'tools/anim_nonroot_selector_native_probe_v2.py',ROOT/'tools/anim_selector_z_bounds_native_probe_v1.py',ROOT/'tools/anim_root_pose_sparse_carrier_audit_v2.py',ROOT/'crates/spore-core/src/anim_nonroot_selector_z.rs',ROOT/'crates/spore-core/examples/anim_nonroot_selector_z_verify.rs.pending',inp/'manifest.json',*[h/n for n in ['host-x86.exe','host-x86.map','host-x86.asm','selector-Z-carrier-x86.exe','root-linked-audit-v1.json','source-closure.json','manifest.json']]];pins.update({str(x.relative_to(ROOT)):sha(x)for x in paths})
 report=dict(schema='combined-selector-Z-independent-font-peer-v1',no_blocking_static_issue_found=True,native_executed=False,host_executed=False,pins=pins,actual_linked_audit_reproduced=True,original_spans_exact=True,producer_partial_entry_real_stack_closed=True,provider_full_selector_ABI_closed=True,producer_before_queries_and_rebaseline_closed=True,guarded_memory_bounds_closed=True,owned_control_and_failure_stack_closed=True,all_native12CW_bounds_joins_closed=True,bodies=1322,blocks=blocks,queries=count,policies=61,source_aliases=1644,old_slots_preserved=preserved,symbolic_operation_order=symbolic,scope_correction='Auditfull_entriesincludes009C48DB butthat isPARTIALcaller9C48DB..4992,notfullfunction. Fullhelper9B6920/sourcefullselector9B2150 andcapabilityhelper9B2E20 arefullentries.',ABI='ProducerESI=Definition,realESP=S;minS+12..23/maxS+24..35;sourcefloorS-352,windowS-384..+383;guardS..3/S+36up;exitS-12/ST0captured80. Fullselectorproviderreturn9B221B,srcproviderEntryESP+160,destIndices,size4*sourceESI boundedcapacity/body/count/nonoverlap once-only.',limits='Staticcompositionadmission only. No suppliedreference/Rustbounds injected intohost; twooriginalproducerpasses precedequeries andonlysixallowedDefinitionwrites/ownedstateare rebaselined. Sixfieldsjoinacceptednative12CWrows; newcombinednative/Rustcomparison pending. 43upstreambodyexclusions; no liveactorassociation,full9C3150/diagonal370/constraints,AEownership/construction/pruning/wholebuilder admission.')
 with Path(output).open('x')as f:json.dump(report,f,indent=2);f.write('\n')
 print(sha(output))
if __name__=='__main__':main(sys.argv[1])
