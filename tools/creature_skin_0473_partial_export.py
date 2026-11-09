"""Opt-in inspection artifact from three pinned, conditional native comparisons.

This does not produce a finalized mesh or identify a live creature. Default
output omits UVs, weights and materials; optional model outputs state their limits.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[1]
SPECS = {
    'average': ('creature-skin-average-0473-native-20261109', '6bef492ef52cc742f63f48df2d5ffb2e4be84f120bfc19e05d544d7d7046a997', 'ec7c9288caf3ba636782d98b91abab0e393ba0c2e637094109d10902d2340182', 704),
    'normal': ('creature-skin-normal-0473-native-20261109', '2307929623cd4e2499cb4867bde56d6d89bd5aff0017a4edfa1a697f5111782d', 'f7959a913b1ea6198e032518c683a8c0b27d8a544a305ddf6c72934d048b0cbb', 1552),
    'indices': ('creature-skin-index-copy-native-20261109', '30cbbcc2f76884bfe1071e47f1accef1fb40dd1b6a2a4c700be9e845e31705bb', '5d6bd1ff3502e9445f47ff58985fdf7788846c63cdc8d62664b71f09789a2b34', 3036),
}
RECORD_SPEC = {
    'synthetic_initial_records': ('creature-skin-record-0473-native-20261109', '361dcd3d3e11c065a124357eb30678687c43d2c2927b5a0d30ff8c71d1dbeeca', '739b7c31d21fc9cbaf171ae26294775eec4283196aef2638351d69d8ae2b4b14', 880),
}
ROW_FOLDER = 'creature-skin-weights-0473-rowcoords-20261109'
ROW_RECEIPT_SHA = '90a1c5c82fa48258a43f5c8d79d2372029add2efaf0b943494aeda7ab0e4dffa'
ROW_STREAM_SHA = 'de642b7e6536e7676e081f7a5a17710c167f7a62573cfb5bd33dfbd418ed686a'
ROW_LIMITS = ('Frozen post-mutation source rows, same-object/lifetime, supplied grid and no correction or smoothing are admissions. Raw index bytes are not palette or bone identifiers; records are initial, not final skin weights or observed live state.')

SMOOTH_FOLDER = 'creature-skin-smoothing-0473-native-v3-20261109'
SMOOTH_RECEIPT_SHA = 'c3fd128b4049e4f7ee40ca8c0a8f598cfd68f89562651e5f777846f0b0d3c1f8'
SMOOTH_STREAM_SHA = 'e0ba1b69c678bf41b3a8eb0e4ea8ae6578b86df99dce93dc0a9d90b0d5fe9a22'
CALLER_SMOOTH_FOLDER = 'creature-skin-smoothing-caller-alpha-0473-native-20261109'
CALLER_SMOOTH_RECEIPT_SHA = 'fce30d8553893daa8e4c7592722ca824edf821948cae81df9e803caa063de9e6'
CALLER_SMOOTH_STREAM_SHA = 'f634488e3bbdcd20f942dadb2c6a40966d060224851896ce794e7f3cc7b866be'
CORNER_FOLDER = 'creature-skin-corner-attributes-0473-prep-v2-20261109'
CORNER_RECEIPT_SHA = 'b8ef734dfc089b6ca115b31ab97c829a0fb094d8bc77531dfa600a0e211b11ed'
CORNER_STREAM_SHA = '4c3e251f216551e5d9cce8ca1de3dba57a9ce62f7f46f8f8c7d16828bde23753'
SMOOTH_LIMITS = ('Conditional frozen-row initial records and topology, explicitly supplied 0.5 factor and pass count, zero-initialized scratch and destination. Not observed runtime configuration, final weights, live creature or palette identities.')

RECORD_LIMITS = ('Synthetic endpoint coordinates (i,0,0), conditional same-object identity, and no correction or smoothing are caller admissions. Raw index bytes are not palette or bone identifiers; records are initial, not final skin weights.')

LIMITS = ('Conditional frozen partial 0473 source rows, caller-supplied step 0.15, zero padding, '
          'model-built grid, derived gradient coefficients and scalar dispatch; not a live identity '
          'or finalized mesh. UVs, skin weights, materials, winding interpretation, correction '
          'and remaining finalization are unresolved.')

def digest(data):
    return hashlib.sha256(data).hexdigest()

def read_inputs(target, specs=SPECS):
    streams, provenance = {}, {}
    for key, (folder, receipt_hash, stream_hash, size) in specs.items():
        directory = target / folder
        receipt = (directory / 'result.json').read_bytes()
        stream = (directory / 'host-stdout.bin').read_bytes()
        if digest(receipt) != receipt_hash or digest(stream) != stream_hash:
            raise ValueError(f'{key}: receipt or stream hash mismatch')
        record = json.loads(receipt)
        if record.get('status') != 'matched' or record.get('host_exit') != 0 or len(stream) != size:
            raise ValueError(f'{key}: unmatched receipt, host exit, or stream size')
        streams[key] = stream
        provenance[key] = {'receipt': f'{folder}/result.json', 'receipt_sha256': receipt_hash,
                           'host_stream': f'{folder}/host-stdout.bin', 'host_stream_sha256': stream_hash}
    return streams, provenance

def compose(streams, provenance):
    a, n, i = (streams[k] for k in ('average', 'normal', 'indices'))
    if (len(a), len(n), len(i)) != (704, 1552, 3036):
        raise ValueError('unexpected stream lengths')
    positions = [struct.unpack_from('<3f', a, j * 16 + 4) for j in range(44)]
    normals = [struct.unpack_from('<3f', n, j * 12) for j in range(44)]
    if not all(math.isfinite(x) for vectors in (positions, normals) for v in vectors for x in v):
        raise ValueError('nonfinite coordinate or normal')
    count, = struct.unpack_from('<I', i, 2024)
    if count != 252:
        raise ValueError('unexpected index count')
    indices = struct.unpack_from('<252I', i, 2028)
    if any(x >= len(positions) for x in indices):
        raise ValueError('index out of range')
    return {'schema': 'rustspore-conditional-0473-partial-geometry-v1', 'limits': LIMITS,
            'provenance': provenance, 'positions': positions, 'normals': normals,
            'triangles': [indices[j:j+3] for j in range(0, count, 3)]}

def parse_initial_records(artifact, data):
    if len(data) != 880 or len(artifact['positions']) != 44:
        raise ValueError('unexpected record or position count')
    records = []
    for j in range(44):
        chunk = data[j * 20:(j + 1) * 20]
        words = struct.unpack_from('<4I', chunk, 4)
        if not all(math.isfinite(struct.unpack('<f', struct.pack('<I', w))[0]) for w in words):
            raise ValueError('nonfinite initial record weight')
        records.append({'raw_index_bytes': list(chunk[:4]),
                        'raw_weight_words_hex': [f'{w:08x}' for w in words]})
    return records

def attach_synthetic_records(artifact, streams, provenance):
    records = parse_initial_records(artifact, streams['synthetic_initial_records'])
    artifact['conditional_synthetic_coordinate_initial_records'] = {
        'limits': RECORD_LIMITS, 'provenance': provenance['synthetic_initial_records'],
        'records_by_position': records}
    return artifact

def read_row_records(target):
    directory = target / ROW_FOLDER
    receipt = (directory / 'result.json').read_bytes()
    data = (directory / 'record-stdout.bin').read_bytes()
    if digest(receipt) != ROW_RECEIPT_SHA or digest(data) != ROW_STREAM_SHA:
        raise ValueError('row-coordinate receipt or stream hash mismatch')
    record = json.loads(receipt)
    if record.get('status') != 'matched' or any(record.get(k) != 0 for k in ('model_exit', 'accum_exit', 'rank_exit', 'record_exit')) or len(data) != 880:
        raise ValueError('row-coordinate comparison or stream invalid')
    provenance = {'receipt': f'{ROW_FOLDER}/result.json', 'receipt_sha256': ROW_RECEIPT_SHA,
                  'record_stream': f'{ROW_FOLDER}/record-stdout.bin', 'record_stream_sha256': ROW_STREAM_SHA}
    return data, provenance

def attach_row_records(artifact, data, provenance):
    records = parse_initial_records(artifact, data)
    artifact['conditional_row_coordinate_initial_records'] = {
        'limits': ROW_LIMITS, 'provenance': provenance, 'records_by_position': records}
    return artifact

def read_smoothing_fixture(target, folder, receipt_sha, stream_sha):
    directory = target / folder
    receipt = (directory / 'result.json').read_bytes()
    native = (directory / 'host-stdout.bin').read_bytes()
    model = (directory / 'model-stdout.bin').read_bytes()
    if digest(receipt) != receipt_sha or digest(native) != stream_sha or digest(model) != stream_sha:
        raise ValueError('smoothing receipt or stream hash mismatch')
    result = json.loads(receipt)
    if (result.get('status') != 'matched' or result.get('host_exit') != 0 or result.get('model_exit') != 0
            or len(native) != 2640 or native != model):
        raise ValueError('smoothing comparison or stream invalid')
    return native, {'receipt': f'{folder}/result.json', 'receipt_sha256': receipt_sha,
                    'native_stream': f'{folder}/host-stdout.bin', 'model_stream': f'{folder}/model-stdout.bin',
                    'stream_sha256': stream_sha}

def read_smoothed_records(target):
    return read_smoothing_fixture(target, SMOOTH_FOLDER, SMOOTH_RECEIPT_SHA, SMOOTH_STREAM_SHA)

def read_caller_smoothed_records(target):
    return read_smoothing_fixture(target, CALLER_SMOOTH_FOLDER, CALLER_SMOOTH_RECEIPT_SHA, CALLER_SMOOTH_STREAM_SHA)

def attach_smoothed_records(artifact, data, provenance, passes):
    if type(passes) is not int or passes not in (0, 1, 2) or len(data) != 2640:
        raise ValueError('invalid smoothing pass count or stream length')
    records = parse_initial_records(artifact, data[passes * 880:(passes + 1) * 880])
    artifact['conditional_half_alpha_smoothed_records'] = {
        'limits': SMOOTH_LIMITS, 'explicit_passes': passes, 'alpha_f32_bits_hex': '3f000000',
        'provenance': provenance, 'records_by_position': records}
    return artifact

def attach_caller_smoothed_records(artifact, data, provenance, passes):
    if type(passes) is not int or passes not in (0, 1, 2) or len(data) != 2640:
        raise ValueError('invalid smoothing pass count or stream length')
    records = parse_initial_records(artifact, data[passes * 880:(passes + 1) * 880])
    artifact['conditional_caller_alpha_smoothed_records'] = {
        'limits': ('Frozen-row conditional inputs; 0x3f333333 is the source caller factor, '
                   'but pass count is explicitly supplied and active configuration is unobserved. '
                   'Raw index bytes are not established palette identities; no live identity or final weights claim.'),
        'explicit_passes': passes, 'alpha_f32_bits_hex': '3f333333',
        'provenance': provenance, 'records_by_position': records}
    return artifact

def read_synthetic_corner_attributes(target):
    directory = target / CORNER_FOLDER
    receipt = (directory / 'result.json').read_bytes()
    data = (directory / 'host-stdout.bin').read_bytes()
    if digest(receipt) != CORNER_RECEIPT_SHA or digest(data) != CORNER_STREAM_SHA:
        raise ValueError('corner receipt or stream hash mismatch')
    result = json.loads(receipt)
    if result.get('status') != 'matched' or result.get('host_exit') != 0 or len(data) != 3168:
        raise ValueError('corner comparison or stream invalid')
    provenance = {'receipt': f'{CORNER_FOLDER}/result.json', 'receipt_sha256': CORNER_RECEIPT_SHA,
                  'native_stream': f'{CORNER_FOLDER}/host-stdout.bin', 'native_stream_sha256': CORNER_STREAM_SHA}
    return data, provenance

def attach_synthetic_corner_attributes(artifact, data, provenance, records, passes):
    if type(passes) is not int or passes not in (0, 1, 2) or len(data) != 3168 or len(records) != 2640 or len(artifact['positions']) != 44:
        raise ValueError('invalid corner input or explicit pass count')
    corners = []
    for j in range(44):
        index = (j * 17) % 44
        chunk = data[(passes*44+j)*24:(passes*44+j+1)*24]
        raw = records[passes*880+index*20:passes*880+(index+1)*20]
        offsets = struct.unpack_from('<4H', chunk)
        words = struct.unpack_from('<4I', chunk, 8)
        if offsets != tuple(x*3 for x in raw[:4]) or chunk[8:] != raw[4:]:
            raise ValueError('corner output does not match supplied record')
        if not all(math.isfinite(struct.unpack('<f', struct.pack('<I', w))[0]) for w in words):
            raise ValueError('nonfinite corner weight')
        corners.append({'synthetic_corner_ordinal': j, 'position_index': index,
                        'raw_u16_offsets': list(offsets), 'raw_weight_words_hex': [f'{w:08x}' for w in words]})
    artifact['conditional_synthetic_corner_attributes'] = {
        'limits': ('Synthetic first corner indices (ordinal times 17 modulo 44), caller-alpha matched records '
                   'and explicit pass count; these are not observed source corner triples, live palette identities '
                   'or a finalized mesh.'),
        'explicit_passes': passes, 'provenance': provenance, 'corners': corners}
    return artifact

MULTI_FOLDER = 'creature-skin-multigroup-rust-verify-20261109'
MULTI_MODEL_FOLDER = 'creature-skin-multigroup-0473-model-v2-20261109'
MULTI_NATIVE_FOLDER = 'creature-skin-outer-group-0473-prep-20261109'
MULTI_DIRECT_FOLDER = 'creature-skin-multigroup-native-rust-verify-20261109'
MULTI_NATIVE_RECEIPT_SHA = '4492803315018f5819322ba23f523ee1d6f11c87e0637748340a49e5b99a8775'
MULTI_DIRECT_RECEIPT_SHA = 'c9dcc5cfb12dccf5627335fcfedadb77faa7764c977ee828aedcd58868a26eb8'
MULTI_INPUT_FOLDER = 'creature-skin-single-group-0473-prep-v2-20261109'
MULTI_RECEIPT_SHA = '1e593a044be9842d734576b78d9373cb832477baf18741ea460e108cc33a7728'
MULTI_STREAM_SHA = '3350ca9dabb564c6cbab49153f4d6eb8073cd32d963319478f00ffcf0bc8519f'
MULTI_INPUT_PINS = {
 'positions.bin':'6240f2d95681ddd6cccb17679b12cf528c15710c6cdb351160aeb56e7656ee4f',
 'indices.bin':'01111d5dd13ff186458dcff418e4186c1d59d8d163d5ca50af7bba7db05b9028',
 'links.bin':'f83f43ff2e5b3a7ba267ba3212b879bf393fe20ecf5bd8ee6cc4f0784a2658bf',
 'orientations.bin':'49bdb28a2e3fcdd9ed42f46e525fbf07f4799bdf9445aedd824842e1c3d399cb'}
MULTI_LIMITS = ('Model-derived multi-group UVs and assignment from conditional frozen 0473 partial '
 'geometry; original outer and UV/edge instructions matched under supplied orientations, four synthetic '
 'preexisting groups, zero face states, sentinel cache and successful preallocated reset, queue, '
 'allocation, clone and append stubs. Native allocator and lifecycle behavior, actual UV or group '
 'ownership, live identity and material/palette binding are not established.')

def read_multigroup(target):
    folder=target/MULTI_FOLDER; receipt=(folder/'result.json').read_bytes(); rust=(folder/'rust-stdout.bin').read_bytes()
    model=(target/MULTI_MODEL_FOLDER/'model-only.bin').read_bytes()
    native_receipt=(target/MULTI_NATIVE_FOLDER/'result.json').read_bytes()
    native=(target/MULTI_NATIVE_FOLDER/'host-stdout.bin').read_bytes()
    direct_receipt=(target/MULTI_DIRECT_FOLDER/'result.json').read_bytes()
    if (digest(receipt)!=MULTI_RECEIPT_SHA or digest(rust)!=MULTI_STREAM_SHA or digest(model)!=MULTI_STREAM_SHA
            or digest(native_receipt)!=MULTI_NATIVE_RECEIPT_SHA or digest(native)!=MULTI_STREAM_SHA
            or digest(direct_receipt)!=MULTI_DIRECT_RECEIPT_SHA):
        raise ValueError('multi-group receipt or stream hash mismatch')
    result=json.loads(receipt)
    if result.get('status')!='matched' or result.get('rust_exit')!=0 or result.get('bytes')!=3696 or rust!=model:
        raise ValueError('multi-group comparison invalid')
    native_result=json.loads(native_receipt);direct_result=json.loads(direct_receipt)
    if (native_result.get('status')!='matched' or native_result.get('host_exit')!=0
            or native_result.get('host_timed_out') is not False or native!=rust
            or direct_result.get('status')!='matched' or direct_result.get('compared_bytes')!=3696
            or direct_result.get('native_receipt_sha256')!=MULTI_NATIVE_RECEIPT_SHA
            or direct_result.get('rust_receipt_sha256')!=MULTI_RECEIPT_SHA
            or direct_result.get('native_stream_sha256')!=MULTI_STREAM_SHA
            or direct_result.get('rust_stream_sha256')!=MULTI_STREAM_SHA):
        raise ValueError('multi-group direct native comparison invalid')
    if result.get('input_pins')!=MULTI_INPUT_PINS:
        raise ValueError('multi-group input provenance mismatch')
    inputs={name:(target/MULTI_INPUT_FOLDER/name).read_bytes() for name in MULTI_INPUT_PINS}
    if any(digest(inputs[name])!=want for name,want in MULTI_INPUT_PINS.items()):
        raise ValueError('multi-group input hash mismatch')
    provenance={'receipt':f'{MULTI_FOLDER}/result.json','receipt_sha256':MULTI_RECEIPT_SHA,
      'rust_stream':f'{MULTI_FOLDER}/rust-stdout.bin','model_stream':f'{MULTI_MODEL_FOLDER}/model-only.bin',
      'stream_sha256':MULTI_STREAM_SHA,'input_folder':MULTI_INPUT_FOLDER,'input_sha256':MULTI_INPUT_PINS.copy(),
      'native_receipt':f'{MULTI_NATIVE_FOLDER}/result.json','native_receipt_sha256':MULTI_NATIVE_RECEIPT_SHA,
      'native_stream':f'{MULTI_NATIVE_FOLDER}/host-stdout.bin','native_stream_sha256':MULTI_STREAM_SHA,
      'direct_comparison_receipt':f'{MULTI_DIRECT_FOLDER}/result.json','direct_comparison_receipt_sha256':MULTI_DIRECT_RECEIPT_SHA}
    return rust,inputs,provenance

def attach_multigroup(artifact,data,inputs,provenance):
    if len(data)!=3696 or len(artifact['positions'])!=44 or len(artifact['triangles'])!=84:
        raise ValueError('invalid multi-group geometry or stream length')
    positions=b''.join(struct.pack('<3f',*x) for x in artifact['positions'])
    indices=b''.join(struct.pack('<3I',*x) for x in artifact['triangles'])
    if positions!=inputs.get('positions.bin') or indices!=inputs.get('indices.bin'):
        raise ValueError('multi-group input geometry does not match artifact')
    words=struct.unpack('<924I',data);cursor=0
    def take(n):
        nonlocal cursor
        if n<0 or cursor+n>len(words):raise ValueError('truncated multi-group stream')
        result=words[cursor:cursor+n];cursor+=n;return result
    if take(4)!=(8,85,44,252):raise ValueError('unexpected multi-group counts')
    groups=[]; seen=[];previous_end=0
    for group in range(4,12):
        gid,seed,axis,uv_end,nfaces=take(5);bounds=take(4);faces=take(nfaces)
        if gid!=group or seed>=84 or axis>=6 or not 1<=nfaces<=84 or not previous_end<uv_end<=85 or seed not in faces:
            raise ValueError('invalid multi-group record')
        if any(face>=84 for face in faces):raise ValueError('multi-group face out of range')
        if not all(math.isfinite(struct.unpack('<f',struct.pack('<I',x))[0]) for x in bounds):raise ValueError('nonfinite group bounds')
        groups.append({'model_group_id':gid,'seed_face':seed,'axis':axis,'uv_end':uv_end,
                       'bounds_words_hex':[f'{x:08x}' for x in bounds],'faces':list(faces)})
        seen.extend(faces);previous_end=uv_end
    if sorted(seen)!=list(range(84)) or previous_end!=85:raise ValueError('multi-group face coverage invalid')
    uv=[take(2) for _ in range(85)];metadata=[take(2) for _ in range(85)];cache=[take(2) for _ in range(44)]
    corners=take(252);states=take(84)
    if cursor!=len(words):raise ValueError('unexpected multi-group trailing words')
    if any(not math.isfinite(struct.unpack('<f',struct.pack('<I',x))[0]) for pair in uv for x in pair):raise ValueError('nonfinite model UV')
    if any(g<4 or g>=12 or c>=252 for g,c in metadata):raise ValueError('invalid UV metadata')
    start=0
    for group in groups:
        end=group['uv_end']
        if any(g!=group['model_group_id'] for g,_ in metadata[start:end]):raise ValueError('invalid UV group range')
        start=end
    if any(g<4 or g>=12 or i>=85 or metadata[i][0]!=g for g,i in cache):raise ValueError('invalid UV cache')
    if any(x>=85 for x in corners):raise ValueError('corner UV index out of range')
    for group in groups:
        gid=group['model_group_id']
        for face in group['faces']:
            if states[face]!=(~gid)&0xffffffff or any(metadata[corners[3*face+k]][0]!=gid for k in range(3)):
                raise ValueError('inconsistent model group or corner ownership')
    section={'limits':MULTI_LIMITS,'provenance':provenance,'initial_group_count':4,'groups':groups,
      'uv_words_hex':[[f'{x:08x}' for x in pair] for pair in uv],
      'uv_metadata':[{'model_group_id':g,'conditional_corner_ordinal':c} for g,c in metadata],
      'corner_uv_indices':list(corners)}
    artifact['conditional_model_only_uv_groups']=section
    return artifact


JOIN_NATIVE_FOLDER = 'creature-skin-uv-offset-join-0473-prep-20261109'
JOIN_RUST_FOLDER = 'creature-skin-uv-offset-composed-rust-verify-20261109'
JOIN_NATIVE_RECEIPT_SHA = 'a0744caa2abc8804a5876dc4edc908c24a79324aed62e1df3101e0cf8266e191'
JOIN_RUST_RECEIPT_SHA = '582a43bf1de238e865679ed0afc8821837833f584561598ad5d3dca6a02039a9'
JOIN_CASES_SHA = 'b20244f4e86f4fc4d64156a16d48a0d08ff2dcfbd89f1df54c6c5b84b2645099'
JOIN_STREAM_SHA = '35de09a8231fd837c04dc54f6d8c3236efe12fec4a17a2a63228855c251723e6'
JOIN_INPUTS = {
 'atlas': ('creature-skin-atlas-bounds-0473-prep-20261109','0f3dc8289c2c561a88668aee3c76cf0bb6c5283a79889741eca4b354dc479357','6b48bbf0e0d730f619dc18821b425014667daf0a0611f27539c862dd48db7b68',264),
 'shelf': ('creature-skin-shelf-select-0473-prep-20261109','bf8c775b3d2430a8c2589c25d915ee496f62dce0278b4ef4b93cdacc42e8b56c','a95f4bdf0b64fb00bbc6d3049585bf06df5bceb4e48fdd9417faec30f2ddcb5a',1544),
 'sort': ('creature-skin-atlas-sort-0473-prep-20261109','3cf0063b51449a543c31da90543804b1e9b6d168b4f89b9007ee781c2949156f','e342a1dc12346b77949bbaa6569b8235d69def6faa25523f76dec7a76230d220',192),
 'offset': ('creature-skin-atlas-offset-0473-prep-20261109','9a3860258c6ad764d5535ef7a32d298ac51b927c911a7f3f84ad77791f02fccc','fbaeff3b4ece1f30344ae01ba82b3780917e07fe332d8ddd4234c87dc0f3402b',264),
}
JOIN_LIMITS = ('Conditional model-derived UV transformation matched against original source instructions. '
 'Row x is model-derived; row y and shelf shifts are synthetic, and scale 0.5 is synthetic. '
 'Four earlier groups, actual atlas placement/scale, native allocation/lifecycle, texture/material '
 'binding and live creature identity remain unverified. This is inspection data, not final UVs.')

def read_transformed_uvs(target):
    sources, provenance = {}, {}
    for key,(folder,rh,sh,size) in JOIN_INPUTS.items():
        receipt=(target/folder/'result.json').read_bytes();stream=(target/folder/'host-stdout.bin').read_bytes()
        if digest(receipt)!=rh or digest(stream)!=sh:raise ValueError(f'{key}: receipt or stream hash mismatch')
        record=json.loads(receipt)
        if record.get('status')!='matched' or record.get('host_exit')!=0 or record.get('host_timed_out') is not False or len(stream)!=size:
            raise ValueError(f'{key}: native comparison invalid')
        sources[key]=stream;provenance[key]={'receipt':f'{folder}/result.json','receipt_sha256':rh,'native_stream':f'{folder}/host-stdout.bin','native_stream_sha256':sh}
    native=target/JOIN_NATIVE_FOLDER;rust=target/JOIN_RUST_FOLDER
    nr=(native/'result.json').read_bytes();ns=(native/'host-stdout.bin').read_bytes();nc=(native/'cases.bin').read_bytes()
    rr=(rust/'result.json').read_bytes();rs=(rust/'uv-stdout.bin').read_bytes();rc=(rust/'joined-cases.bin').read_bytes();ro=(rust/'offset-stdout.bin').read_bytes()
    if (digest(nr)!=JOIN_NATIVE_RECEIPT_SHA or digest(rr)!=JOIN_RUST_RECEIPT_SHA or digest(ns)!=JOIN_STREAM_SHA
        or digest(rs)!=JOIN_STREAM_SHA or digest(nc)!=JOIN_CASES_SHA or digest(rc)!=JOIN_CASES_SHA
        or digest(ro)!=JOIN_INPUTS['offset'][2]):raise ValueError('joined UV receipt or stream hash mismatch')
    n=json.loads(nr);r=json.loads(rr)
    if (n.get('status')!='matched' or n.get('host_exit')!=0 or n.get('host_timed_out') is not False
        or r.get('status')!='matched' or r.get('offset_exit')!=0 or r.get('uv_exit')!=0
        or r.get('uv_count')!=85 or r.get('control_count')!=3 or r.get('native_receipt_sha256')!=JOIN_NATIVE_RECEIPT_SHA
        or r.get('offset_stream_sha256')!=JOIN_INPUTS['offset'][2] or r.get('joined_cases_sha256')!=JOIN_CASES_SHA
        or r.get('uv_stream_sha256')!=JOIN_STREAM_SHA or ns!=rs or nc!=rc or ro!=sources['offset']
        or len(ns)!=704 or len(nc)!=2816):raise ValueError('joined UV comparison invalid')
    provenance['joined']={'native_receipt':f'{JOIN_NATIVE_FOLDER}/result.json','native_receipt_sha256':JOIN_NATIVE_RECEIPT_SHA,
      'native_stream':f'{JOIN_NATIVE_FOLDER}/host-stdout.bin','rust_receipt':f'{JOIN_RUST_FOLDER}/result.json',
      'rust_receipt_sha256':JOIN_RUST_RECEIPT_SHA,'rust_stream':f'{JOIN_RUST_FOLDER}/uv-stdout.bin',
      'stream_sha256':JOIN_STREAM_SHA,'joined_cases_sha256':JOIN_CASES_SHA}
    return ns,nc,sources,provenance

def attach_transformed_uvs(artifact,data,cases,sources,provenance, *, scale=0x3f000000, output_key="conditional_model_only_transformed_uvs", limits=JOIN_LIMITS, scale_key="synthetic_scale_f32_bits_hex"):
    section=artifact.get('conditional_model_only_uv_groups')
    if not isinstance(section,dict) or len(data)!=704 or len(cases)!=2816:
        raise ValueError('missing conditional groups or invalid joined UV stream')
    if len(section.get('uv_words_hex',[]))!=85 or len(section.get('uv_metadata',[]))!=85 or len(section.get('corner_uv_indices',[]))!=252:
        raise ValueError('invalid conditional UV/group shape')
    atlas=sources.get('atlas',b'');shelf=sources.get('shelf',b'');sort=sources.get('sort',b'');offset=sources.get('offset',b'')
    if (len(atlas),len(shelf),len(sort),len(offset))!=(264,1544,192,264):raise ValueError('invalid UV join inputs')
    bounds=list(struct.iter_unpack('<6I',atlas))[:8];s=struct.unpack('<386I',shelf);order=struct.unpack('<48I',sort)[:8];offsets=list(struct.iter_unpack('<6I',offset))[:8]
    if s[0]!=4 or sorted(order)!=list(range(8)):raise ValueError('invalid shelf or sort order')
    by_group={};k=0
    for row in range(4):
        record=s[13+15*row:13+15*(row+1)];count=record[0]
        if not 1<=count<=8:raise ValueError('invalid shelf group count')
        for member in record[3:3+count]:
            if member>=8 or k>=8:raise ValueError('invalid shelf member')
            gid=4+order[member]
            if gid in by_group:raise ValueError('duplicate shelf group')
            by_group[gid]=offsets[k][2:4];k+=1
    if k!=8 or len(by_group)!=8:raise ValueError('incomplete shelf group map')
    rows=list(struct.iter_unpack('<8I',cases));values=list(struct.iter_unpack('<2I',data))
    if any(not math.isfinite(struct.unpack('<f',struct.pack('<I',x))[0]) for pair in values for x in pair):
        raise ValueError('nonfinite transformed UV')
    for i in range(85):
        meta=section['uv_metadata'][i]
        if not isinstance(meta,dict):raise ValueError('invalid UV metadata')
        gid=meta.get('model_group_id');row=rows[i]
        if type(gid) is not int or gid not in by_group:raise ValueError('invalid UV group id')
        group=bounds[gid-4]
        try:original=tuple(int(x,16) for x in section['uv_words_hex'][i])
        except (ValueError,TypeError):raise ValueError('invalid original UV words')
        if (len(original)!=2 or row[:2]!=original or row[2:5]!=(group[0],group[1],group[4])
                or row[5:7]!=by_group[gid] or row[7]!=scale):raise ValueError('joined UV inputs do not match group')
    corners=section['corner_uv_indices']
    if any(type(x) is not int or not 0<=x<85 for x in corners):raise ValueError('invalid transformed corner indices')
    artifact[output_key]={'limits':limits,'provenance':provenance,
      scale_key:f'{scale:08x}',
      'uv':[[struct.unpack('<f',struct.pack('<I',x))[0] for x in pair] for pair in values[:85]],
      'uv_words_hex':[[f'{x:08x}' for x in pair] for pair in values[:85]],
      'corner_uv_indices':list(corners)}
    return artifact

PAIRED_NATIVE = 'creature-skin-uv-paired-join-0473-prep-20261109'
PAIRED_RUST = 'creature-skin-uv-paired-composed-rust-verify-20261109'
PAIRED_INPUTS = {
 'atlas': JOIN_INPUTS['atlas'], 'sort': JOIN_INPUTS['sort'],
 'pair': ('creature-skin-atlas-pair-scale-0473-prep-20261109','b4eedac21de34f071026bd8293f4dd346d141ece708592ace65dab63dc3770ed','571a4e967cc2417165010ce9367d8f3fe47f51ba55824a629b0a7f69eb3b1ea3',2340),
 'offset': ('creature-skin-paired-offset-0473-prep-20261109','b09f431223a97a1ad13dd519f28796c5e5793d37f3b685587ad9046e077e2bb4','c47c384cca54407be5baf6d8cf9908a1955cec14440b54edab4a2ea04fc120b6',192),
}
PAIRED_RECEIPTS = ('88ba87a0a4599fb5e8f3f747e2ac192dc19566d0709d6e57e79fa62db2a6a873','8aa327f4d1d2e7b201ea50e64f34cb954c4130e6c41a6df0252affdbf9f41dda')
PAIRED_CASE_SHA = 'a7545074023e8399b77a83cc34ac2588a4a41c1dc4dacab99e3634c0c570e9bb'
PAIRED_STREAM_SHA = '71cd4b1a08a5d432108a6b6b0aca74047c9ec906aa4b18f3bdc3643eb0b38c09'
PAIRED_LIMITS = ('Conditional native-matched paired shelf offsets and source scale for supplied synthetic shelf order and zero-origin row scheduling. Four earlier groups, upstream sqrt and selection, allocation/lifecycle, actual atlas placement, texture/material binding and live creature identity remain unverified. This is inspection data, not final UVs.')

def read_paired_uvs(target):
    sources={}; provenance={}
    for key,(folder,rh,sh,size) in PAIRED_INPUTS.items():
        receipt=(target/folder/'result.json').read_bytes(); stream=(target/folder/'host-stdout.bin').read_bytes()
        if digest(receipt)!=rh or digest(stream)!=sh: raise ValueError(f'{key}: receipt or stream hash mismatch')
        r=json.loads(receipt)
        if r.get('status')!='matched' or r.get('host_exit')!=0 or r.get('host_timed_out') is not False or len(stream)!=size: raise ValueError(f'{key}: native comparison invalid')
        sources[key]=stream; provenance[key]={'receipt':f'{folder}/result.json','receipt_sha256':rh,'native_stream':f'{folder}/host-stdout.bin','native_stream_sha256':sh}
    native=target/PAIRED_NATIVE;rust=target/PAIRED_RUST
    nr=(native/'result.json').read_bytes();rr=(rust/'result.json').read_bytes()
    ns=(native/'host-stdout.bin').read_bytes();rs=(rust/'uv-stdout.bin').read_bytes()
    nc=(native/'cases.bin').read_bytes();rc=(rust/'joined-cases.bin').read_bytes()
    rp=(rust/'pair-stdout.bin').read_bytes();ro=(rust/'offset-stdout.bin').read_bytes()
    inp=(target/PAIRED_INPUTS['offset'][0]/'cases.bin').read_bytes()
    if (digest(nr)!=PAIRED_RECEIPTS[0] or digest(rr)!=PAIRED_RECEIPTS[1] or digest(ns)!=PAIRED_STREAM_SHA or digest(rs)!=PAIRED_STREAM_SHA
        or digest(nc)!=PAIRED_CASE_SHA or digest(rc)!=PAIRED_CASE_SHA or rp!=sources['pair'] or ro!=sources['offset']
        or digest(inp)!='c77e90717be45554ca39057b279e8006eb6038d4d202c1dfbc97238b51bef23f'):
        raise ValueError('paired UV receipt or stream hash mismatch')
    n=json.loads(nr);r=json.loads(rr)
    if (n.get('status')!='matched' or n.get('host_exit')!=0 or n.get('host_timed_out') is not False or r.get('status')!='matched'
        or r.get('uv_count')!=85 or r.get('control_count')!=3 or r.get('scale')!='3ee3db63'
        or r.get('native_receipt_sha256')!=PAIRED_RECEIPTS[0] or r.get('pair_sha256')!=PAIRED_INPUTS['pair'][2]
        or r.get('offset_sha256')!=PAIRED_INPUTS['offset'][2] or r.get('cases_sha256')!=PAIRED_CASE_SHA or r.get('uv_sha256')!=PAIRED_STREAM_SHA
        or ns!=rs or nc!=rc or len(ns)!=704 or len(nc)!=2816 or len(inp)!=256): raise ValueError('paired UV comparison invalid')
    sources['offset_inputs']=inp
    provenance['joined']={'native_receipt':f'{PAIRED_NATIVE}/result.json','native_receipt_sha256':PAIRED_RECEIPTS[0],
      'rust_receipt':f'{PAIRED_RUST}/result.json','rust_receipt_sha256':PAIRED_RECEIPTS[1],
      'stream_sha256':PAIRED_STREAM_SHA,'joined_cases_sha256':PAIRED_CASE_SHA}
    return ns,nc,sources,provenance

def attach_paired_uvs(artifact,data,cases,sources,provenance):
    if tuple(len(sources.get(k,b'')) for k in ('atlas','sort','pair','offset','offset_inputs')) != (264,192,2340,192,256): raise ValueError('invalid paired join inputs')
    bounds=list(struct.iter_unpack('<6I',sources['atlas']))[:8];order=struct.unpack('<48I',sources['sort'])[:8]
    pw=struct.unpack('<585I',sources['pair'])[:195]; offsets=list(struct.iter_unpack('<6I',sources['offset'])); inputs=list(struct.iter_unpack('<8I',sources['offset_inputs']))
    if sorted(order)!=list(range(8)) or pw[2]!=0x3ee3db63: raise ValueError('invalid paired order or scale')
    shelf=[0]*386;shelf[0]=4;k=0;seen=set()
    for row in range(4):
        rec=pw[3+16*row:3+16*(row+1)];count,_,height,shift=rec[:4]
        if not 1<=count<=8: raise ValueError('invalid paired count')
        shelf[13+15*row]=count
        for j,member in enumerate(rec[4:4+count]):
            if member>=8 or k>=8 or member in seen: raise ValueError('invalid paired member')
            seen.add(member);gid=4+order[member]
            if inputs[k]!=(*bounds[gid-4][:4],height,shift,row,int(j+1==count)): raise ValueError('paired offset inputs mismatch')
            shelf[13+15*row+3+j]=member;k+=1
    if k!=8: raise ValueError('incomplete paired groups')
    derived={'atlas':sources['atlas'],'sort':sources['sort'],'shelf':struct.pack('<386I',*shelf),'offset':sources['offset']+bytes(72)}
    return attach_transformed_uvs(artifact,data,cases,derived,provenance,scale=pw[2],output_key='conditional_paired_transformed_uvs',limits=PAIRED_LIMITS,scale_key='conditional_source_scale_f32_bits_hex')

TWELVE_NATIVE='creature-skin-uv-paired-12group-0473-prep-20261109'
TWELVE_RUST='creature-skin-uv-paired-12group-rust-verify-20261109'
TWELVE_INPUTS={
 'atlas':('creature-skin-atlas-bounds-12group-0473-prep-20261109','ec4f90b0c623ec5430798f2329527e581593f648d740c2e1061fb5f580ec3641','73b947945227e84f585bc5d1e415f6424388ad55869737ffa2c779478912200c',288),
 'sort':('creature-skin-atlas-sort-12group-0473-prep-20261109','3b01b1879bc5a89b0b59f02203fcfa880486c1d4e412635b1a098163ce126394','68e30a1bd12c3fcb33f2a3064c428b78506aa345c7889c99f0ba006eaf361cd1',192),
 'pair':('creature-skin-atlas-pair-scale-12group-0473-prep-20261109','9dbcd0d187cec05f2fd4a6fe62bbdad4d04bb3e6a76e891a9d35a3177685afbd','1f42bb891da2ff2a1d0245650ed16dd085d0309e1008e3c1d053966b9d73f16f',2340),
 'offset':('creature-skin-paired-offset-12group-0473-prep-20261109','dbe4c9ff6b85f619bf65e1e19f161f23a182b155a6284c68f403ec0a13d3672e','1671f9dcee9c1e0bef1089eca5e06079bce74d002b31a01bc87ffe0ee53c5269',288),
 'rows':('creature-skin-row-groups-0473-prep-v2-20261109','2dd6698805542769411dd1c35c5c623298e469ca6e305ee7713e4dcb7a475d61','7ca40ed9208a7b0aaa13f3cfa1ecceb9f3b88f953c1e3ec8fda9d8cfd14aa657',1192),
 'sqrt':('creature-skin-atlas-sqrt-12group-0473-prep-20261109','6e10edfd6bdfd44f26b9cb43c22f73787a82797200cff94fc63b634cf2b83d58','8a993f964fe880654f8dda17d4c0c06e3a3078d4ecc5c07bb988a906fa872f6e',1040),
 'shelf':('creature-skin-shelf-select-12group-0473-prep-20261109','39c16fa1472ff73ebab01e3f53cb56b6d48a544949beddc239dd893693a7bcaa','4fe6f6bcb8fcdfbfba462732ae414b836447c953234b57106e78f8061e0f093b',1544),
}
TWELVE_CASE_SHA='0dac10115f999614dc244b0778a09e4f9d2b3289dba1755d55ae109a19813340'
TWELVE_STREAM_SHA='bb946f07d8e1ca05fed4516b960d48a5e77ab9f48276c8c8964766c5b58879d8'
TWELVE_RECEIPTS=('b7763cea0d2290c090eb26f7be1c88027cf3362566d0b94b02b24b3bcb52b940','4c138be42db9db585be1a790630d0d5b602017fb785442c2b4b6a86355cc9e3a')
TWELVE_LIMITS=('Conditional twelve-group source-instruction and Rust matched UV inspection data. Four initial groups use frozen 0473 row assumptions; eight generated groups, zero-origin wrapper scheduling, preallocated lifecycle stubs and successful same-object operations are conditional. Actual live atlas, texture/material binding and creature identity remain unverified. This is not final UVs.')

def read_twelve_uvs(target):
    sources={};provenance={}
    for key,(folder,rh,sh,size) in TWELVE_INPUTS.items():
        receipt=(target/folder/'result.json').read_bytes();stream=(target/folder/'host-stdout.bin').read_bytes()
        if digest(receipt)!=rh or digest(stream)!=sh:raise ValueError(f'{key}: receipt or stream hash mismatch')
        q=json.loads(receipt)
        if q.get('status')!='matched' or q.get('host_exit')!=0 or q.get('host_timed_out') is not False or len(stream)!=size:raise ValueError(f'{key}: native comparison invalid')
        sources[key]=stream;provenance[key]={'receipt':f'{folder}/result.json','receipt_sha256':rh,'stream':f'{folder}/host-stdout.bin','stream_sha256':sh}
    for key,sha in [('offset','dc2947fa95f57326346282288c456b20d50156bd5af5ddb6964e985b13506a50'),('atlas','53123fea428c3d9ad0662a919ddf9c595bd8c4cfb662a39a58f467ef8211d469'),('shelf','90caa06a08c212eda110a322e5e92ad504cbea83f5358e1aa5f725dc38513b34')]:
        blob=(target/TWELVE_INPUTS[key][0]/'cases.bin').read_bytes()
        if digest(blob)!=sha:raise ValueError(f'{key}: cases hash mismatch')
        sources[key+'_inputs']=blob
    native=target/TWELVE_NATIVE;rust=target/TWELVE_RUST
    nr=(native/'result.json').read_bytes();rr=(rust/'result.json').read_bytes();ns=(native/'host-stdout.bin').read_bytes();rs=(rust/'uv-stdout.bin').read_bytes();nc=(native/'cases.bin').read_bytes();rc=(rust/'joined-cases.bin').read_bytes();rp=(rust/'pair-stdout.bin').read_bytes();ro=(rust/'offset-stdout.bin').read_bytes()
    if (digest(nr)!=TWELVE_RECEIPTS[0] or digest(rr)!=TWELVE_RECEIPTS[1] or digest(ns)!=TWELVE_STREAM_SHA or digest(rs)!=TWELVE_STREAM_SHA or digest(nc)!=TWELVE_CASE_SHA or digest(rc)!=TWELVE_CASE_SHA or rp!=sources['pair'] or ro!=sources['offset']):raise ValueError('twelve UV receipt or stream hash mismatch')
    n=json.loads(nr);q=json.loads(rr)
    if (n.get('status')!='matched' or n.get('host_exit')!=0 or n.get('host_timed_out') is not False or q.get('status')!='matched' or q.get('uv_count')!=85 or q.get('control_count')!=3 or q.get('scale')!='3f2c2996' or q.get('native_receipt_sha256')!=TWELVE_RECEIPTS[0] or q.get('pair_sha256')!=TWELVE_INPUTS['pair'][2] or q.get('offset_sha256')!=TWELVE_INPUTS['offset'][2] or q.get('cases_sha256')!=TWELVE_CASE_SHA or q.get('uv_sha256')!=TWELVE_STREAM_SHA or ns!=rs or nc!=rc or len(ns)!=704 or len(nc)!=2816):raise ValueError('twelve UV comparison invalid')
    provenance['joined']={'native_receipt':f'{TWELVE_NATIVE}/result.json','native_receipt_sha256':TWELVE_RECEIPTS[0],'rust_receipt':f'{TWELVE_RUST}/result.json','rust_receipt_sha256':TWELVE_RECEIPTS[1],'stream_sha256':TWELVE_STREAM_SHA,'joined_cases_sha256':TWELVE_CASE_SHA}
    return ns,nc,sources,provenance

def attach_twelve_uvs(artifact,data,cases,sources,provenance):
    section=artifact.get('conditional_model_only_uv_groups')
    if not isinstance(section,dict) or len(data)!=704 or len(cases)!=2816 or len(section.get('uv_words_hex',[]))!=85 or len(section.get('uv_metadata',[]))!=85 or len(section.get('corner_uv_indices',[]))!=252:raise ValueError('invalid twelve UV/group shape')
    if tuple(len(sources.get(k,b'')) for k in ('atlas','sort','pair','offset','rows','sqrt','shelf','offset_inputs','atlas_inputs','shelf_inputs'))!=(288,192,2340,288,1192,1040,1544,384,336,400):raise ValueError('invalid twelve join inputs')
    bounds=list(struct.iter_unpack('<6I',sources['atlas']));order=struct.unpack('<48I',sources['sort'])[:12];pw=struct.unpack('<585I',sources['pair'])[:195];offsets=list(struct.iter_unpack('<6I',sources['offset']));inputs=list(struct.iter_unpack('<8I',sources['offset_inputs']));rows=struct.unpack_from('<42I',sources['rows']);initial=list(struct.iter_unpack('<7I',sources['atlas_inputs']));shelf_cases=struct.unpack_from('<50I',sources['shelf_inputs']);shelves=struct.unpack_from('<386I',sources['shelf'])
    if sorted(order)!=list(range(12)) or pw[2]!=0x3f2c2996 or struct.unpack_from('<I',sources['sqrt'])[0]!=0x3f990ab4 or shelves[0]!=3 or shelf_cases[:2]!=(12,0x3f990ab4):raise ValueError('invalid twelve order, shelf or scale')
    for i in range(4):
        row=rows[i*7:(i+1)*7]
        if row[0]!=i or row[1]!=0 or row[6]!=0 or initial[i][:4]!=row[2:6]:raise ValueError('row-derived initial group mismatch')
    for i,member in enumerate(order):
        if shelf_cases[2+4*i:6+4*i]!=bounds[member][:4]:raise ValueError('shelf sorted bounds mismatch')
    by_group={};k=0
    for row in range(3):
        rec=pw[3+16*row:3+16*(row+1)];count,_,height,shift=rec[:4]
        if not 1<=count<=12:raise ValueError('invalid paired row count')
        for j,member in enumerate(rec[4:4+count]):
            if member>=12 or k>=12:raise ValueError('invalid paired member')
            gid=order[member]
            if gid in by_group or inputs[k]!=(*bounds[gid][:4],height,shift,row,int(j+1==count)):raise ValueError('paired group or offset mismatch')
            by_group[gid]=offsets[k][2:4];k+=1
    if k!=12 or len(by_group)!=12:raise ValueError('incomplete paired groups')
    values=list(struct.iter_unpack('<2I',data));uvcases=list(struct.iter_unpack('<8I',cases))
    if any(not math.isfinite(struct.unpack('<f',struct.pack('<I',x))[0]) for pair in values for x in pair):raise ValueError('nonfinite twelve UV')
    for i in range(85):
        meta=section['uv_metadata'][i]
        if not isinstance(meta,dict):raise ValueError('invalid UV metadata')
        gid=meta.get('model_group_id')
        if type(gid) is not int or not 4<=gid<12:raise ValueError('invalid generated group id')
        try:original=tuple(int(x,16) for x in section['uv_words_hex'][i])
        except (TypeError,ValueError):raise ValueError('invalid original UV words')
        row=uvcases[i];g=bounds[gid]
        if len(original)!=2 or row[:2]!=original or row[2:5]!=(g[0],g[1],g[4]) or row[5:7]!=by_group[gid] or row[7]!=pw[2]:raise ValueError('twelve joined UV mismatch')
    corners=section['corner_uv_indices']
    if any(type(x) is not int or not 0<=x<85 for x in corners):raise ValueError('invalid twelve UV corner indices')
    artifact['conditional_twelve_group_transformed_uvs']={'limits':TWELVE_LIMITS,'provenance':provenance,'conditional_source_scale_f32_bits_hex':f'{pw[2]:08x}',
      'uv':[[struct.unpack('<f',struct.pack('<I',x))[0] for x in pair] for pair in values[:85]],'uv_words_hex':[[f'{x:08x}' for x in pair] for pair in values[:85]],'corner_uv_indices':list(corners)}
    return artifact

TANGENT_PREFIX='creature-skin-uv-tangent-prefix-12group-0473-prep-20261109'
TANGENT_NATIVE='creature-skin-uv-tangent-final-12group-0473-prep-20261109'
TANGENT_RUST='creature-skin-uv-tangent-final-12group-rust-verify-20261109'
TANGENT_PINS={
 TANGENT_PREFIX:{'result.json':'8d18aaf662c4f632a26243ce6d84d8b6c82b705f7ff488a44fcaea9dfe475ea3','host-stdout.bin':'747edd288c43f31ca32d432b8552d1e15a3e17b9b875b3ac4d94bad180ad1a39','uvs.bin':'59e140907199e78c753ad36f20aaadf84ce324c995ab0dee7cd55ee98fe7b2b6','corners.bin':'687ade6614a2e231c066a87d7769b532eb4c31ddd1e997a2fc768ec0a6b5990e','indices.bin':'1985ed53e669dba5fab92c7fa4edb54948c22a95da44c081f1afdcbd870e1b2e','position-indices.bin':'01111d5dd13ff186458dcff418e4186c1d59d8d163d5ca50af7bba7db05b9028'},
 TANGENT_NATIVE:{'result.json':'a601e8d8781acf0e8f8508fb35aa41cd1c42db1286e6cb0a218541236809a9a2','host-stdout.bin':'79b43aea2a95ba576cb526eaf0e0d0c49d4fdcffde065c267d7ac1825ee122e7','tangents.bin':'747edd288c43f31ca32d432b8552d1e15a3e17b9b875b3ac4d94bad180ad1a39','normals.bin':'9e98416ac15a9e52e3bdb1c611b4c3d6e074e94bdfe43e37207496bd0c6e8651','indices.bin':'1985ed53e669dba5fab92c7fa4edb54948c22a95da44c081f1afdcbd870e1b2e','normal-indices.bin':'01111d5dd13ff186458dcff418e4186c1d59d8d163d5ca50af7bba7db05b9028'},
 TANGENT_RUST:{'result.json':'6f20e861a1e872a515233a8bcae20bddc7cd72121b2067aa1abd6bef6802f7bd','rust-stdout.bin':'fa914af114c3c8ef192676cdd87f0eaddc2e935b92aea69b18af9a00c48218e2'},
}
TANGENT_LIMITS=('Conditional frozen-row UV, position and normal fixture; same-object +6C/+58 index identity, successful lifecycle, preallocated native storage and CRT-compatible square-root spills are admitted. Native and Rust match only these 85 pairs. Actual texture/material binding, live creature identity and unconditional finalized mesh remain unverified.')

def read_tangent_pairs(target):
    data={};provenance={}
    for folder,files in TANGENT_PINS.items():
        for name,sha in files.items():
            raw=(target/folder/name).read_bytes()
            if digest(raw)!=sha:raise ValueError(f'{folder}/{name}: hash mismatch')
            data[folder,name]=raw
        provenance[folder]={'receipt':f'{folder}/result.json','receipt_sha256':files['result.json']}
    for folder in (TANGENT_PREFIX,TANGENT_NATIVE):
        q=json.loads(data[folder,'result.json'])
        if q.get('status')!='matched' or q.get('host_exit')!=0 or q.get('host_timed_out') is not False:raise ValueError('tangent native comparison invalid')
    q=json.loads(data[TANGENT_RUST,'result.json']);native=data[TANGENT_NATIVE,'host-stdout.bin'];rust=data[TANGENT_RUST,'rust-stdout.bin']
    if (q.get('status')!='matched' or q.get('rust_exit')!=0 or q.get('cases')!=85 or q.get('native_receipt_sha256')!=TANGENT_PINS[TANGENT_NATIVE]['result.json'] or q.get('output_sha256')!=TANGENT_PINS[TANGENT_RUST]['rust-stdout.bin'] or len(native)!=3064 or len(rust)!=2040 or native[:2040]!=rust or data[TANGENT_NATIVE,'tangents.bin']!=data[TANGENT_PREFIX,'host-stdout.bin'] or data[TANGENT_NATIVE,'indices.bin']!=data[TANGENT_PREFIX,'indices.bin'] or data[TANGENT_NATIVE,'normal-indices.bin']!=data[TANGENT_PREFIX,'position-indices.bin']):raise ValueError('tangent stream comparison invalid')
    provenance['stream_sha256']=TANGENT_PINS[TANGENT_RUST]['rust-stdout.bin']
    return rust,data,provenance

def attach_tangent_pairs(artifact,stream,data,provenance):
    uv=artifact.get('conditional_twelve_group_transformed_uvs');groups=artifact.get('conditional_model_only_uv_groups')
    if not isinstance(uv,dict) or not isinstance(groups,dict) or len(stream)!=2040:raise ValueError('missing conditional twelve-group UVs or tangent stream')
    try:
        rawuv=data[TANGENT_PREFIX,'uvs.bin'];rawcorners=data[TANGENT_PREFIX,'corners.bin'];rawindices=data[TANGENT_PREFIX,'indices.bin'];rawnormals=data[TANGENT_NATIVE,'normals.bin'];rawpos=data[TANGENT_NATIVE,'normal-indices.bin']
    except KeyError:raise ValueError('missing tangent input')
    if (len(rawuv),len(rawcorners),len(rawindices),len(rawnormals),len(rawpos))!=(680,1008,340,528,1008):raise ValueError('invalid tangent input lengths')
    uvwords=list(struct.iter_unpack('<2I',rawuv));corners=struct.unpack('<252I',rawcorners);indices=struct.unpack('<85I',rawindices);normals=list(struct.iter_unpack('<3I',rawnormals));pos=struct.unpack('<252I',rawpos);pairs=list(struct.iter_unpack('<6I',stream))
    if len(uv.get('uv_words_hex',[]))!=85 or len(groups.get('uv_metadata',[]))!=85 or tuple(uv.get('corner_uv_indices',[]))!=corners or len(artifact.get('normals',[]))!=44:raise ValueError('tangent UV or normal shape mismatch')
    try:triangles=tuple(x for tri in artifact['triangles'] for x in tri)
    except (KeyError,TypeError):raise ValueError('invalid position indices')
    if triangles!=pos:raise ValueError('tangent normal-index relation mismatch')
    for i,v in enumerate(artifact['normals']):
        try:bits=tuple(struct.unpack('<I',struct.pack('<f',x))[0] for x in v)
        except (TypeError,ValueError,OverflowError,struct.error):raise ValueError('invalid normal words')
        if bits!=normals[i]:raise ValueError('tangent normal mismatch')
    for i,(words,k) in enumerate(zip(uvwords,indices)):
        try:actual=tuple(int(x,16) for x in uv['uv_words_hex'][i]);meta=groups['uv_metadata'][i]
        except (TypeError,ValueError):raise ValueError('invalid UV words')
        if not isinstance(meta,dict) or actual!=words or meta.get('conditional_corner_ordinal')!=k or k>=252 or corners[k]!=i or pos[k]>=44:raise ValueError('tangent UV mapping mismatch')
    if any(not math.isfinite(struct.unpack('<f',struct.pack('<I',x))[0]) for v in pairs for x in v):raise ValueError('nonfinite tangent pair')
    artifact['conditional_tangent_pairs']={'limits':TANGENT_LIMITS,'provenance':provenance,
      'tangents':[[struct.unpack('<f',struct.pack('<I',x))[0] for x in row[:3]] for row in pairs],
      'bitangents':[[struct.unpack('<f',struct.pack('<I',x))[0] for x in row[3:]] for row in pairs],
      'tangent_words_hex':[[f'{x:08x}' for x in row[:3]] for row in pairs],
      'bitangent_words_hex':[[f'{x:08x}' for x in row[3:]] for row in pairs],
      'corner_uv_indices':list(corners)}
    return artifact

EIGHT_NATIVE='creature-skin-export-attributes-8group-0473-prep-20261109'
EIGHT_RUST='creature-skin-export-attributes-8group-rust-verify-20261109'
EIGHT_TRIPLES='creature-skin-triples-8group-0473-prep-v2-20261109'
EIGHT_PINS={EIGHT_NATIVE:{'result.json':'04bdb04705a1c5e8c448e3bb0ed63b8688ba858b1d6e5c20db0af709478fe79e','host-stdout.bin':'a60cf07dd312716d7c5380767762b6002c6a5ade63350cca34a1a778cad2a0a0','triples.bin':'aecf4feb8f5fbdaa1fcd5a1c638246e83dcd1ed23e21dd587c2cd248b6259008','positions.bin':'6240f2d95681ddd6cccb17679b12cf528c15710c6cdb351160aeb56e7656ee4f','normals.bin':'9e98416ac15a9e52e3bdb1c611b4c3d6e074e94bdfe43e37207496bd0c6e8651','uv.bin':'59e140907199e78c753ad36f20aaadf84ce324c995ab0dee7cd55ee98fe7b2b6','tangents.bin':'fa914af114c3c8ef192676cdd87f0eaddc2e935b92aea69b18af9a00c48218e2'},EIGHT_RUST:{'result.json':'bd36f8a1e903e6b298e81432cf50e36125b39de9c93558a4c1649a0638ed2978','rust-stdout.bin':'a60cf07dd312716d7c5380767762b6002c6a5ade63350cca34a1a778cad2a0a0'},EIGHT_TRIPLES:{'result.json':'5bd2dbece6a3682c7be9b376b08a82f4d2b602962838fadb85198ccc88fb0885','host-stdout.bin':'18f673e883de0334cab3499e2a65d0bc10322b90c0d659aed7d451a4968f0182'}}
EIGHT_LIMITS=('Conditional eight generated-group inspection geometry; four earlier row-derived groups are excluded. Frozen source rows, same-object and copied +6C normal-index relation, synthetic lifecycle and preallocation are admissions. UV placement and source scale are conditional; native allocation, live creature identity and material or texture binding are unverified. Tangents contain only the source-exported 12-byte half of each tangent/bitangent pair.')

def read_eight_export(target):
    data={(folder,name):(target/folder/name).read_bytes() for folder,files in EIGHT_PINS.items() for name in files}
    if any(digest(data[folder,name])!=want for folder,files in EIGHT_PINS.items() for name,want in files.items()):raise ValueError('eight-group export hash mismatch')
    n=json.loads(data[EIGHT_NATIVE,'result.json']);r=json.loads(data[EIGHT_RUST,'result.json']);t=json.loads(data[EIGHT_TRIPLES,'result.json'])
    out=data[EIGHT_NATIVE,'host-stdout.bin']
    if n.get('status')!='matched' or n.get('host_exit')!=0 or n.get('host_timed_out') is not False or t.get('status')!='matched' or t.get('host_exit')!=0 or r.get('status')!='matched' or r.get('native_receipt_sha256')!=EIGHT_PINS[EIGHT_NATIVE]['result.json'] or r.get('output_sha256')!=digest(out) or out!=data[EIGHT_RUST,'rust-stdout.bin'] or len(out)!=3740:raise ValueError('eight-group export comparison invalid')
    return out,data,{'files_sha256':EIGHT_PINS}

def attach_eight_export(artifact,out,data,provenance):
    groups=artifact.get('conditional_model_only_uv_groups',{}).get('groups',[]);uv=artifact.get('conditional_twelve_group_transformed_uvs',{}).get('uv_words_hex',[]);tan=artifact.get('conditional_tangent_pairs',{}).get('tangent_words_hex',[])
    if len(groups)!=8 or len(uv)!=85 or len(tan)!=85 or len(out)!=3740:raise ValueError('missing conditional eight-group inputs')
    raw=data[EIGHT_TRIPLES,'host-stdout.bin'];triples=data[EIGHT_NATIVE,'triples.bin']
    if len(raw)!=12384 or len(triples)!=1020:raise ValueError('invalid triple stream size')
    words=struct.unpack('<3096I',raw);flat=struct.unpack('<255I',triples);parts=(out[:1020],out[1020:2040],out[2040:2720],out[2720:]);rows=[list(struct.iter_unpack('<3I',parts[0])),list(struct.iter_unpack('<3I',parts[1])),list(struct.iter_unpack('<2I',parts[2])),list(struct.iter_unpack('<3I',parts[3]))]
    positions=[tuple(struct.unpack('<3I',struct.pack('<3f',*p))) for p in artifact['positions']];normals=[tuple(struct.unpack('<3I',struct.pack('<3f',*p))) for p in artifact['normals']];corners=artifact['conditional_model_only_uv_groups']['corner_uv_indices'];triangles=artifact['triangles']
    result=[];at=0
    for gi,group in enumerate(groups):
        row=words[gi*387:(gi+1)*387];gid,nfaces,count=row[:3];faces=group['faces']
        if gid!=gi+4 or group['model_group_id']!=gid or nfaces!=len(faces) or not 1<=count<=44:raise ValueError('invalid group framing')
        ts=[row[3+3*j:6+3*j] for j in range(count)];local=row[135:135+3*nfaces]
        if tuple(x for t in ts for x in t)!=flat[3*at:3*(at+count)] or any(x>=count for x in local):raise ValueError('invalid local triple mapping')
        for k,face in enumerate(faces):
            if not 0<=face<84:raise ValueError('invalid face')
            for lane in range(3):
                pi,ni,ui=ts[local[3*k+lane]]
                if pi!=triangles[face][lane] or ni!=pi or ui!=corners[3*face+lane]:raise ValueError('corner mapping mismatch')
        for j,(pi,ni,ui) in enumerate(ts):
            if pi>=44 or ni>=44 or ui>=85 or rows[0][at+j]!=positions[pi] or rows[1][at+j]!=normals[ni] or rows[2][at+j]!=tuple(int(x,16) for x in uv[ui]) or rows[3][at+j]!=tuple(int(x,16) for x in tan[ui]):raise ValueError('attribute mapping mismatch')
        result.append({'conditional_group_id':gid,'face_ids':faces,'local_triangles':[list(local[i:i+3]) for i in range(0,len(local),3)],'vertex_offset':at,'vertex_count':count});at+=count
    if at!=85:raise ValueError('invalid vertex count')
    def floats(xs):return [[struct.unpack('<f',struct.pack('<I',x))[0] for x in row] for row in xs]
    if any(not math.isfinite(v) for rows_ in rows for row in floats(rows_) for v in row):raise ValueError('nonfinite attributes')
    artifact['conditional_eight_group_export']={'limits':EIGHT_LIMITS,'provenance':provenance,'groups':result,'positions':floats(rows[0]),'normals':floats(rows[1]),'uvs':floats(rows[2]),'tangents':floats(rows[3]),'attribute_words_hex':[[[f'{x:08x}' for x in row] for row in rows_] for rows_ in rows]}
    return artifact

SAME_OBJECT_FOLDER = 'creature-skin-export-loop-same-object-prep-v1-20261109'
SAME_OBJECT_ENTRY_FOLDER = 'creature-skin-export-entry-same-object-prep-v1-20261109'
SAME_OBJECT_PINS = {
    SAME_OBJECT_FOLDER: {'result.json': 'bc4cb5abd10379fc9aa48bd3e0aae96b3e3bf7e592a6c2ba4fc3e61f083ef98c', 'host-stdout.bin': '1816afa9346922d3fa65c37523ef4bf8b37188e3ebf450c80644bedfbd3da772'},
    SAME_OBJECT_ENTRY_FOLDER: {'result.json': '3b11010aa9d8de37c0b200e8b0ccd7e04bc5b8ac919e5de6dcef8c8b3943b8ff', 'host-stdout.bin': '1816afa9346922d3fa65c37523ef4bf8b37188e3ebf450c80644bedfbd3da772'},
    CALLER_SMOOTH_FOLDER: {'result.json': CALLER_SMOOTH_RECEIPT_SHA, 'host-stdout.bin': CALLER_SMOOTH_STREAM_SHA},
}
SAME_OBJECT_LIMITS = ('Conditional same-object inspection with supplied frozen inputs, synthetic owner, preallocated buffers and caller-alpha 0.7 pass-2 records. The original entry checked the supplied owner and 44-record count, and the export loop computed bases after four conditionally empty groups; seven synthetic no-growth reserve stubs were used. Real owner identity, atlas cleanup, allocation, lifecycle, live settings, palette identities and texture binding are unverified.')

def read_same_object_export(target):
    data={(folder,name):(target/folder/name).read_bytes() for folder,files in SAME_OBJECT_PINS.items() for name in files}
    if any(digest(data[folder,name])!=want for folder,files in SAME_OBJECT_PINS.items() for name,want in files.items()):raise ValueError('same-object export hash mismatch')
    receipt=json.loads(data[SAME_OBJECT_FOLDER,'result.json']);entry=json.loads(data[SAME_OBJECT_ENTRY_FOLDER,'result.json']);smooth=json.loads(data[CALLER_SMOOTH_FOLDER,'result.json']);out=data[SAME_OBJECT_FOLDER,'host-stdout.bin'];entry_out=data[SAME_OBJECT_ENTRY_FOLDER,'host-stdout.bin'];records=data[CALLER_SMOOTH_FOLDER,'host-stdout.bin']
    if receipt.get('status')!='matched' or receipt.get('host_exit')!=0 or receipt.get('host_timed_out') is not False or receipt.get('host_stdout_sha256')!=digest(out) or entry.get('status')!='matched' or entry.get('host_exit')!=0 or entry.get('host_timed_out') is not False or entry.get('host_stdout_sha256')!=digest(entry_out) or entry_out!=out or smooth.get('status')!='matched' or smooth.get('host_exit')!=0 or smooth.get('host_timed_out') is not False or smooth.get('host_stdout_sha256')!=digest(records) or len(out)!=28476 or len(records)!=2640:raise ValueError('same-object export receipt invalid')
    return out,records,{'files_sha256':SAME_OBJECT_PINS}

def attach_same_object_export(artifact,out,records,triples,provenance):
    section=artifact.get('conditional_eight_group_export',{});groups=section.get('groups',[]);words=section.get('attribute_words_hex',[])
    if len(out)!=28476 or len(records)!=2640 or len(triples)!=1020 or len(groups)!=8 or len(words)!=4:raise ValueError('missing same-object inputs')
    try: attribute=b''.join(struct.pack('<I',int(v,16)) for rows in words for row in rows for v in row)
    except (ValueError,TypeError,struct.error) as e:raise ValueError('invalid attribute words') from e
    if len(attribute)!=3740 or out[21688:25428]!=attribute:raise ValueError('same-object attribute mismatch')
    global_indices=[];expected_offset=0
    for group in groups:
        offset=group['vertex_offset'];count=group['vertex_count']
        if offset!=expected_offset or not 1<=count<=44:raise ValueError('invalid group offsets')
        for triangle in group['local_triangles']:
            if len(triangle)!=3 or any(not isinstance(x,int) or x<0 or x>=count for x in triangle):raise ValueError('invalid local indices')
            global_indices.extend(offset+x for x in triangle)
        expected_offset+=count
    if expected_offset!=85 or len(global_indices)!=252 or out[27468:]!=struct.pack('<252I',*global_indices):raise ValueError('same-object index mismatch')
    ts=list(struct.iter_unpack('<3I',triples));offsets=out[25428:26108];weights=out[26108:27468];raw_records=records[1760:2640]
    if len(ts)!=85:raise ValueError('invalid triple count')
    for i,(position,normal,uv) in enumerate(ts):
        if position>=44 or normal>=44 or uv>=85:raise ValueError('invalid triple index')
        rec=raw_records[position*20:(position+1)*20]
        if offsets[i*8:(i+1)*8]!=b''.join((v*3).to_bytes(2,'little') for v in rec[:4]) or weights[i*16:(i+1)*16]!=rec[4:]:raise ValueError('same-object record mapping mismatch')
    if any(not math.isfinite(v[0]) for v in struct.iter_unpack('<f',weights)):raise ValueError('nonfinite weights')
    artifact['conditional_same_object_export']={'limits':SAME_OBJECT_LIMITS,'provenance':provenance,'explicit_alpha_bits_hex':'3f333333','explicit_passes':2,'global_indices':global_indices,'index_offset_words_hex':[[f'{v:04x}' for v in row] for row in struct.iter_unpack('<4H',offsets)],'weight_words_hex':[[f'{v:08x}' for v in row] for row in struct.iter_unpack('<4I',weights)]}
    return artifact

ROW_CATEGORIES_FOLDER = 'creature-skin-export-category-same-object-prep-v1-20261109'
ROW_PREFIX_FOLDER = 'creature-skin-category-prefix-same-object-prep-v1-20261109'
ROW_CATEGORIES_PINS = {ROW_CATEGORIES_FOLDER: {'result.json':'241789b27edf3fdec72f7cff7bccc6d05928b7502f09b2dabed7847d7d513a5e','host-stdout.bin':'3a117ba39ddf15ed58bf2daf4360e013a1738ae4183ee25f397e02a55ae7f41e','rows.bin':'24a7822442c1caa64c0f386e5bd645b172af51fb187ad9fc28c6cecfebe10a5c'},ROW_PREFIX_FOLDER:{'result.json':'81452cd40ac77d6890bb9583e167643a5259b49635513b6f1322d6f254706477','host-stdout.bin':'0813ca304ebe9ae5fe8a70174870808b19bfc3c5525cb2bcbf03e83d400a12b3'}}
ROW_CATEGORIES_LIMITS = ('Conditional classifications and trailing-class-1 prefix of ten frozen rows in source order, under a synthetic same-owner association, external-count bypass and synthetic logical resize. These values are row categories, not palette IDs. Live owner, allocation, destructor, lifecycle, smoothing state and texture/material binding are unverified.')

def read_row_categories(target):
    data={(folder,name):(target/folder/name).read_bytes() for folder,files in ROW_CATEGORIES_PINS.items() for name in files}
    if any(digest(data[folder,name])!=want for folder,files in ROW_CATEGORIES_PINS.items() for name,want in files.items()):raise ValueError('row category hash mismatch')
    receipt=json.loads(data[ROW_CATEGORIES_FOLDER,'result.json']);prefix_receipt=json.loads(data[ROW_PREFIX_FOLDER,'result.json']);out=data[ROW_CATEGORIES_FOLDER,'host-stdout.bin'];prefix_out=data[ROW_PREFIX_FOLDER,'host-stdout.bin'];rows=data[ROW_CATEGORIES_FOLDER,'rows.bin']
    if receipt.get('status')!='matched' or receipt.get('host_exit')!=0 or receipt.get('host_timed_out') is not False or receipt.get('host_stdout_sha256')!=digest(out) or prefix_receipt.get('status')!='matched' or prefix_receipt.get('host_exit')!=0 or prefix_receipt.get('host_timed_out') is not False or prefix_receipt.get('host_stdout_sha256')!=digest(prefix_out) or len(out)!=28516 or len(prefix_out)!=28740 or len(rows)!=1400:raise ValueError('row category receipt invalid')
    return out,rows,prefix_out,{'files_sha256':ROW_CATEGORIES_PINS}

def attach_row_categories(artifact,out,rows,prefix_out,same_out,provenance):
    if 'conditional_same_object_export' not in artifact or len(out)!=28516 or len(prefix_out)!=28740 or len(same_out)!=28476 or len(rows)!=1400:raise ValueError('missing row category inputs')
    if out[:28476]!=same_out:raise ValueError('row category prefix mismatch')
    if prefix_out[:28516]!=out:raise ValueError('row prefix stream mismatch')
    values=[]
    for i in range(10):
        row=rows[i*140:(i+1)*140];flags=struct.unpack_from('<H',row,8)[0];kind=row[11]
        values.append(1 if kind==5 else 2 if kind==3 and flags&1 else 3 if kind in (3,4) else 4 if not(flags&8) else 5)
    if out[28476:]!=struct.pack('<10I',*values):raise ValueError('row category mapping mismatch')
    snapshots=struct.unpack('<56I',prefix_out[28516:]);rowsets=[rows]
    for mode in range(1,4):
        changed=bytearray(rows)
        for i in (range(8,10) if mode==1 else range(10) if mode==2 else [1]):changed[i*140+11]=1
        rowsets.append(changed)
    for i,rowset in enumerate(rowsets):
        n=10
        while n and rowset[(n-1)*140+11]==1:n-=1
        if snapshots[14*i:14*(i+1)]!=(10,n,n,n,*values):raise ValueError('row prefix mapping mismatch')
    artifact['conditional_row_categories']={'limits':ROW_CATEGORIES_LIMITS,'provenance':provenance,'categories_in_frozen_row_order':values,'conditional_prefix_length':snapshots[1]}
    return artifact

TRIANGLE_BOUNDS_FOLDER = 'creature-skin-triangle-bounds-prep-v1-20261109'
POSITION_SELECTOR_FOLDER = 'creature-skin-position-selector-prep-v5-20261109'
TRIANGLE_BOUNDS_PINS = {
    TRIANGLE_BOUNDS_FOLDER: {'result.json':'7e9c7562029e325fe36a76f4bf819133792dd83b07f1d5793822c4576587cfe0','host-stdout.bin':'38b0f3fc4a24d8948c47935d30544bf279495421cd762d186e91f3b0f0cb75c9'},
    POSITION_SELECTOR_FOLDER: {'result.json':'9b41013c9c61bd2bda4c00c2a0da60753c93b740ef9c7a4f0b6f06d93958c62e','host-stdout.bin':'106899d6d5c6770eac03961aa17aa035ae8b5e7b37473bbac64aad78ca88ad7b'},
}
TRIANGLE_BOUNDS_LIMITS = ('Conditional source-loop triangle bounds using supplied index mapping, synthetic group and seeded non-remap branch. Actual group and primitive selection, remapping, allocation, lifecycle, renderer, shader, palette and live creature identity remain unverified.')

def read_triangle_bounds(target):
    data={(folder,name):(target/folder/name).read_bytes() for folder,files in TRIANGLE_BOUNDS_PINS.items() for name in files}
    if any(digest(data[folder,name])!=want for folder,files in TRIANGLE_BOUNDS_PINS.items() for name,want in files.items()):raise ValueError('triangle bounds hash mismatch')
    for folder in TRIANGLE_BOUNDS_PINS:
        receipt=json.loads(data[folder,'result.json']);stream=data[folder,'host-stdout.bin']
        if receipt.get('status')!='matched' or receipt.get('host_exit')!=0 or receipt.get('host_timed_out') is not False or receipt.get('host_stdout_sha256')!=digest(stream):raise ValueError('triangle bounds receipt invalid')
    out=data[TRIANGLE_BOUNDS_FOLDER,'host-stdout.bin'];selector=data[POSITION_SELECTOR_FOLDER,'host-stdout.bin']
    if len(out)!=30980 or len(selector)!=28964 or out[:28964]!=selector:raise ValueError('triangle bounds prefix mismatch')
    return out,selector,{'files_sha256':TRIANGLE_BOUNDS_PINS}

def triangle_extrema_words(positions,indices):
    if len(positions)!=85 or len(indices)!=252 or any(len(row)!=3 for row in positions):raise ValueError('invalid triangle inputs')
    try:
        words=[[int(v,16) for v in row] for row in positions]
        vals=[[struct.unpack('<f',struct.pack('<I',v))[0] for v in row] for row in words]
    except (ValueError,TypeError,struct.error,OverflowError) as e:raise ValueError('invalid position words') from e
    if any(not math.isfinite(v) for row in vals for v in row):raise ValueError('nonfinite positions')
    if any(type(i) is not int or i<0 or i>=85 for i in indices):raise ValueError('invalid triangle indices')
    out=[]
    for t in range(84):
        x,y,z=indices[3*t:3*t+3];row=[]
        for maximum in (False,True):
            for axis in range(3):
                second=y if ((vals[y][axis]>=vals[z][axis]) if maximum else (vals[y][axis]<=vals[z][axis])) else z
                selected=x if ((vals[x][axis]>=vals[second][axis]) if maximum else (vals[x][axis]<=vals[second][axis])) else second
                row.append(words[selected][axis])
        out.append(row)
    return out

def attach_triangle_bounds(artifact,out,selector,same_out,provenance):
    position=artifact.get('conditional_eight_group_export',{}).get('attribute_words_hex',[])
    indices=artifact.get('conditional_same_object_export',{}).get('global_indices',[])
    if len(out)!=30980 or len(selector)!=28964 or len(same_out)!=28476 or len(position)!=4:raise ValueError('missing triangle inputs')
    if out[:28964]!=selector or selector[:28476]!=same_out:raise ValueError('triangle bounds prefix mismatch')
    try:position_bytes=b''.join(struct.pack('<I',int(v,16)) for row in position[0] for v in row)
    except (ValueError,TypeError,struct.error,OverflowError) as e:raise ValueError('invalid position words') from e
    rows=triangle_extrema_words(position[0],indices)
    if same_out[21688:22708]!=position_bytes or same_out[27468:28476]!=struct.pack('<252I',*indices):raise ValueError('triangle input mapping mismatch')
    if out[28964:]!=b''.join(struct.pack('<6I',*row) for row in rows):raise ValueError('triangle bounds mapping mismatch')
    artifact['conditional_triangle_bounds']={'limits':TRIANGLE_BOUNDS_LIMITS,'provenance':provenance,'extrema_words_hex':[[f'{v:08x}' for v in row] for row in rows]}
    return artifact

AGGREGATE_BOUNDS_FOLDER = 'creature-skin-bounds-reduction-prep-v3-20261109'
AGGREGATE_BOUNDS_PINS = {AGGREGATE_BOUNDS_FOLDER: {'result.json':'f3fbdb1bb91539d8dabcb70e8bd2ba65cdac644441a62fbc55a6559547ec7965','host-stdout.bin':'944ef8b538409bbd80f9ac0ee5ddb1d3f49f5ea3ac3e599f78037ac6d36f597f','controls-reduce.bin':'286b893a2f39c1c08be76157eff82314c16e7704c8ae2122dea627162a1cd97c'}}
AGGREGATE_BOUNDS_LIMITS = ('Conditional original-loop aggregate over supplied non-remap triangle bounds and a synthetic group. Spatial tree creation, group selection, allocation, lifecycle, renderer and live creature identity remain unverified.')

def read_aggregate_bounds(target):
    data={(folder,name):(target/folder/name).read_bytes() for folder,files in AGGREGATE_BOUNDS_PINS.items() for name in files}
    if any(digest(data[folder,name])!=want for folder,files in AGGREGATE_BOUNDS_PINS.items() for name,want in files.items()):raise ValueError('aggregate bounds hash mismatch')
    receipt=json.loads(data[AGGREGATE_BOUNDS_FOLDER,'result.json']);out=data[AGGREGATE_BOUNDS_FOLDER,'host-stdout.bin'];controls=data[AGGREGATE_BOUNDS_FOLDER,'controls-reduce.bin']
    if receipt.get('status')!='matched' or receipt.get('host_exit')!=0 or receipt.get('host_timed_out') is not False or receipt.get('host_stdout_sha256')!=digest(out) or len(out)!=31052 or len(controls)!=96:raise ValueError('aggregate bounds receipt invalid')
    return out,controls,{'files_sha256':AGGREGATE_BOUNDS_PINS}

def reduce_bounds_words(rows):
    if not rows or any(len(row)!=6 for row in rows):raise ValueError('invalid aggregate inputs')
    try:
        words=[[int(v,16) if isinstance(v,str) else v for v in row] for row in rows]
        vals=[[struct.unpack('<f',struct.pack('<I',v))[0] for v in row] for row in words]
    except (ValueError,TypeError,struct.error,OverflowError) as e:raise ValueError('invalid aggregate words') from e
    if any(not math.isfinite(v) for row in vals for v in row):raise ValueError('nonfinite aggregate inputs')
    acc=words[0][:];values=vals[0][:]
    for nxt,nvalues in zip(words[1:],vals[1:]):
        if values[0]>values[3]:acc=nxt[:];values=nvalues[:];continue
        for axis in range(3):
            if values[axis]>nvalues[axis]:acc[axis]=nxt[axis];values[axis]=nvalues[axis]
            if nvalues[axis+3]>values[axis+3]:acc[axis+3]=nxt[axis+3];values[axis+3]=nvalues[axis+3]
    return acc

def attach_aggregate_bounds(artifact,out,controls,triangle_out,provenance):
    rows=artifact.get('conditional_triangle_bounds',{}).get('extrema_words_hex',[])
    if len(rows)!=84 or len(out)!=31052 or len(controls)!=96 or len(triangle_out)!=30980:raise ValueError('missing aggregate inputs')
    if out[:30980]!=triangle_out:raise ValueError('aggregate bounds prefix mismatch')
    try: triangle=b''.join(struct.pack('<6I',*(int(v,16) for v in row)) for row in rows)
    except (ValueError,TypeError,struct.error,OverflowError) as e:raise ValueError('invalid aggregate words') from e
    if triangle_out[28964:]!=triangle:raise ValueError('aggregate triangle mapping mismatch')
    controls_rows=list(struct.iter_unpack('<6I',controls))
    computed=reduce_bounds_words(rows)+reduce_bounds_words(controls_rows[:2])+reduce_bounds_words(controls_rows[2:])
    if out[30980:]!=struct.pack('<18I',*computed):raise ValueError('aggregate bounds mapping mismatch')
    artifact['conditional_aggregate_bounds']={'limits':AGGREGATE_BOUNDS_LIMITS,'provenance':provenance,'extrema_words_hex':[f'{v:08x}' for v in computed[:6]]}
    return artifact

DEFORM_FOLDER = 'creature-skin-deform-position-prep-v1-20261109'
DEFORM_PINS = {
    DEFORM_FOLDER: {'result.json':'321e8f0083b3790ce0c075c10b60cd5d9883d4202b88ee9a5f4f88a133f138c3','host-stdout.bin':'38679f80f7ccbd81a2260c2ec20a2803317edff4b29cb72c3be8e307eeb9e0ba','synthetic-palette.bin':'5a946c16d7ab0dad7bf9cae07d79b4722d43da04b56e213f6b8bc0764b522f64'},
    POSITION_SELECTOR_FOLDER: TRIANGLE_BOUNDS_PINS[POSITION_SELECTOR_FOLDER],
}
DEFORM_LIMITS = ('Conditional original CPU position deformation with ten explicit synthetic translation matrices and supplied group/weight association. Normals and tangents are not deformed. Actual palette, live pose, world, rendering, GPU binding, lifecycle and creature identity remain unverified.')

def read_synthetic_deformation(target):
    data={(folder,name):(target/folder/name).read_bytes() for folder,files in DEFORM_PINS.items() for name in files}
    if any(digest(data[folder,name])!=want for folder,files in DEFORM_PINS.items() for name,want in files.items()):raise ValueError('deformation hash mismatch')
    for folder in DEFORM_PINS:
        receipt=json.loads(data[folder,'result.json']);stream=data[folder,'host-stdout.bin']
        if receipt.get('status')!='matched' or receipt.get('host_exit')!=0 or receipt.get('host_timed_out') is not False or receipt.get('host_stdout_sha256')!=digest(stream):raise ValueError('deformation receipt invalid')
    out=data[DEFORM_FOLDER,'host-stdout.bin'];selector=data[POSITION_SELECTOR_FOLDER,'host-stdout.bin'];palette=data[DEFORM_FOLDER,'synthetic-palette.bin']
    if len(out)!=29984 or len(selector)!=28964 or len(palette)!=480 or out[:28964]!=selector:raise ValueError('deformation prefix mismatch')
    return out,selector,palette,{'files_sha256':DEFORM_PINS}

def f32_round(value):
    try:result=struct.unpack('<f',struct.pack('<f',value))[0]
    except (OverflowError,struct.error) as e:raise ValueError('nonfinite deformation intermediate') from e
    if not math.isfinite(result):raise ValueError('nonfinite deformation intermediate')
    return result

def deform_position_words(positions,offsets,weights,palette):
    if len(positions)!=85 or len(offsets)!=85 or len(weights)!=85 or len(palette)!=480 or any(len(r)!=3 for r in positions) or any(len(r)!=4 for r in offsets) or any(len(r)!=4 for r in weights):raise ValueError('invalid deformation inputs')
    try:
        pos=[[struct.unpack('<f',struct.pack('<I',int(x,16)))[0] for x in row] for row in positions]
        offs=[[int(x,16) for x in row] for row in offsets]
        ws=[[struct.unpack('<f',struct.pack('<I',int(x,16)))[0] for x in row] for row in weights]
        mats=struct.unpack('<120f',palette)
    except (ValueError,TypeError,struct.error,OverflowError) as e:raise ValueError('invalid deformation words') from e
    if any(not math.isfinite(v) for row in pos+ws for v in row) or any(not math.isfinite(v) for v in mats):raise ValueError('nonfinite deformation input')
    if any(v<0 or v>27 or v%3 for row in offs for v in row):raise ValueError('invalid deformation offsets')
    mul=lambda a,b:f32_round(a*b)
    add=lambda a,b:f32_round(a+b)
    out=[]
    for p,indices,w in zip(pos,offs,ws):
        matrix=[]
        for k in range(12):
            value=mul(mats[(indices[0]//3)*12+k],w[0])
            for lane in range(1,4):value=add(value,mul(mats[(indices[lane]//3)*12+k],w[lane]))
            matrix.append(value)
        x,y,z=p
        xyz=[add(add(add(mul(matrix[2],z),mul(matrix[1],y)),mul(matrix[0],x)),matrix[3]),add(add(add(mul(matrix[4],x),mul(matrix[6],z)),mul(matrix[5],y)),matrix[7]),add(add(add(mul(matrix[10],z),mul(matrix[9],y)),mul(matrix[8],x)),matrix[11])]
        out.append([struct.unpack('<I',struct.pack('<f',v))[0] for v in xyz])
    return out

def attach_synthetic_deformation(artifact,out,selector,palette,same_out,provenance):
    attributes=artifact.get('conditional_eight_group_export',{}).get('attribute_words_hex',[]);skin=artifact.get('conditional_same_object_export',{})
    offsets=skin.get('index_offset_words_hex',[]);weights=skin.get('weight_words_hex',[])
    if len(attributes)!=4 or len(out)!=29984 or len(selector)!=28964 or len(same_out)!=28476:raise ValueError('missing deformation inputs')
    if out[:28964]!=selector or selector[:28476]!=same_out:raise ValueError('deformation prefix mismatch')
    rows=deform_position_words(attributes[0],offsets,weights,palette)
    try:
        positions=b''.join(struct.pack('<I',int(v,16)) for row in attributes[0] for v in row)
        offset_bytes=b''.join(struct.pack('<4H',*(int(v,16) for v in row)) for row in offsets)
        weight_bytes=b''.join(struct.pack('<4I',*(int(v,16) for v in row)) for row in weights)
    except (ValueError,TypeError,struct.error,OverflowError) as e:raise ValueError('invalid deformation words') from e
    if same_out[21688:22708]!=positions or same_out[25428:26108]!=offset_bytes or same_out[26108:27468]!=weight_bytes:raise ValueError('deformation input mapping mismatch')
    if out[28964:]!=b''.join(struct.pack('<3I',*row) for row in rows):raise ValueError('deformation output mismatch')
    artifact['conditional_synthetic_cpu_deformation']={'limits':DEFORM_LIMITS,'provenance':provenance,'synthetic_palette_words_hex':[f'{v:08x}' for (v,) in struct.iter_unpack('<I',palette)],'position_words_hex':[[f'{v:08x}' for v in row] for row in rows]}
    return artifact

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path, help='new JSON artifact; existing files are not overwritten')
    parser.add_argument('--include-synthetic-initial-records', action='store_true',
                        help='include conditional initial records using synthetic endpoint coordinates; not palette identities')
    parser.add_argument('--include-row-coordinate-initial-records', action='store_true',
                        help='include conditional frozen-row initial records; not palette identities')
    parser.add_argument('--include-half-alpha-smoothed-records', type=int, choices=(0, 1, 2),
                        help='include controlled native-matched records for the explicitly selected pass count')
    parser.add_argument('--include-caller-alpha-smoothed-records', type=int, choices=(0, 1, 2),
                        help='include caller-factor records for an explicitly selected pass count')
    parser.add_argument('--include-synthetic-corner-attributes', type=int, choices=(0, 1, 2),
                        help='include validated synthetic corner mapping with explicitly selected pass count')
    parser.add_argument('--include-model-only-uv-groups', action='store_true', help='include conditional model-only UV/group fixture; not actual group assignment')
    parser.add_argument('--include-model-only-transformed-uvs', action='store_true', help='include conditional transformed UV inspection data with synthetic placement and scale')
    parser.add_argument('--include-paired-transformed-uvs', action='store_true', help='include conditional native-matched paired UVs; supplied shelves and scheduling are synthetic')
    parser.add_argument('--include-twelve-group-transformed-uvs', action='store_true', help='include conditional twelve-group native-matched UV inspection data')
    parser.add_argument('--include-tangent-pairs', action='store_true', help='include conditional native-matched per-UV tangent pairs')
    parser.add_argument('--include-eight-group-export', action='store_true', help='include conditional eight-group indexed inspection geometry; excludes first four groups')
    parser.add_argument('--include-same-object-export', action='store_true', help='include conditional same-object indices and pass-2 record words')
    parser.add_argument('--include-row-categories', action='store_true', help='include conditional frozen-row categories; not palette IDs')
    parser.add_argument('--include-triangle-bounds', action='store_true', help='include conditional source-loop triangle bounds with supplied index mapping')
    parser.add_argument('--include-aggregate-bounds', action='store_true', help='include conditional aggregate bounds; implies triangle bounds')
    parser.add_argument('--include-synthetic-cpu-deformation', action='store_true', help='include conditional position deformation with explicit synthetic matrices')
    args = parser.parse_args()
    artifact = compose(*read_inputs(ROOT / 'target'))
    if args.include_synthetic_initial_records:
        artifact = attach_synthetic_records(artifact, *read_inputs(ROOT / 'target', RECORD_SPEC))
    if args.include_row_coordinate_initial_records:
        artifact = attach_row_records(artifact, *read_row_records(ROOT / 'target'))
    if args.include_half_alpha_smoothed_records is not None:
        artifact = attach_smoothed_records(artifact, *read_smoothed_records(ROOT / 'target'), args.include_half_alpha_smoothed_records)
    if args.include_caller_alpha_smoothed_records is not None:
        artifact = attach_caller_smoothed_records(artifact, *read_caller_smoothed_records(ROOT / 'target'), args.include_caller_alpha_smoothed_records)
    if args.include_synthetic_corner_attributes is not None:
        records, record_provenance = read_caller_smoothed_records(ROOT / 'target')
        corners, corner_provenance = read_synthetic_corner_attributes(ROOT / 'target')
        artifact = attach_synthetic_corner_attributes(artifact, corners, {'corner': corner_provenance, 'records': record_provenance}, records, args.include_synthetic_corner_attributes)
    if args.include_model_only_uv_groups or args.include_model_only_transformed_uvs or args.include_paired_transformed_uvs or args.include_twelve_group_transformed_uvs or args.include_tangent_pairs or args.include_eight_group_export or args.include_same_object_export or args.include_row_categories or args.include_triangle_bounds or args.include_aggregate_bounds or args.include_synthetic_cpu_deformation:
        artifact = attach_multigroup(artifact, *read_multigroup(ROOT / 'target'))
    if args.include_model_only_transformed_uvs:
        artifact = attach_transformed_uvs(artifact, *read_transformed_uvs(ROOT / 'target'))
    if args.include_paired_transformed_uvs:
        artifact = attach_paired_uvs(artifact, *read_paired_uvs(ROOT / 'target'))
    if args.include_twelve_group_transformed_uvs or args.include_tangent_pairs or args.include_eight_group_export or args.include_same_object_export or args.include_row_categories or args.include_triangle_bounds or args.include_aggregate_bounds or args.include_synthetic_cpu_deformation:
        artifact = attach_twelve_uvs(artifact, *read_twelve_uvs(ROOT / 'target'))
    if args.include_tangent_pairs or args.include_eight_group_export or args.include_same_object_export or args.include_row_categories or args.include_triangle_bounds or args.include_aggregate_bounds or args.include_synthetic_cpu_deformation:
        artifact = attach_tangent_pairs(artifact, *read_tangent_pairs(ROOT / 'target'))
    if args.include_eight_group_export or args.include_same_object_export or args.include_row_categories or args.include_triangle_bounds or args.include_aggregate_bounds or args.include_synthetic_cpu_deformation:
        eight_out,eight_data,eight_provenance = read_eight_export(ROOT / 'target')
        artifact = attach_eight_export(artifact, eight_out,eight_data,eight_provenance)
    if args.include_same_object_export or args.include_row_categories or args.include_triangle_bounds or args.include_aggregate_bounds or args.include_synthetic_cpu_deformation:
        same_out,same_records,same_provenance=read_same_object_export(ROOT / 'target')
        artifact = attach_same_object_export(artifact,same_out,same_records,eight_data[EIGHT_NATIVE,'triples.bin'],same_provenance)
    if args.include_row_categories:
        category_out,category_rows,prefix_out,category_provenance=read_row_categories(ROOT / 'target')
        artifact = attach_row_categories(artifact,category_out,category_rows,prefix_out,same_out,category_provenance)
    if args.include_triangle_bounds or args.include_aggregate_bounds:
        bounds_out,selector_out,bounds_provenance=read_triangle_bounds(ROOT / 'target')
        artifact=attach_triangle_bounds(artifact,bounds_out,selector_out,same_out,bounds_provenance)
    if args.include_aggregate_bounds:
        aggregate_out,aggregate_controls,aggregate_provenance=read_aggregate_bounds(ROOT / 'target')
        artifact=attach_aggregate_bounds(artifact,aggregate_out,aggregate_controls,bounds_out,aggregate_provenance)
    if args.include_synthetic_cpu_deformation:
        deform_out,deform_selector,deform_palette,deform_provenance=read_synthetic_deformation(ROOT / 'target')
        artifact=attach_synthetic_deformation(artifact,deform_out,deform_selector,deform_palette,same_out,deform_provenance)
    with args.output.open('x', encoding='utf-8', newline='\n') as f:
        json.dump(artifact, f, indent=2, allow_nan=False)
        f.write('\n')
    print(f'Wrote {args.output}: 44 positions, 44 normals, 84 index triples. {LIMITS}')

if __name__ == '__main__':
    main()
