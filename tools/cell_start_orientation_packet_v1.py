"""Freeze original orientation/RNG bytes and source-only fixture inputs.

This preparer never compiles or executes a host or the game. No host or runner
is implemented by this packet; source, linked-host and exact-runner independent
reviews are mandatory before any future original-byte execution.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'tools'))
from cell_schema_probe import SourceImage, SOURCE_SHA256

SPANS = [(0xE74B98, 0xE74C10, '307b498427dd42946221a6104705e505985c31bb66e812e6dd8f935d51f2d4f4'),
         (0x935D40, 0x935D76, '35d81765a9410a57d5516506680395d783bffbc0d2ff6cc20e1b74aa1211afce')]
CONSTANTS = {0x143F52C: bytes.fromhex('0000802f'), 0x1471B2C: bytes.fromhex('0000003f'),
             0x15A8BB8: bytes.fromhex('db0fc940'), 0x15A8C40: bytes.fromhex('00000000000000000000803f')}
TRANSFERS = {0xE74B9D: 0x935D40, 0x935D44: 0x935D4A, 0x935D6E: 0x935D74}

def sha(data): return hashlib.sha256(data).hexdigest()
def file_sha(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda:f.read(1024*1024),b''): h.update(chunk)
    return h.hexdigest()

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--out',type=Path,required=True)
    p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    args=p.parse_args()
    args.out.resolve().relative_to((ROOT/'target/animation').resolve())
    if file_sha(args.source_exe)!=SOURCE_SHA256: raise ValueError('source executable hash mismatch')
    image=SourceImage(args.source_exe)
    sys.path.insert(0,str(ROOT/'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM
    from native_crt_acos_probe import decoder
    found={}; reports=[]; raw_spans=[]
    for start,end,expected in SPANS:
        raw=image.read(start,end-start)
        if sha(raw)!=expected: raise ValueError(f'source bytes differ: {start:08X}')
        ins=list(decoder().disasm(raw,start))
        if sum(i.size for i in ins)!=len(raw): raise ValueError('incomplete instruction decoding')
        for i in ins:
            if i.group(CS_GRP_CALL) or i.group(CS_GRP_JUMP):
                if len(i.operands)!=1 or i.operands[0].type!=X86_OP_IMM: raise ValueError('indirect flow')
                found[i.address]=i.operands[0].imm
        reports.append({'start':f'{start:08X}','end_exclusive':f'{end:08X}', 'sha256':sha(raw),
                        'disassembly':[f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in ins]})
        raw_spans.append(raw)
    if found!=TRANSFERS: raise ValueError(f'control-flow mismatch: {found}')
    for addr,expected in CONSTANTS.items():
        if image.read(addr,len(expected))!=expected: raise ValueError(f'constant mismatch: {addr:08X}')
    # Inputs are explicitly synthetic and are not a source actor/RNG snapshot.
    controls=[0x7f|pc|(rc<<10) for pc in (0x200,0x300) for rc in range(4)]
    corpus=[{'seed':seed,'cw':cw,'mxcsr':0x1f80,'angle_bits':0x40c90fdb,
             'axis_bits':axis} for seed in (1,0xffffffff,0xc0000000,0x80000000)
            for cw in controls for axis in ([0,0,0x3f800000],[0x80000000,0,0x3f800000])]
    encoded=(json.dumps(corpus,sort_keys=True,separators=(',',':'))+'\n').encode()
    args.out.mkdir(parents=True,exist_ok=False)
    for (start,_,_),raw in zip(SPANS,raw_spans): (args.out/f'{start:08X}.bin').write_bytes(raw)
    for addr,data in CONSTANTS.items(): (args.out/f'{addr:08X}.bin').write_bytes(data)
    (args.out/'cases.json').write_bytes(encoded)
    report={'schema':'cell-start-orientation-source-packet-v1','source_sha256':SOURCE_SHA256,
            'preparer_sha256':file_sha(Path(__file__)),'source_helper_sha256':file_sha(ROOT/'tools/cell_schema_probe.py'),
            'decoder_helper_sha256':file_sha(ROOT/'tools/native_crt_acos_probe.py'),
            'spans':reports,'transfers':{f'{a:08X}':f'{b:08X}' for a,b in TRANSFERS.items()},
            'constants':{f'{a:08X}':b.hex() for a,b in CONSTANTS.items()},
            'cases':len(corpus),'cases_sha256':sha(encoded),
            'entry_contract':'E74B98: EBP=synthetic actor, ESP=S guarded stack, empty x87, source RNG word at 01602760, DF clear',
            'source_writes':'actor+14..23, stack S-8..S-1 and S+18..S+27, RNG word 01602760',
            'source_exit':'E74C10 after E74C0D final actor word store',
            'runtime_supplied':'angle global 016B4DB0 from observed initial 015A8BB8; axis 015A8C40; RNG word; x87 CW and MXCSR',
            'fp_scope':'PC53/PC64, four rounding modes, masked exceptions, MXCSR1F80; C2 checkpoints not implemented',
            'excluded':'zero seed RDTSC, live actor/cache, matrix, unbounded/malformed inputs and runtime global provenance',
            'host_implemented':False,'runner_implemented':False,'ready_for_execution':False,
            'native_executed':False,'required':'independent source, linked-host and exact-runner safety review before original-byte execution'}
    (args.out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'cases':len(corpus),'cases_sha256':sha(encoded),'ready_for_execution':False}))
if __name__=='__main__': main()
