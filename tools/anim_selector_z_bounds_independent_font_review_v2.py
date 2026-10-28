"""Actual corrected Z-producer v2 independent peer; no native execution."""
import hashlib,json,struct,sys
from pathlib import Path
from types import SimpleNamespace
import anim_selector_z_bounds_audit_v2 as audit
import anim_selector_z_bounds_native_probe_v1 as sourceprobe
from anim_selector_z_bounds_symbolic_review_v1 import check
from cell_schema_probe import SourceImage
from native_crt_acos_probe import decoder
from anim_root_pose_sparse_carrier_audit_v2 import Image,syms
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(output):
 h=ROOT/'target/animation/selector-Z-producer-native-root-v2';inp=ROOT/'target/animation/selector-Z-producer-inputs-v1';source=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe');im=SourceImage(source)
 for n,v in [('Z-carrier-x86.exe','f3928df59287c1794bcc28d925925ad655a075d2f8d6694ee03c80e2f71eabf1'),('host-x86.exe','10a6480d141e4cb2c42479a181381cea54e9698ef379aaa4207b02bea308b766'),('host-x86.map','d37dc0b626d376b6d3eb329c2c463bd1af74db068f925b99eeebbf469ce5aa4b')]:assert sha(h/n)==v
 fresh=audit.audit(SimpleNamespace(carrier=h/'Z-carrier-x86.exe',owned_exe=h/'host-x86.exe',owned_map=h/'host-x86.map',prep=h,source_exe=source));assert fresh==json.loads((h/'root-linked-audit-v2.json').read_text())
 proof=json.loads((h/'source-closure.json').read_text());oldasm=sourceprobe.assembly(h,proof);newasm=(h/'host-x86.asm').read_text();assert oldasm.replace('mov esi,SourceStack\nadd esi,24\nmov edi,StackWindow\n','mov esi,SourceStack\nadd esi,36\nmov edi,StackWindow\n')==newasm
 pe=Image(h/'Z-carrier-x86.exe');s=syms(h/'host-x86.map');rows=list(decoder().disasm(pe.read(s['RunPolicy'],s['HostMain']-s['RunPolicy']),s['RunPolicy']));by={r.address:i for i,r in enumerate(rows)}
 assert any(r.mnemonic=='sub' and r.op_str=='esp, 0x320' for r in rows)
 wd=rows[by[s['WriterDone']]:];assert [r.mnemonic for r in wd[:5]]==['mov','sub','cmp','jne','fnstsw'];assert wd[1].op_str=='eax, 0xc'
 assert sum(r.mnemonic=='fstp' and 'xword ptr' in r.op_str for r in wd)==1
 assert 'mov ecx,0C8h\nmov eax,0A5A5A5A5h\nrep stosd' in newasm and 'mov esp,DriverStack' in newasm
 assert im.read(0x9c48db,3)==bytes.fromhex('0f57c0') and im.read(0x9b692c,6)==bytes.fromhex('81ec40010000')
 symbolic=check(im)
 m=json.loads((inp/'manifest.json').read_text());assert m['records']==15864 and m['bodies']==1322 and m['blocks']==46256 and sha(inp/'input.bin')==m['input_sha256'];index=json.loads((inp/'body-index.json').read_text());up=ROOT/'target/animation/nonroot-selector-inputs-v3';upidx=json.loads((up/'body-index.json').read_text());assert len(index)==len(upidx)==1322
 pins=m['pins'].copy();assert all(sha(ROOT/k)==v for k,v in pins.items());bodies=blocks=policies=0
 with (inp/'input.bin').open('rb') as f,(up/'input.bin').open('rb') as original:
  for b,u in zip(index,upidx):
   hraw=f.read(32);magic,n,q,bid,*res=struct.unpack('<8I',hraw);assert magic==0x315a4242 and n==b['blocks']==u['finalized_blocks'] and q==12 and bid==bodies and not any(res)
   records=f.read(n*324);uh=struct.unpack('<8I',original.read(32));ur=original.read(uh[1]*324);original.read(uh[2]*32);assert records==ur and hashlib.sha256(records).hexdigest()==b['records_sha256']==u['body_records_sha256']
   cw=struct.unpack('<12I',f.read(48));assert list(cw)==m['policies']
   for j in range(n):
    v=struct.unpack_from('<39I',records,j*324+168)
    assert all(x&0x7fffffff<=0x49800000 for x in v[7:10]) and all(x&0x7fffffff<=0x40000000 for x in v[10:14]) and all(0<x<=0x49800000 for x in v[18:21])
   bodies+=1;blocks+=n;policies+=q
  assert not f.read(1) and not original.read(1)
 assert (bodies,blocks,policies)==(1322,46256,15864)
 paths=[Path(__file__),Path(audit.__file__),Path(sourceprobe.__file__),ROOT/'tools/anim_selector_z_bounds_native_probe_v2.py',ROOT/'tools/anim_selector_z_bounds_audit_v1.py',ROOT/'tools/anim_selector_z_bounds_symbolic_review_v1.py',ROOT/'tools/anim_root_pose_sparse_carrier_audit_v2.py',ROOT/'crates/spore-core/src/anim_selector_z_bounds.rs',ROOT/'crates/spore-core/examples/anim_selector_z_bounds_verify.rs.pending',inp/'manifest.json',inp/'body-index.json',inp/'input.bin',ROOT/'target/animation/selector-Z-producer-v1-independent-font-blocker.json',*[h/n for n in ['host-x86.exe','host-x86.map','host-x86.asm','Z-carrier-x86.exe','source-closure.json','root-linked-audit-v2.json','manifest.json']]];pins.update({str(p.relative_to(ROOT)):sha(p) for p in paths})
 report=dict(schema='selector-Z-producer-v2-independent-font-peer-v1',no_blocking_static_issue_found=True,native_executed=False,host_executed=False,actual_linked_audit_reproduced=True,original_spans_exact=True,partial_entry_frame_registers_closed=True,real_stack_write_bounds_closed=True,guarded_memory_bounds_closed=True,owned_control_and_failure_stack_closed=True,operation_order_model_source_closed=True,genuine_body_input_join_closed=True,pins=pins,symbolic_operation_order=symbolic,bodies=bodies,blocks=blocks,policies=policies,ABI='EnterrealESP=S,ESI=ownedDefinition,DF0,emptyx87,suppliedCW/MXCSR1F80,all8XMMpoisoned. Sourcepushes3args,fullhelperpreservesESI. HelperminimumESP=S-352 withinS-384..S+383window,two16Bguards. AllowedaboveSwritesS+4..35;S..3andS+36upguarded. ExitS-12withST0=y*y+z*z;capture80popsbeforetransport,restoreDriverStack beforeAPI. Definitiononly358..36C permitted.',limits='Static admission only. Oldv1blockedguard evidence retained; no original/input/modelnumeric changes. No native/Rust numeric acceptance. Binarycommutativeoperand canonicalization only in symbolicproof,never reassociation or normalization. Supplied actual finalizer geometry,includinggeneratedblocks,not inferredactorassociation. Callerbefore9C48DB/diagonal370/restconstraints/fullbuilder remainoutside scope.')
 with Path(output).open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(sha(output))
if __name__=='__main__':main(sys.argv[1])
