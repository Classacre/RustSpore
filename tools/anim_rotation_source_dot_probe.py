"""Prepare finite-input overflow/unordered ROT contexts, never execute code."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct

import anim_quaternion_probe as quat
from anim_rotation_fixture import rows
from anim_rotation_native_probe import INPUT_BYTES

ROOT = Path(__file__).resolve().parent.parent
HOST = '972bf058d3cf7711b2e9f0ea5ff63c03d5e4e3f1a7a6dafa7fa17653e9d27f18'
ASM = 'b914ed8a7ff6cd972b60cfc3733628489895ec9a2e538a79988547f27f8e84c6'
H = 0x6E000000  # Exact binary32 2^93, within the measured source magnitude.


def corpus():
    frames = [0, 3, 11, 32]
    vectors = {
        'positive-infinity': [[H]*4]*4,
        'negative-infinity': [[H]*4, [H | 0x80000000]*4]*2,
        'unordered': [[H]*4, [H, H | 0x80000000, H, H | 0x80000000]]*2,
    }
    requests, index = [], []
    for blend in [0, 1]:
        for kind, quaternions in vectors.items():
            for previous in range(4):
                for following in range(4):
                    data = bytearray(rows(frames, qp=previous, qn=following,
                                          seed=0x99CA1B+previous*4+following))
                    for n, quaternion in enumerate(quaternions):
                        struct.pack_into('<4I', data, n*120+20, *quaternion)
                    for frame in [1., 7., 17.]:
                        left = sum(key <= frame for key in frames)-1
                        a, b = quaternions[left:left+2]
                        signs = {(x ^ y) >> 31 for x, y in zip(a, b)}
                        expected = ('unordered' if len(signs) == 2 else
                                    'negative-infinity' if 1 in signs else 'positive-infinity')
                        assert expected == kind
                        # Every product has exact magnitude2^186, above
                        # binary32 maximum. Mixed signs make the ordered
                        # source SSE sum unordered; no Python FP oracle.
                        assert all((v & 0x7FFFFFFF) == H for v in a+b)
                        for control in quat.CONTROLS:
                            request = struct.pack('<fIf5I', frame, 4, 40., 0,
                                                  control, blend, 1, 0)+data
                            assert len(request) == INPUT_BYTES
                            requests.append(request)
                            index.append(dict(row=len(index), family=f'source-dot-{kind}',
                                source=False, blend=blend, frame_bits=f'{quat.fbits(frame):08X}',
                                count=4, wrap=0, enabled=1, cw=f'{control:04X}',
                                previous_quaternion_mode=previous,
                                next_quaternion_mode=following, primary_dot=kind))
    return b''.join(requests), index


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    out = args.out.resolve()
    out.relative_to((ROOT/'target').resolve())
    data, index = corpus()
    out.mkdir(parents=True, exist_ok=False)
    (out/'inputs.bin').write_bytes(data)
    encoded_index = (json.dumps(index, indent=2)+'\n').encode()
    (out/'input-index.json').write_bytes(encoded_index)
    manifest = dict(schema='source-dot-overflow-unordered-rot-cohort-v1',
        host_executed=False, host_exe_sha256=HOST, host_source_sha256=ASM,
        input_bytes=INPUT_BYTES, rows=len(index), input_sha256=hashlib.sha256(data).hexdigest(),
        input_index_sha256=hashlib.sha256(encoded_index).hexdigest(),
        primary_dot_counts=dict(Counter(item['primary_dot'] for item in index)),
        blend_counts=dict(Counter(item['blend'] for item in index)),
        context='Both explicit blend policies, all12 masked CWs, all16 quaternion mode pairs, three interior frames, INFO20+ROT100, fixed wrap0/enabled1.',
        magnitude=dict(component_bits=f'{H:08X}', exact='2^93',
                       product_magnitude='2^186 > maximum finite binary32',
                       measured_source_maximum_component=1.983225806558461e28),
        predicate_proof='Every adjacent primary quaternion dot is +infinity, -infinity or unordered by exact product signs. COMISS zero,dot/JBE at99C936/99C939 sums unordered; COMISS raw,-1/JA at99CA1B/99CA41 clamps unordered to-1. All inputs remain finite; native outputs are retained without repair.',
        host_changes=False, original_bytes_changes=False,
        range_proof='Exact sorted integer keys0,3,11,32; frames1,7,17 are interior and duration40 covers final key. t in[0,1], all source slerp fractions[-1,2]. Quaternion magnitudes never enter fraction arithmetic; source dot clamp bounds any CRT argument.',
        scope='Focused whole-branch arithmetic boundary cohort using unchanged reviewed rotation-native-v2 host. Complements immutable30240 baseline contexts and full731383-row source census; no new original runtime policy or actor integration is assumed.',
        next_step='Root reviews exact new cohort/runner before hidden native execution.')
    (out/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(json.dumps(manifest, indent=2))


if __name__ == '__main__':
    main()
