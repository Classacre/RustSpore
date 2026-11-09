"""Freeze and inspect the two original instance-reference spans; never execute.

This is source-packet preparation, not a native host or admission to execute.
Output must be a new directory under target/animation. Proprietary bytes stay
there and are not copied into the repository.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'tools'))
from cell_schema_probe import SourceImage, SOURCE_SHA256
from native_crt_acos_probe import decoder

SPANS = ((0x9A3140, 0x9A3192), (0x9A3396, 0x9A33AC))
TRANSFERS = {
    0x9A3156: 0x9A316D, 0x9A315A: 0x9A3162,
    0x9A3166: 0x9A316D, 0x9A3168: 0x99CD90,
    0x9A3177: 0x9A3192, 0x9A317B: 0x9A3184,
    0x9A317F: 0xA16FC0, 0x9A3189: 0x9A3192,
    0x9A318D: 0x9C4B50, 0x9A339B: 0x9A33AC,
    0x9A33A5: 0x9A33AC, 0x9A33A7: 0x99EDE0,
}

def sha(data):
    return hashlib.sha256(data).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--source-exe', type=Path, default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    args = parser.parse_args()
    args.out.resolve().relative_to((ROOT / 'target/animation').resolve())
    image = SourceImage(args.source_exe)
    assert sha(args.source_exe.read_bytes()) == SOURCE_SHA256
    sys.path.insert(0, str(ROOT / 'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM
    report = []
    found = {}
    originals = []
    for start, end in SPANS:
        data = image.read(start, end - start)
        decoded = list(decoder().disasm(data, start))
        assert sum(i.size for i in decoded) == end - start
        for ins in decoded:
            if ins.group(CS_GRP_CALL) or ins.group(CS_GRP_JUMP):
                assert len(ins.operands) == 1 and ins.operands[0].type == X86_OP_IMM
                found[ins.address] = ins.operands[0].imm
        originals.append(data)
        report.append({'start': f'{start:08X}', 'end_exclusive': f'{end:08X}',
                       'sha256': sha(data), 'bytes': len(data),
                       'instructions': [f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in decoded]})
    assert found == TRANSFERS, (found, TRANSFERS)
    args.out.mkdir(parents=True, exist_ok=False)
    for (start, _), data in zip(SPANS, originals):
        (args.out / f'{start:08X}.bin').write_bytes(data)
    manifest = {'schema': 'instance-handoff-source-packet-v1', 'source_sha256': SOURCE_SHA256,
                'preparer_sha256': sha(Path(__file__).read_bytes()), 'native_executed': False,
                'spans': report, 'host_complete': False,
                'limits': 'Source bytes and control flow only; no host, callback, memory, ABI or lifecycle validation.'}
    (args.out / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'out': str(args.out), 'native_executed': False, 'host_complete': False}))

if __name__ == '__main__':
    main()
