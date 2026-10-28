"""Prepare static-only input and source evidence for a future x86 host.

Does not compile, execute native code, produce a host, or prove equivalence.
"""
import argparse
import hashlib
import itertools
import json
from pathlib import Path
import random
import struct
import sys

from cell_schema_probe import SourceImage, SOURCE_SHA256
from native_crt_acos_probe import decoder

ROOT = Path(__file__).resolve().parent.parent
START, END = 0xE7CE91, 0xE7CF19
CAP, GLOBAL = 0x14846DC, 0x16B4BE4


def digest(data):
    return hashlib.sha256(data).hexdigest()


def validate(row):
    if not isinstance(row, dict) or set(row) != {'progress', 'counter20', 'counter24', 'counter28', 'amount', 'source'}:
        raise ValueError('wrong record keys')
    for key in ('progress', 'counter20', 'counter24', 'counter28', 'amount'):
        if type(row[key]) is not int or not -2147483648 <= row[key] <= 2147483647:
            raise ValueError('signed word required: ' + key)
    if type(row['source']) is not int or not 0 <= row['source'] <= 255:
        raise ValueError('source byte required')


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--source-exe', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    a.out.resolve().relative_to((ROOT / 'target').resolve())
    image = SourceImage(a.source_exe)
    code = image.read(START, END - START)
    cap = image.read(CAP, 4)
    if cap != struct.pack('<I', 1000):
        raise ValueError('unexpected source cap')
    rows = list(decoder().disasm(code, START))
    boundaries = {i.address for i in rows}
    if not rows or sum(i.size for i in rows) != len(code):
        raise ValueError('incomplete instruction decoding')
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM
    exits = set()
    for i in rows:
        if i.group(CS_GRP_CALL):
            raise ValueError('unexpected call')
        if i.group(CS_GRP_JUMP):
            if i.operands[0].type != X86_OP_IMM:
                raise ValueError('indirect branch')
            target = i.operands[0].imm
            if target not in boundaries:
                exits.add(target)
    # CF19 is the fallthrough, D090 is the unchanged branch target.
    if exits != {0xE7CF19, 0xE7D090} or rows[-1].address != 0xE7CF13:
        raise ValueError('unexpected exit structure')
    values = [-2147483648, -1, 0, 999, 1000, 1001, 2147483647]
    amounts = [-2147483648, -1, 0, 1, 1000, 2147483647]
    counters = [0, -2147483648, 2147483647]
    cases = []
    for progress, amount, source, counter in itertools.product(values, amounts, (0, 1, 2, 255), counters):
        cases.append(dict(progress=progress, counter20=counter, counter24=counter,
                          counter28=counter, amount=amount, source=source))
    rng = random.Random(0xE7CE40)
    for _ in range(1000):
        w = lambda: rng.randrange(-2147483648, 2147483648)
        cases.append(dict(progress=w(), counter20=w(), counter24=w(), counter28=w(),
                          amount=w(), source=rng.randrange(256)))
    for row in cases:
        validate(row)
    bad = [dict(cases[0], source=-1), dict(cases[0], source=256),
           dict(cases[0], amount=2147483648), dict(cases[0], progress=True),
           {k: v for k, v in cases[0].items() if k != 'amount'}]
    for row in bad:
        try:
            validate(row)
        except ValueError:
            continue
        raise ValueError('malformed input accepted')
    a.out.mkdir(parents=True, exist_ok=False)
    (a.out / 'source-slice.bin').write_bytes(code)
    (a.out / 'source-cap.bin').write_bytes(cap)
    (a.out / 'inputs.json').write_text(json.dumps(cases, indent=2) + '\n', encoding='utf-8')
    audit = dict(schema='cell-progress-slice-static-prep-v1', source_sha256=SOURCE_SHA256,
        source_slice_sha256=digest(code), source_slice_start=hex(START), source_slice_end_exclusive=hex(END),
        cap_address=hex(CAP), cap_bytes=cap.hex(), global_pointer_address=hex(GLOBAL),
        source_instruction_count=len(rows), instructions=[dict(address=hex(i.address), size=i.size,
            mnemonic=i.mnemonic, operands=i.op_str) for i in rows], branch_external=[hex(x) for x in exits],
        fallthrough=hex(END), input_count=len(cases), seed=hex(0xE7CE40), rejected_examples=len(bad),
        native_executed=False, host_prepared=False, scope='post E79750 accounting only; no downstream effects or return value')
    (a.out / 'audit.json').write_text(json.dumps(audit, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({k: audit[k] for k in ('source_slice_sha256', 'source_instruction_count', 'input_count', 'host_prepared')}))


if __name__ == '__main__':
    main()
