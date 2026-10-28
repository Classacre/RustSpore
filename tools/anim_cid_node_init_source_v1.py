"""Prepare exact nonzero-node init loop; no build, execution or full CID claim."""
import argparse, hashlib, json, sys
from pathlib import Path
from cell_schema_probe import SourceImage, SOURCE_SHA256
ROOT=Path(__file__).resolve().parent.parent
START=0x9c52f4
END=0x9c54b1
ONE=0x14861d0

def prepare(out):
    out.resolve().relative_to((ROOT/'target').resolve())
    image=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    sys.path.insert(0,str(ROOT/'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    from native_crt_acos_probe import decoder
    raw=image.read(START,END-START)
    rows=list(decoder().disasm(raw,START))
    assert sum(i.size for i in rows)==len(raw)==445
    boundaries={i.address for i in rows}
    jumps=[]
    for i in rows:
        assert not i.group(CS_GRP_CALL)
        if i.group(CS_GRP_JUMP):
            assert i.operands[0].type==X86_OP_IMM
            target=i.operands[0].imm
            assert target in boundaries or target==END
            jumps.append(dict(at=f'{i.address:08X}',target=f'{target:08X}'))
        for op in i.operands:
            if op.type==X86_OP_MEM and not op.mem.base and not op.mem.index:
                assert op.mem.disp==ONE and op.size==4
    one=image.read(ONE,4)
    assert one==bytes.fromhex('0000803f')
    out.mkdir(parents=True,exist_ok=False)
    (out/'original-loop445.bin').write_bytes(raw)
    (out/'source-one.bin').write_bytes(one)
    (out/'disassembly.txt').write_text('\n'.join(f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in rows)+'\n')
    m=dict(schema='CID-nonzero-node-init-source-loop-v1',source_sha256=SOURCE_SHA256,
      start=f'{START:08X}',end_exclusive=f'{END:08X}',original_bytes=445,
      original_sha256=hashlib.sha256(raw).hexdigest(),calls=0,jumps=jumps,
      native_executed=False,full_function=False,
      required_registers={'EDX':0,'EBX':0,'ESI':'CID','EBP':'address of CID+2E4','EDI':'Definition'},
      required_frame={'ESP+10':'nodecount in1..255','ESP+18':'Definition pointer'},
      required_memory={'CID+2E4':'allocated nodecount*0x2BC array','CID+70':'source initializer scale1.0','Definition+384':'nodecount*0x468 Definition blocks'},
      exit='fallthrough009C54B1; boundedhostmustreturnbefore9BD160call',
      float_operations=['sourceXORPSzero/MOVSSone','quaternionFLD/FSTP','positionMULSS','positionhistoryFLD/FSTP'],
      provenance='q21full9C5130body; exactloopspanonly; noallocator/ref/cache/lifetimeclaim')
    (out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
    return m
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True)
    print(json.dumps(prepare(p.parse_args().out)))
