"""Independent float-color fixtures from the previously indexed local PROP corpus."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from prop_numeric_probe import fields, digest


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('numeric_corpus', type=Path)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    args.out.mkdir(parents=True, exist_ok=False)
    resources = color_fields = color_values = components = negative = hdr = 0
    colors_by_type = {0x32: 0, 0x34: 0}
    with args.numeric_corpus.open('rb') as source, (args.out/'corpus.bin').open('xb') as output:
        assert source.read(8) == b'RSPROP01'
        count, = struct.unpack('<I', source.read(4))
        output.write(b'RSCOLOR1' + struct.pack('<I', count))
        for ordinal in range(count):
            length, primitive_digest = struct.unpack('<IQ', source.read(12))
            assert length <= 32*1024*1024
            data = source.read(length)
            assert len(data) == length
            # Revalidate the independently retained input record before deriving
            # a different comparison; the old primitive digest stays unchanged.
            assert digest(fields(data)) == primitive_digest, ordinal
            values = fields(data, {0x32, 0x34})
            expected = digest(values)
            output.write(struct.pack('<IQ', length, expected) + data)
            resources += 1
            for _, typ, _, words in values:
                color_fields += 1
                n = len(words)//({0x32: 3, 0x34: 4}[typ])
                color_values += n
                colors_by_type[typ] += n
                components += len(words)
                for word in words:
                    number, = struct.unpack('<f', struct.pack('<I', word))
                    negative += number < 0
                    hdr += number > 1
        assert source.read(1) == b''
    report = dict(resources=resources, color_fields=color_fields, color_values=color_values,
        components=components, colors_by_type={f'{k:04X}': v for k, v in colors_by_type.items()},
        negative_components=negative, above_one_components=hdr,
        input_sha256=hashlib.sha256(args.numeric_corpus.read_bytes()).hexdigest(),
        fixture_sha256=hashlib.sha256((args.out/'corpus.bin').read_bytes()).hexdigest(),
        scope='Complete named lists from the prior 82-package corpus; its four framing exclusions remain excluded.')
    (args.out/'inventory.json').write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
