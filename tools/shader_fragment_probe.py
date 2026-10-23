"""Bounded source shader-fragment layout investigation, independent of Rust."""
import argparse
import json
import struct
from pathlib import Path


class Reader:
    def __init__(self, data):
        self.data, self.offset = data, 0

    def take(self, count):
        assert 0 <= count <= len(self.data) - self.offset, (self.offset, count, len(self.data))
        value = self.data[self.offset:self.offset+count]
        self.offset += count
        return value

    def word(self):
        return struct.unpack('>I', self.take(4))[0]

    def string(self):
        size = self.word()
        assert size <= 1_000_000, (self.offset - 4, size)
        return self.take(size).decode('utf-8')


def vertex(reader):
    offset = reader.offset
    metadata = reader.take(15)
    code, declarations = reader.string(), reader.string()
    count = reader.word()
    assert count <= 2048, (reader.offset-4, count)
    uniforms = [dict(declaration=reader.string(), metadata=reader.take(12).hex()) for _ in range(count)]
    name = reader.string() if int.from_bytes(metadata[-4:], 'big') & 2 else ''
    return dict(offset=offset, end=reader.offset, metadata=metadata.hex(), code=code,
                declarations=declarations, uniforms=uniforms, name=name)


def pixel(reader):
    offset = reader.offset
    metadata = reader.take(14)
    code, declarations = reader.string(), reader.string()
    count = reader.word()
    assert count <= 2048, (reader.offset-4, count)
    uniforms = [dict(declaration=reader.string(), metadata=reader.take(12).hex()) for _ in range(count)]
    texture_metadata = reader.take(36)
    name = reader.string() if int.from_bytes(metadata[-4:], 'big') & 2 else ''
    return dict(offset=offset, end=reader.offset, metadata=metadata.hex(), texture_metadata=texture_metadata.hex(),
                code=code, declarations=declarations, uniforms=uniforms, name=name)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('file', type=Path)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    reader = Reader(args.file.read_bytes())
    version = reader.word()
    records, pixels, error = [], [], None
    for index in range(255):
        try:
            record = vertex(reader)
            record['index'] = index
            records.append(record)
        except Exception as failure:
            error = dict(index=index, offset=reader.offset, error=str(failure),
                         context=reader.data[max(0,reader.offset-16):reader.offset+48].hex(' '))
            break
    if error is None:
        for index in range(256):
            try:
                record = pixel(reader)
                record['index'] = index
                pixels.append(record)
            except Exception as failure:
                error = dict(stage='pixel', index=index, offset=reader.offset, error=str(failure),
                             context=reader.data[max(0,reader.offset-16):reader.offset+48].hex(' '))
                break
    result = dict(file=str(args.file), version=version, vertex_records=records, pixel_records=pixels,
                  stop=error, consumed=reader.offset, bytes=len(reader.data), exact=reader.offset==len(reader.data))
    args.out.write_text(json.dumps(result, indent=2), encoding='utf-8')
    print('version', version, 'vertices', len(records), 'pixels', len(pixels), 'stop', error,
          'consumed', reader.offset, '/', len(reader.data))
    for record in records + pixels:
        if record['name']:
            print(record['index'], record['offset'], record['name'], len(record['code']), len(record['declarations']))


if __name__ == '__main__':
    main()
