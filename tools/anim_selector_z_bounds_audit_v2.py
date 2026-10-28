"""Independent Z linked audit v2: require corrected SourceS+36 guard."""
import argparse,json,hashlib
from pathlib import Path
import anim_selector_z_bounds_audit_v1 as base
from anim_root_pose_sparse_carrier_audit_v2 import Image,syms
from native_crt_acos_probe import decoder
BASE_SHA='8a8462a906d050e5138e54820892c0fd21236fb8d0a83c434bbe796a7c978b3d'
def audit(a):
 assert hashlib.sha256(Path(base.__file__).read_bytes()).hexdigest()==BASE_SHA
 r=base.audit(a);im=Image(a.carrier);s=syms(a.owned_map);rows=list(decoder().disasm(im.read(s['WriterDone'],s['HostMain']-s['WriterDone']),s['WriterDone']))
 assert sum(x.mnemonic=='add'and x.op_str=='esi, 0x24'for x in rows)==1
 assert not any(x.mnemonic=='add'and x.op_str=='esi, 0x18'for x in rows)
 i=next(i for i,x in enumerate(rows)if x.mnemonic=='add'and x.op_str=='esi, 0x24')
 assert rows[i-1].mnemonic=='mov'and rows[i-1].op_str==f'esi, dword ptr [0x{s["SourceStack"]:x}]'
 r.update(schema='actual-linked-Z-producer-independent-audit-v2',upper_real_stack_guard_offset=36,allowed_original_caller_above_stack_writes='S+4..35',owned_guard_correction_verified=True)
 return r
def main():
 p=argparse.ArgumentParser(description=__doc__)
 for n in('carrier','owned-exe','owned-map','prep','out'):p.add_argument('--'+n,type=Path,required=True)
 p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));a=p.parse_args();r=audit(a)
 with a.out.open('x')as f:json.dump(r,f,indent=2);f.write('\n')
 print(json.dumps(r,indent=2))
if __name__=='__main__':main()
