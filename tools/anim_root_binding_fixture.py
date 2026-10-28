"""Independent integer reference/cohort for the bounded ordinary root constructor.

No original code is executed. Source descriptor coverage uses literal prepared
descriptor rows in isolated, explicitly supplied claim contexts, not live bindings.
"""
import argparse
import csv
import hashlib
import itertools
import json
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parent.parent
INPUT_BYTES, OUTPUT_BYTES = 364, 48
TSV = ROOT/'target/source-cell/runtime-data-source-census-v2/prepared-data-components.tsv'
GATES = ROOT/'target/animation/root-selector-binding-source-prep-v1/source-gates-v1.json'


def encode(flags=1, root=0, ordinal=0, channel_byte=7, channel_index=11,
           words=(0x12345678, 0x87654321), mirror=0, owner=9, ac=1,
           secondary=0xffffffff, descriptors=(), claims=(255,)*9):
    assert len(descriptors) <= 24 and len(claims) == 9
    header = struct.pack('<16I', flags, root, ordinal, channel_byte, channel_index,
                         *words, mirror, owner, ac, len(descriptors), secondary, 0, 0, 0, 0)
    rows = b''.join(struct.pack('<3I', *row) for row in descriptors)
    return header+rows+bytes(288-len(rows))+bytes(claims)+bytes(3)


def reference(record):
    assert len(record) == INPUT_BYTES
    h = struct.unpack_from('<16I', record)
    f, root, ordinal, channel, index, word0, word1, mirror, owner, ac, count, secondary = h[:12]
    if f not in (1, 5) or root != 0:
        raise ValueError('selector')
    if count > 24 or channel > 255 or index > 255 or mirror > 1 or owner > 255:
        raise ValueError('range')
    if secondary != 0xffffffff and secondary > 255:
        raise ValueError('secondary')
    if any(h[12:]) or any(record[64+count*12:352]) or any(record[361:]):
        raise ValueError('padding')
    if ac & 0x32 not in (0, 0x20):
        raise ValueError('mirror-mode')
    claims = bytearray(record[352:361])
    flags = 2 if mirror and ac & 0x32 == 0 else 0
    bits = 0
    admitted = False
    for i in range(count):
        df, tag, slot = struct.unpack_from('<3I', record, 64+i*12)
        if df & 15 == 3 and tag == 0x70e47545:
            raise ValueError('pseudo')
        if slot >= 9 or claims[slot] not in (255, owner):
            continue
        admitted = True
        if not df & 16:
            continue
        claims[slot] = owner
        bits += 1 << i
        if df & 15 == 1:
            flags |= 32
    if not bits and admitted and ac & 8:
        raise ValueError('empty-fallback')
    if secondary != 0xffffffff:
        flags |= 16
    secondary_byte = secondary & 255
    output = struct.pack('<4I', bool(bits), 0, bool(bits), bool(bits and secondary_byte != 255))
    output += bytes(claims)+bytes(3)
    output += (struct.pack('<4I', word0, word1, (index << 24)+bits,
               channel << 8 | secondary_byte << 16 | flags << 24) if bits else bytes(16))
    return output+bytes(4)


def prepare(out):
    out.resolve().relative_to((ROOT/'target').resolve())
    out.mkdir(parents=True, exist_ok=False)
    requests, replies, labels = [], [], []
    def add(label, data):
        requests.append(data)
        replies.append(reference(data))
        labels.append(label)
    # Ordinal mask boundary, all slots, admission, enabled/kind and mirror/secondary.
    for pos, slot, claim, df, mirror, secondary, primary in itertools.product(
            (0, 7, 23), (0, 8, 9, 0xffffffff), (255, 9, 8),
            (0, 1, 3, 16, 17, 19), (0, 1), (0xffffffff, 0, 255), (1, 5)):
        rows = [(0, 0, 9)]*pos+[(df, 0x98c942b3, slot)]
        add('synthetic', encode(flags=primary, mirror=mirror, secondary=secondary,
             descriptors=rows, claims=(claim,)*9))
    # Repeated slots, disabled then enabled, disjoint and empty descriptors.
    for rows in ([], [(0, 0, 0)], [(17, 0, 0), (19, 0, 0)],
                 [(0, 0, 0), (19, 0, 8)], [(19, 0, i % 9) for i in range(24)]):
        add('sequence', encode(descriptors=rows))
    gates = json.loads(GATES.read_text())
    key = lambda r: (r['package'], r['low'], r['type'], r['scope'], int(r['channel']), int(r['component']))
    gate_map = {key(r): r for r in gates['records']}
    source_rows = list(csv.DictReader(TSV.open(newline=''), delimiter='\t'))
    assert len(source_rows) == len(gate_map) == 2086
    clips = set()
    for r in source_rows:
        g = gate_map[key(r)]
        assert r['status'] == 'matched' and r['source_sha256'] == g['source_sha256']
        pos = int(r['component'])
        assert 0 <= pos < 24
        clips.add(key(r)[:4])
        rows = [(0, 0, 9)]*pos+[(int(r['prepared_flags'], 16), int(r['tag'], 16), int(r['prepared_slot'], 16))]
        for mirror, claim in itertools.product((0, 1), (255, 9, 8)):
            add('source-isolated', encode(flags=int(g['primary'][0], 16),
                channel_index=int(r['channel']), mirror=mirror, ac=int(g['metadataAC'], 16),
                descriptors=rows, claims=(claim,)*9))
    errors = [encode(flags=2), encode(root=1), encode(ac=2),
              encode(descriptors=[(19, 0x70e47545, 0)]),
              encode(ac=9, descriptors=[(0, 0, 0)])]
    excessive = bytearray(encode())
    struct.pack_into('<I', excessive, 40, 25)
    errors.append(bytes(excessive))
    for data in errors:
        try:
            reference(data)
        except ValueError:
            pass
        else:
            raise AssertionError('negative input accepted')
    for name, data in [('inputs.bin', b''.join(requests)), ('reference.bin', b''.join(replies)),
                       ('rejected.bin', b''.join(errors))]:
        (out/name).write_bytes(data)
    report = dict(schema='ordinary-root-constructor-reference-v1', native_executed=False,
        input_bytes=INPUT_BYTES, output_bytes=OUTPUT_BYTES, records=len(requests),
        source_descriptors=len(source_rows), source_clips=len(clips),
        source_contexts=labels.count('source-isolated'),
        synthetic_contexts=len(requests)-labels.count('source-isolated'), rejected_cases=len(errors),
        source_scope='One literal prepared data descriptor at its actual component ordinal per fixture; other ordinals are explicit invalid-slot placeholders. Claims/owner/secondary/words are supplied contexts. No whole-channel pruning or live actor association.',
        files={n: hashlib.sha256((out/n).read_bytes()).hexdigest() for n in ('inputs.bin', 'reference.bin', 'rejected.bin')},
        sources={str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in (TSV, GATES)})
    (out/'manifest.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--out', type=Path, required=True)
    prepare(p.parse_args().out)
