"""Parse the isolated font fixture protocol without launching any executable.

Retain original replies only under target/. Storage, heap counters, font+C8
and pitch padding are forensic fields, excluded from the active-pixel digest.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parent.parent
REQUEST, OUTPUT, ERROR = 0x31544E46, 0x31524E46, 0x31524546
CONTEXT, GLYPH, FINAL = 0x31585443, 0x31594C47, 0x31444E45
ROUND_TARGETS = {0x8AB680, 0x8AB6B0, 0x8AB6E0, 0x8AB710, 0x8AB740, 0x8ABA30, 0x8AB780, 0x8AB7D0}
CVT_TARGETS = {0x8CC230, 0x8CC250, 0x8CC2A0}
MAX_REPLY = 68 * 1024 * 1024


class ProtocolError(ValueError):
    pass


class NativeFailure(Exception):
    def __init__(self, fields, offset):
        self.record = dict(zip(
            ['magic', 'version', 'kind', 'detail', 'value', 'extra', 'phase',
             'context', 'glyph', 'live_allocations', 'live_bytes', 'setjmp_calls'], fields))
        self.record['reply_offset'] = offset


def require(condition, message):
    if not condition:
        raise ProtocolError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def request_bytes(glyphs, *, poison=0, layout=0, sequence=False):
    require(poison in (0, 0xA5) and layout in (0, 1), 'unsupported heap policy')
    require(isinstance(sequence, bool), 'sequence must be boolean')
    require(1 <= len(glyphs) <= 64 and all(isinstance(g, int) and 0 <= g < 578 for g in glyphs), 'glyph domain/count')
    return struct.pack('<4I', REQUEST, poison | (layout << 8), int(sequence), len(glyphs)) + struct.pack(f'<{len(glyphs)}I', *glyphs)


def parse_request(data):
    require(len(data) >= 16, 'truncated request header')
    magic, policy, mode, count = struct.unpack_from('<4I', data)
    require(magic == REQUEST, 'request magic')
    require(not policy & ~0x1FF and (policy & 0xFF) in (0, 0xA5), 'request heap policy')
    require(mode in (0, 1) and 1 <= count <= 64, 'request mode/count')
    require(len(data) == 16 + 4 * count, 'request body length or trailing bytes')
    glyphs = list(struct.unpack_from(f'<{count}I', data, 16))
    require(all(g < 578 for g in glyphs), 'request glyph domain')
    return {'mode': mode, 'glyphs': glyphs, 'poison': policy & 0xFF, 'layout': policy >> 8}


class Reader:
    def __init__(self, data):
        require(len(data) <= MAX_REPLY, 'oversized reply')
        self.data, self.position = data, 0

    def raw(self, length):
        require(0 <= length <= len(self.data) - self.position, f'truncated reply at {self.position}')
        result = self.data[self.position:self.position + length]
        self.position += length
        return result

    def words(self, count, *, record=False):
        if record and len(self.data) - self.position >= 4 and struct.unpack_from('<I', self.data, self.position)[0] == ERROR:
            start = self.position
            fields = struct.unpack('<12I', self.raw(48))
            require(fields[1] == 1, 'error-record version')
            require(self.position == len(self.data), 'trailing data after fatal record')
            raise NativeFailure(fields, start)
        return struct.unpack(f'<{count}I', self.raw(4 * count))


def controls(reader):
    cw, mxcsr, direction = reader.words(3)
    require(cw == 0x37F and mxcsr & ~0x3F == 0x1F80 and direction == 0, 'control-state mismatch')
    return {'x87_cw': cw, 'mxcsr': mxcsr, 'direction_flag': direction}


def hint_state(reader):
    cvt_count, storage_count, rounding, cvt_function, flags = reader.words(5, record=True)
    require((cvt_count, storage_count) == (173, 47), 'hint-state capacities')
    require(rounding in ROUND_TARGETS and cvt_function in CVT_TARGETS, 'hint function outside admitted set')
    require(flags <= 0xFFFF, 'hint flag width')
    cvt, storage = reader.raw(4 * cvt_count), reader.raw(4 * storage_count)
    return {'round_function': f'{rounding:08X}', 'cvt_function': f'{cvt_function:08X}',
            'raw_flags_e0_e1': flags, 'cvt_s32': list(struct.unpack('<173i', cvt)),
            'storage_s32': list(struct.unpack('<47i', storage)), 'raw_state_sha256': sha(cvt + storage),
            'comparison_scope': 'diagnostic snapshot; source initialization proven separately; excluded from glyph digest'}


def parse_reply(data, request=None, *, active_sink=None):
    reader, contexts, glyphs = Reader(data), [], []
    result = {'schema': 'native-font-glyph-reply-v1', 'reply_sha256': sha(data), 'reply_bytes': len(data)}
    try:
        fields = reader.words(12, record=True)
        require(fields[:2] == (OUTPUT, 1), 'reply magic/version')
        _, _, mode, count, poison, layout, font_bytes, glyph_capacity, cvt_capacity, storage_capacity, definitions, function_capacity = fields
        require(mode in (0, 1) and 1 <= count <= 64 and poison in (0, 0xA5) and layout in (0, 1), 'reply request fields')
        require((font_bytes, glyph_capacity, cvt_capacity, storage_capacity, definitions, function_capacity) == (117628, 578, 173, 47, 53, 85), 'reply source-font capacities')
        if request is not None:
            require((mode, count, poison, layout) == (request['mode'], len(request['glyphs']), request['poison'], request['layout']), 'reply/request header mismatch')
        result.update({'mode': mode, 'count': count, 'poison': poison, 'layout': layout})
        expected_contexts = 1 if mode else count
        for context_index in range(expected_contexts):
            fields = reader.words(8, record=True)
            magic, index, first, context_count, definition_count, ppem_x, ppem_y, setjmp_calls = fields
            expected_first, expected_count = (0, count) if mode else (context_index, 1)
            require((magic, index, first, context_count) == (CONTEXT, context_index, expected_first, expected_count), 'context order/count')
            require((definition_count, ppem_x, ppem_y) == (53, 15, 15) and setjmp_calls > 0, 'FPGM/PREP context fields')
            context = {'index': index, 'first': first, 'count': context_count, 'fpgm_definitions_checked': definition_count,
                       'ppem': [ppem_x, ppem_y], 'setjmp_calls': setjmp_calls, 'controls': controls(reader), 'hint': hint_state(reader)}
            contexts.append(context)
            for ordinal in range(first, first + context_count):
                fields = reader.words(15, record=True)
                magic, actual_context, actual_ordinal, glyph, width, height, pitch, byte_count, offset_e4, offset_e0, advance_d0, font_c8, bitmap_present, live, live_bytes = fields
                require((magic, actual_context, actual_ordinal) == (GLYPH, context_index, ordinal), 'glyph order/context')
                require(glyph < 578 and (request is None or glyph == request['glyphs'][ordinal]), 'glyph/request mismatch')
                require(width <= 4096 and height <= 4096 and width <= pitch <= 4096, 'glyph extent/pitch')
                require(byte_count == pitch * height <= 0x100000, 'glyph bitmap extent/size')
                require(bitmap_present in (0, 1) and (not byte_count or bitmap_present), 'glyph bitmap presence')
                require(live > 0 and live_bytes > 0, 'missing live glyph context')
                control, hint = controls(reader), hint_state(reader)
                pitched = reader.raw(byte_count)
                active = b''.join(pitched[row * pitch:row * pitch + width] for row in range(height))
                semantic = struct.pack('<7I', glyph, width, height, offset_e4, offset_e0, advance_d0, bitmap_present) + active
                item = {'context': context_index, 'ordinal': ordinal, 'glyph_index': glyph,
                        'width': width, 'height': height, 'pitch': pitch, 'pitched_bytes': byte_count,
                        'offset_e4_raw': offset_e4, 'offset_e0_raw': offset_e0, 'advance_d0_raw': advance_d0,
                        'bitmap_present': bool(bitmap_present), 'active_bytes': len(active),
                        'active_sha256': sha(active), 'pitched_sha256': sha(pitched),
                        'semantic_sha256': sha(b'FNTGLYPH1\0' + semantic),
                        'forensic_font_c8': font_c8, 'live_allocations': live, 'live_bytes': live_bytes,
                        'controls': control, 'hint': hint}
                glyphs.append(item)
                if active_sink is not None:
                    active_sink(item, active)
        fields = reader.words(15, record=True)
        require(fields[0] == FINAL and fields[1] == expected_contexts and fields[12:] == (0, 0, 0), 'final ownership/context fields')
        require(fields[2] == fields[3] and all(fields[i] > 0 for i in (2, 5, 6, 7, 8, 9)), 'final allocation/guard/read counters')
        require(reader.position == len(data), 'trailing reply bytes')
        result.update({'success': True, 'contexts': contexts, 'glyphs': glyphs,
                       'final': dict(zip(['contexts', 'allocations', 'frees', 'reallocations', 'guard_checks',
                                          'peak_live_bytes', 'setjmp_calls', 'stream_reads', 'stream_read_bytes',
                                          'memcpy_calls', 'memset_calls', 'live_allocations', 'live_bytes', 'reserved'], fields[1:]))})
    except NativeFailure as error:
        result.update({'success': False, 'native_error': error.record, 'contexts': contexts, 'glyphs': glyphs})
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reply', type=Path, required=True)
    parser.add_argument('--request', type=Path)
    parser.add_argument('--json', type=Path, required=True)
    parser.add_argument('--active-dir', type=Path)
    args = parser.parse_args()
    args.json.resolve().relative_to((ROOT / 'target').resolve())
    require(not args.json.exists(), 'do not overwrite a retained report')
    require(args.reply.stat().st_size <= MAX_REPLY, 'oversized reply file')
    expected = parse_request(args.request.read_bytes()) if args.request else None
    exports = []
    result = parse_reply(args.reply.read_bytes(), expected,
                         active_sink=lambda item, pixels: exports.append((item, pixels)))
    if args.active_dir:
        args.active_dir.resolve().relative_to((ROOT / 'target').resolve())
        args.active_dir.mkdir(parents=True, exist_ok=False)
        for item, pixels in exports:
            name = f"glyph-{item['ordinal']:02d}-{item['glyph_index']:03d}.alpha"
            (args.active_dir / name).write_bytes(pixels)
            item['active_file'] = str(args.active_dir / name)
    args.json.parent.mkdir(parents=True, exist_ok=True)
    args.json.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({k: result[k] for k in ('success', 'reply_sha256', 'reply_bytes')}, indent=2))
    if not result['success']:
        print(json.dumps(result['native_error'], indent=2))
        raise SystemExit(2)


if __name__ == '__main__':
    main()
