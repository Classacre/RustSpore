"""Independent fullbuilder linked auditv2, exact owned copy-count JA guard."""
import argparse,json,hashlib
from pathlib import Path
import anim_ungrouped_binding_builder_audit_v1 as base
from anim_root_pose_sparse_carrier_audit_v2 import Image,syms
from native_crt_acos_probe import decoder
BASE_SHA='f1eba517a054966c30c3184441227dcf94dd0341f5b91f15b645291c1287f6be'
def audit(a):
 assert hashlib.sha256(Path(base.__file__).read_bytes()).hexdigest()==BASE_SHA
 r=base.audit(a);im=Image(a.carrier);s=syms(a.owned_map);rows=list(decoder().disasm(im.read(s['MemcpyProvider'],s['RunProducer']-s['MemcpyProvider']),s['MemcpyProvider']))
 positions=[i for i,x in enumerate(rows)if x.mnemonic=='cmp'and x.op_str==f'dword ptr [0x{s["ProviderCalls"]:x}], edx'];assert len(positions)==1;i=positions[0]
 assert rows[i-1].mnemonic=='shl'and rows[i-1].op_str=='edx, 1'
 assert rows[i+1].mnemonic=='ja'and rows[i+1].operands[0].imm==s['HostFailure']
 r.update(schema='whole9A1E00-prepared-ungrouped-independent-linked-audit-v2',owned_provider_maximum_count_branch='JA',owned_provider_count_maximum='2*sourcechannelcount',inherited_wrong_equality_rejected=True)
 return r
def main():
 p=argparse.ArgumentParser(description=__doc__)
 for n in('carrier','owned-exe','owned-map','prep','out'):p.add_argument('--'+n,type=Path,required=True)
 p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));a=p.parse_args();r=audit(a)
 with a.out.open('x')as f:json.dump(r,f,indent=2);f.write('\n')
 print(json.dumps(r,indent=2))
if __name__=='__main__':main()
