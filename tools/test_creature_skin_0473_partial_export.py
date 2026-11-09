from pathlib import Path
import struct
import tempfile
import unittest
import creature_skin_0473_partial_export as exporter


def synthetic_streams():
    average = bytearray(704)
    normal = bytearray(1552)
    indices = bytearray(3036)
    for j in range(44):
        struct.pack_into('<3f', average, j * 16 + 4, j + 0.25, -j, j + 0.5)
        struct.pack_into('<3f', normal, j * 12, 0.0, 1.0, 0.0)
    struct.pack_into('<I', indices, 2024, 252)
    struct.pack_into('<252I', indices, 2028, *(j % 44 for j in range(252)))
    return {'average': bytes(average), 'normal': bytes(normal), 'indices': bytes(indices)}


def fixture_available(root):
    return all((root / folder / name).is_file()
               for folder, _, _, _ in exporter.SPECS.values()
               for name in ('result.json', 'host-stdout.bin'))


class ExportTests(unittest.TestCase):
    def test_synthetic_composition_and_rejections(self):
        streams = synthetic_streams()
        artifact = exporter.compose(streams, {'fixture': 'synthetic'})
        self.assertEqual((len(artifact['positions']), len(artifact['normals']), len(artifact['triangles'])), (44, 44, 84))
        self.assertEqual(artifact['positions'][1], (1.25, -1.0, 1.5))
        self.assertEqual(artifact['normals'][1], (0.0, 1.0, 0.0))
        self.assertEqual(artifact['triangles'][0], (0, 1, 2))
        self.assertNotIn('uvs', artifact)
        for key, offset, word in [('average', 4, 0x7f800000), ('normal', 0, 0x7fc00000),
                                  ('indices', 2024, 251), ('indices', 2028, 44)]:
            damaged = dict(streams)
            data = bytearray(damaged[key]); struct.pack_into('<I', data, offset, word); damaged[key] = bytes(data)
            with self.subTest(key=key, offset=offset), self.assertRaises(ValueError):
                exporter.compose(damaged, {})
        damaged = dict(streams); damaged['normal'] = damaged['normal'][:-1]
        with self.assertRaises(ValueError):
            exporter.compose(damaged, {})

    def test_synthetic_optional_records_preserve_bits_and_reject_invalid(self):
        artifact = exporter.compose(synthetic_streams(), {})
        self.assertNotIn('conditional_synthetic_coordinate_initial_records', artifact)
        data = bytearray(880)
        for j in range(44):
            struct.pack_into('<4B4I', data, j * 20, 255, 1, 2, 3, 0x80000000, 0x3f800000, 0, 0x3e800000)
        streams = {'synthetic_initial_records': bytes(data)}
        provenance = {'synthetic_initial_records': {'receipt_sha256': 'synthetic'}}
        result = exporter.attach_synthetic_records(artifact, streams, provenance)
        section = result['conditional_synthetic_coordinate_initial_records']
        self.assertEqual(len(section['records_by_position']), 44)
        self.assertEqual(section['records_by_position'][0]['raw_index_bytes'], [255, 1, 2, 3])
        self.assertEqual(section['records_by_position'][0]['raw_weight_words_hex'], ['80000000', '3f800000', '00000000', '3e800000'])
        self.assertIn('not palette', section['limits'])
        for damaged in (bytes(data[:-1]), bytes(data[:4] + struct.pack('<I', 0x7fc00000) + data[8:])):
            with self.assertRaises(ValueError):
                exporter.attach_synthetic_records(artifact, {'synthetic_initial_records': damaged}, provenance)

    @unittest.skipUnless(fixture_available(exporter.ROOT / 'target') and all(
        (exporter.ROOT / 'target' / folder / name).is_file()
        for folder, _, _, _ in exporter.RECORD_SPEC.values()
        for name in ('result.json', 'host-stdout.bin')), 'pinned geometry or record fixture is unavailable')
    def test_pinned_optional_records(self):
        artifact = exporter.compose(*exporter.read_inputs(exporter.ROOT / 'target'))
        streams, provenance = exporter.read_inputs(exporter.ROOT / 'target', exporter.RECORD_SPEC)
        section = exporter.attach_synthetic_records(artifact, streams, provenance)['conditional_synthetic_coordinate_initial_records']
        self.assertEqual(len(section['records_by_position']), 44)
        self.assertEqual(section['records_by_position'][0]['raw_index_bytes'], [2, 1, 3, 0])
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            folder = exporter.RECORD_SPEC['synthetic_initial_records'][0]
            directory = root / folder; directory.mkdir()
            original = exporter.ROOT / 'target' / folder
            for name in ('result.json', 'host-stdout.bin'):
                (directory / name).write_bytes((original / name).read_bytes())
            path = directory / 'host-stdout.bin'
            data = bytearray(path.read_bytes()); data[0] ^= 1; path.write_bytes(data)
            with self.assertRaisesRegex(ValueError, 'hash mismatch'):
                exporter.read_inputs(root, exporter.RECORD_SPEC)
        self.assertEqual(section['records_by_position'][0]['raw_weight_words_hex'][0],
                         f"{struct.unpack_from('<I', streams['synthetic_initial_records'], 4)[0]:08x}")

    def test_row_records_synthetic_bits_and_rejections(self):
        artifact = exporter.compose(synthetic_streams(), {})
        data = bytearray(880)
        for j in range(44):
            struct.pack_into('<4B4I', data, j * 20, 9, 8, 7, 6, 0x80000000, 0x3f800000, 0, 0x3e800000)
        result = exporter.attach_row_records(artifact, bytes(data), {'receipt_sha256': 'synthetic'})
        section = result['conditional_row_coordinate_initial_records']
        self.assertEqual(section['records_by_position'][0]['raw_index_bytes'], [9, 8, 7, 6])
        self.assertEqual(section['records_by_position'][0]['raw_weight_words_hex'][0], '80000000')
        self.assertIn('not palette', section['limits'])
        for damaged in (bytes(data[:-1]), bytes(data[:4] + struct.pack('<I', 0x7f800000) + data[8:])):
            with self.assertRaises(ValueError): exporter.attach_row_records(artifact, damaged, {})

    @unittest.skipUnless(fixture_available(exporter.ROOT / 'target') and all(
        (exporter.ROOT / 'target' / exporter.ROW_FOLDER / name).is_file()
        for name in ('result.json', 'record-stdout.bin')), 'pinned geometry or row record fixture is unavailable')
    def test_pinned_row_records_and_tamper(self):
        artifact = exporter.compose(*exporter.read_inputs(exporter.ROOT / 'target'))
        data, provenance = exporter.read_row_records(exporter.ROOT / 'target')
        section = exporter.attach_row_records(artifact, data, provenance)['conditional_row_coordinate_initial_records']
        self.assertEqual(len(section['records_by_position']), 44)
        self.assertEqual(section['records_by_position'][0]['raw_index_bytes'], list(data[:4]))
        self.assertEqual(section['records_by_position'][0]['raw_weight_words_hex'][0], f"{struct.unpack_from('<I', data, 4)[0]:08x}")
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); directory = root / exporter.ROW_FOLDER; directory.mkdir()
            original = exporter.ROOT / 'target' / exporter.ROW_FOLDER
            for name in ('result.json', 'record-stdout.bin'):
                (directory / name).write_bytes((original / name).read_bytes())
            path = directory / 'record-stdout.bin'; b = bytearray(path.read_bytes()); b[0] ^= 1; path.write_bytes(b)
            with self.assertRaisesRegex(ValueError, 'hash mismatch'): exporter.read_row_records(root)

    def test_smoothing_selection_and_rejections(self):
        artifact = exporter.compose(synthetic_streams(), {})
        data = bytearray(2640)
        for iteration in range(3):
            for j in range(44):
                struct.pack_into('<4B4I', data, iteration * 880 + j * 20, iteration, 1, 2, 3, 0x80000000, 0x3f800000, 0, 0)
        section = exporter.attach_smoothed_records(artifact, data, {'source': 'synthetic'}, 2)['conditional_half_alpha_smoothed_records']
        self.assertEqual(section['records_by_position'][0]['raw_index_bytes'][0], 2)
        self.assertEqual(section['records_by_position'][0]['raw_weight_words_hex'][0], '80000000')
        self.assertEqual((section['explicit_passes'], section['alpha_f32_bits_hex']), (2, '3f000000'))
        for count in (-1, 3, True):
            with self.assertRaises(ValueError): exporter.attach_smoothed_records(artifact, data, {}, count)
        with self.assertRaises(ValueError): exporter.attach_smoothed_records(artifact, data[:-1], {}, 0)
        struct.pack_into('<I', data, 1764, 0x7f800000)
        with self.assertRaises(ValueError): exporter.attach_smoothed_records(artifact, data, {}, 2)

    @unittest.skipUnless(fixture_available(exporter.ROOT / 'target') and all(
        (exporter.ROOT / 'target' / exporter.SMOOTH_FOLDER / name).is_file()
        for name in ('result.json', 'host-stdout.bin', 'model-stdout.bin')), 'pinned smoothing fixture is unavailable')
    def test_pinned_smoothing_and_tamper(self):
        artifact = exporter.compose(*exporter.read_inputs(exporter.ROOT / 'target'))
        data, provenance = exporter.read_smoothed_records(exporter.ROOT / 'target')
        section = exporter.attach_smoothed_records(artifact, data, provenance, 1)['conditional_half_alpha_smoothed_records']
        self.assertEqual(section['records_by_position'][0]['raw_index_bytes'], list(data[880:884]))
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); directory = root / exporter.SMOOTH_FOLDER; directory.mkdir()
            original = exporter.ROOT / 'target' / exporter.SMOOTH_FOLDER
            for name in ('result.json', 'host-stdout.bin', 'model-stdout.bin'):
                (directory / name).write_bytes((original / name).read_bytes())
            path = directory / 'host-stdout.bin'; b = bytearray(path.read_bytes()); b[0] ^= 1; path.write_bytes(b)
            with self.assertRaisesRegex(ValueError, 'hash mismatch'): exporter.read_smoothed_records(root)

    def test_caller_smoothing_synthetic_and_rejections(self):
        artifact = exporter.compose(synthetic_streams(), {})
        data = bytearray(2640)
        for iteration in range(3):
            for j in range(44):
                struct.pack_into('<4B4I', data, iteration * 880 + j * 20, iteration, 1, 2, 3, 0x80000000, 0x3f800000, 0, 0)
        section = exporter.attach_caller_smoothed_records(artifact, data, {}, 2)['conditional_caller_alpha_smoothed_records']
        self.assertEqual(section['records_by_position'][0]['raw_index_bytes'][0], 2)
        self.assertEqual(section['records_by_position'][0]['raw_weight_words_hex'][0], '80000000')
        self.assertEqual((section['explicit_passes'], section['alpha_f32_bits_hex']), (2, '3f333333'))
        self.assertIn('unobserved', section['limits'])
        for count in (-1, 3, True):
            with self.assertRaises(ValueError): exporter.attach_caller_smoothed_records(artifact, data, {}, count)
        with self.assertRaises(ValueError): exporter.attach_caller_smoothed_records(artifact, data[:-1], {}, 0)
        struct.pack_into('<I', data, 1764, 0x7f800000)
        with self.assertRaises(ValueError): exporter.attach_caller_smoothed_records(artifact, data, {}, 2)

    @unittest.skipUnless(fixture_available(exporter.ROOT / 'target') and all(
        (exporter.ROOT / 'target' / exporter.CALLER_SMOOTH_FOLDER / name).is_file()
        for name in ('result.json', 'host-stdout.bin', 'model-stdout.bin')), 'caller smoothing fixture is unavailable')
    def test_pinned_caller_smoothing_and_tamper(self):
        artifact = exporter.compose(*exporter.read_inputs(exporter.ROOT / 'target'))
        data, provenance = exporter.read_caller_smoothed_records(exporter.ROOT / 'target')
        for passes in range(3):
            section = exporter.attach_caller_smoothed_records(artifact, data, provenance, passes)['conditional_caller_alpha_smoothed_records']
            self.assertEqual(section['records_by_position'][0]['raw_index_bytes'], list(data[passes*880:passes*880+4]))
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); directory = root / exporter.CALLER_SMOOTH_FOLDER; directory.mkdir()
            original = exporter.ROOT / 'target' / exporter.CALLER_SMOOTH_FOLDER
            for name in ('result.json', 'host-stdout.bin', 'model-stdout.bin'):
                (directory / name).write_bytes((original / name).read_bytes())
            path = directory / 'model-stdout.bin'; b = bytearray(path.read_bytes()); b[0] ^= 1; path.write_bytes(b)
            with self.assertRaisesRegex(ValueError, 'hash mismatch'): exporter.read_caller_smoothed_records(root)

    def test_synthetic_corner_join_and_rejections(self):
        artifact = exporter.compose(synthetic_streams(), {})
        records = bytearray(2640); output = bytearray(3168)
        for i in range(132):
            struct.pack_into('<4B4I', records, i*20, i%44, 1, 2, 3, 0x80000000, 0x3f800000, 0, 0)
        for i in range(132):
            j = (i//44)*44 + (i*17)%44; raw = records[j*20:j*20+20]
            struct.pack_into('<4H', output, i*24, *(3*x for x in raw[:4]))
            output[i*24+8:i*24+24] = raw[4:]
        section = exporter.attach_synthetic_corner_attributes(artifact, output, {}, records, 1)['conditional_synthetic_corner_attributes']
        self.assertEqual(section['corners'][1]['position_index'], 17)
        self.assertEqual(section['corners'][1]['raw_u16_offsets'][0], 51)
        self.assertEqual(section['corners'][1]['raw_weight_words_hex'][0], '80000000')
        self.assertIn('not observed', section['limits'])
        with self.assertRaises(ValueError): exporter.attach_synthetic_corner_attributes(artifact, output, {}, records, 3)
        output[44*24] ^= 1
        with self.assertRaises(ValueError): exporter.attach_synthetic_corner_attributes(artifact, output, {}, records, 1)

    @unittest.skipUnless(fixture_available(exporter.ROOT / 'target') and all(
        (exporter.ROOT / 'target' / folder / name).is_file()
        for folder, name in [(exporter.CORNER_FOLDER,'result.json'), (exporter.CORNER_FOLDER,'host-stdout.bin'),
                             (exporter.CALLER_SMOOTH_FOLDER,'result.json'), (exporter.CALLER_SMOOTH_FOLDER,'host-stdout.bin'),
                             (exporter.CALLER_SMOOTH_FOLDER,'model-stdout.bin')]), 'corner fixture is unavailable')
    def test_pinned_synthetic_corner_and_tamper(self):
        root = exporter.ROOT / 'target'; artifact = exporter.compose(*exporter.read_inputs(root))
        data, provenance = exporter.read_synthetic_corner_attributes(root)
        records, _ = exporter.read_caller_smoothed_records(root)
        for passes in range(3):
            section = exporter.attach_synthetic_corner_attributes(artifact, data, provenance, records, passes)['conditional_synthetic_corner_attributes']
            self.assertEqual(len(section['corners']), 44)
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); directory = root / exporter.CORNER_FOLDER; directory.mkdir()
            original = exporter.ROOT / 'target' / exporter.CORNER_FOLDER
            for name in ('result.json', 'host-stdout.bin'): (directory / name).write_bytes((original / name).read_bytes())
            path = directory / 'host-stdout.bin'; b = bytearray(path.read_bytes()); b[0] ^= 1; path.write_bytes(b)
            with self.assertRaisesRegex(ValueError, 'hash mismatch'): exporter.read_synthetic_corner_attributes(root)

    def test_fixture_availability_detection(self):
        with tempfile.TemporaryDirectory() as temp:
            self.assertFalse(fixture_available(Path(temp)))

    @unittest.skipUnless(fixture_available(exporter.ROOT / 'target'), 'pinned target fixture is unavailable')
    def test_pinned_fixture_and_tamper_rejection(self):
        streams, provenance = exporter.read_inputs(exporter.ROOT / 'target')
        artifact = exporter.compose(streams, provenance)
        self.assertEqual((len(artifact['positions']), len(artifact['normals']), len(artifact['triangles'])), (44, 44, 84))
        self.assertEqual({x for t in artifact['triangles'] for x in t}, set(range(44)))
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for folder, _, _, _ in exporter.SPECS.values():
                directory = root / folder; directory.mkdir()
                original = exporter.ROOT / 'target' / folder
                for name in ('result.json', 'host-stdout.bin'):
                    (directory / name).write_bytes((original / name).read_bytes())
            path = root / exporter.SPECS['normal'][0] / 'host-stdout.bin'
            data = bytearray(path.read_bytes()); data[0] ^= 1; path.write_bytes(data)
            with self.assertRaisesRegex(ValueError, 'hash mismatch'):
                exporter.read_inputs(root)

    def test_multigroup_synthetic_and_rejections(self):
        artifact = exporter.compose(synthetic_streams(), {})
        self.assertNotIn('conditional_model_only_uv_groups', artifact)
        inputs={'positions.bin':b''.join(struct.pack('<3f',*p) for p in artifact['positions']),
                'indices.bin':b''.join(struct.pack('<3I',*t) for t in artifact['triangles'])}
        words=[8,85,44,252]; faces_by_group=[list(range(i*10,(i+1)*10)) for i in range(7)]+[list(range(70,84))]
        ends=[10,20,30,40,50,60,70,85]
        for j,faces in enumerate(faces_by_group): words += [4+j,faces[0],j%6,ends[j],len(faces),0,0,0,0,*faces]
        uv_start=len(words);words += [0,0]*85
        for j,end in enumerate(ends):
            start=ends[j-1] if j else 0
            for i in range(start,end): words += [4+j,0]
        words += [4,0]*44
        for j,faces in enumerate(faces_by_group): words += [0 if j==0 else ends[j-1]]*(len(faces)*3)
        words += [((~(4+j))&0xffffffff) for j,faces in enumerate(faces_by_group) for _ in faces]
        data=struct.pack('<924I',*words)
        section=exporter.attach_multigroup(artifact,data,inputs,{'source':'synthetic'})['conditional_model_only_uv_groups']
        self.assertEqual((len(section['groups']),len(section['uv_words_hex']),len(section['corner_uv_indices'])),(8,85,252))
        self.assertIn('Model-derived',section['limits'])
        for offset,value in [(0,9),(uv_start,0x7f800000),(len(words)-1,0)]:
            bad=bytearray(data);struct.pack_into('<I',bad,offset*4,value)
            with self.assertRaises(ValueError):exporter.attach_multigroup(artifact,bad,inputs,{})
        bad=dict(inputs);bad['indices.bin']=b'bad'
        with self.assertRaises(ValueError):exporter.attach_multigroup(artifact,data,bad,{})
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaises(FileNotFoundError):exporter.read_multigroup(Path(temp))

    @unittest.skipUnless(fixture_available(exporter.ROOT/'target') and all(
       (exporter.ROOT/'target'/folder/name).is_file() for folder,name in [
       (exporter.MULTI_FOLDER,'result.json'),(exporter.MULTI_FOLDER,'rust-stdout.bin'),
       (exporter.MULTI_MODEL_FOLDER,'model-only.bin'),(exporter.MULTI_NATIVE_FOLDER,'result.json'),
       (exporter.MULTI_NATIVE_FOLDER,'host-stdout.bin'),(exporter.MULTI_DIRECT_FOLDER,'result.json'),
       *[(exporter.MULTI_INPUT_FOLDER,n) for n in exporter.MULTI_INPUT_PINS]]), 'multi-group fixture unavailable')
    def test_multigroup_pinned_fixture_and_tamper(self):
        root=exporter.ROOT/'target';artifact=exporter.compose(*exporter.read_inputs(root))
        data,inputs,provenance=exporter.read_multigroup(root)
        section=exporter.attach_multigroup(artifact,data,inputs,provenance)['conditional_model_only_uv_groups']
        self.assertEqual([g['seed_face'] for g in section['groups']],[0,52,79,31,27,63,14,29])
        self.assertEqual(provenance['native_receipt_sha256'], exporter.MULTI_NATIVE_RECEIPT_SHA)
        self.assertEqual(provenance['direct_comparison_receipt_sha256'], exporter.MULTI_DIRECT_RECEIPT_SHA)
        self.assertIn('synthetic', section['limits'])
        with tempfile.TemporaryDirectory() as temp:
            dest=Path(temp)
            for folder,name in [(exporter.MULTI_FOLDER,'result.json'),(exporter.MULTI_FOLDER,'rust-stdout.bin'),(exporter.MULTI_MODEL_FOLDER,'model-only.bin'),(exporter.MULTI_NATIVE_FOLDER,'result.json'),(exporter.MULTI_NATIVE_FOLDER,'host-stdout.bin'),(exporter.MULTI_DIRECT_FOLDER,'result.json'),*[(exporter.MULTI_INPUT_FOLDER,n) for n in exporter.MULTI_INPUT_PINS]]:
                d=dest/folder;d.mkdir(exist_ok=True);(d/name).write_bytes((root/folder/name).read_bytes())
            for folder,name in [(exporter.MULTI_FOLDER,'rust-stdout.bin'),(exporter.MULTI_NATIVE_FOLDER,'host-stdout.bin'),(exporter.MULTI_NATIVE_FOLDER,'result.json'),(exporter.MULTI_DIRECT_FOLDER,'result.json')]:
                path=dest/folder/name;original=path.read_bytes();b=bytearray(original);b[0]^=1;path.write_bytes(b)
                with self.subTest(folder=folder,name=name),self.assertRaisesRegex(ValueError,'hash mismatch'):exporter.read_multigroup(dest)
                path.write_bytes(original)
            for folder,name in [(exporter.MULTI_NATIVE_FOLDER,'result.json'),(exporter.MULTI_NATIVE_FOLDER,'host-stdout.bin'),(exporter.MULTI_DIRECT_FOLDER,'result.json')]:
                path=dest/folder/name;original=path.read_bytes();path.unlink()
                with self.subTest(missing=name,folder=folder),self.assertRaises(FileNotFoundError):exporter.read_multigroup(dest)
                path.write_bytes(original)

    def test_transformed_uv_synthetic_and_rejections(self):
        artifact=exporter.compose(synthetic_streams(),{})
        self.assertNotIn('conditional_model_only_transformed_uvs',artifact)
        artifact['conditional_model_only_uv_groups']={
          'uv_words_hex':[['00000000','00000000'] for _ in range(85)],
          'uv_metadata':[{'model_group_id':4+i%8} for i in range(85)],
          'corner_uv_indices':[0]*252}
        shelf=[0]*386;shelf[0]=4
        for i in range(4):
            start=13+15*i;shelf[start]=2;shelf[start+3]=2*i;shelf[start+4]=2*i+1
        sources={'atlas':bytes(264),'shelf':struct.pack('<386I',*shelf),
                 'sort':struct.pack('<48I',*(list(range(8))+[0]*40)),'offset':bytes(264)}
        rows=[[0,0,0,0,0,0,0,0x3f000000] for _ in range(88)]
        cases=struct.pack('<704I',*(x for row in rows for x in row))
        values=[0]*176;values[0]=0x80000000;data=struct.pack('<176I',*values)
        section=exporter.attach_transformed_uvs(artifact,data,cases,sources,{'source':'synthetic'})['conditional_model_only_transformed_uvs']
        self.assertEqual((len(section['uv_words_hex']),len(section['corner_uv_indices'])),(85,252))
        self.assertEqual(section['uv_words_hex'][0][0],'80000000')
        self.assertEqual(struct.pack('<f',section['uv'][0][0]),struct.pack('<I',0x80000000))
        self.assertEqual(section['synthetic_scale_f32_bits_hex'],'3f000000')
        self.assertIn('not final UVs',section['limits'])
        for index,value in [(0,0x7f800000),(175,0x7fc00000)]:
            bad=bytearray(data);struct.pack_into('<I',bad,index*4,value)
            with self.assertRaises(ValueError):exporter.attach_transformed_uvs(artifact,bad,cases,sources,{})
        for index,value in [(0,1),(5,1),(7,0x3f800000)]:
            bad=bytearray(cases);struct.pack_into('<I',bad,index*4,value)
            with self.assertRaises(ValueError):exporter.attach_transformed_uvs(artifact,data,bad,sources,{})
        bad=dict(sources);bad['shelf']=bad['shelf'][:-4]
        with self.assertRaises(ValueError):exporter.attach_transformed_uvs(artifact,data,cases,bad,{})
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaises(FileNotFoundError):exporter.read_transformed_uvs(Path(temp))

    @unittest.skipUnless(fixture_available(exporter.ROOT/'target') and all(
       (exporter.ROOT/'target'/folder/name).is_file() for folder,name in [
       (exporter.MULTI_FOLDER,'result.json'),(exporter.MULTI_FOLDER,'rust-stdout.bin'),
       (exporter.MULTI_MODEL_FOLDER,'model-only.bin'),(exporter.MULTI_NATIVE_FOLDER,'result.json'),
       (exporter.MULTI_NATIVE_FOLDER,'host-stdout.bin'),(exporter.MULTI_DIRECT_FOLDER,'result.json'),
       *[(exporter.MULTI_INPUT_FOLDER,n) for n in exporter.MULTI_INPUT_PINS],
       *[(folder,name) for folder,_,_,_ in exporter.JOIN_INPUTS.values() for name in ('result.json','host-stdout.bin')],
       (exporter.JOIN_NATIVE_FOLDER,'result.json'),(exporter.JOIN_NATIVE_FOLDER,'host-stdout.bin'),(exporter.JOIN_NATIVE_FOLDER,'cases.bin'),
       (exporter.JOIN_RUST_FOLDER,'result.json'),(exporter.JOIN_RUST_FOLDER,'uv-stdout.bin'),
       (exporter.JOIN_RUST_FOLDER,'joined-cases.bin'),(exporter.JOIN_RUST_FOLDER,'offset-stdout.bin')]), 'transformed UV fixture unavailable')
    def test_transformed_uv_pinned_and_tamper(self):
        root=exporter.ROOT/'target';artifact=exporter.compose(*exporter.read_inputs(root))
        exporter.attach_multigroup(artifact,*exporter.read_multigroup(root))
        data,cases,sources,provenance=exporter.read_transformed_uvs(root)
        section=exporter.attach_transformed_uvs(artifact,data,cases,sources,provenance)['conditional_model_only_transformed_uvs']
        self.assertEqual(len(section['uv_words_hex']),85)
        self.assertEqual(section['uv_words_hex'][0],[f'{x:08x}' for x in struct.unpack_from('<2I',data)])
        with tempfile.TemporaryDirectory() as temp:
            dest=Path(temp);files=[*[(folder,name) for folder,_,_,_ in exporter.JOIN_INPUTS.values() for name in ('result.json','host-stdout.bin')],
              (exporter.JOIN_NATIVE_FOLDER,'result.json'),(exporter.JOIN_NATIVE_FOLDER,'host-stdout.bin'),(exporter.JOIN_NATIVE_FOLDER,'cases.bin'),
              (exporter.JOIN_RUST_FOLDER,'result.json'),(exporter.JOIN_RUST_FOLDER,'uv-stdout.bin'),
              (exporter.JOIN_RUST_FOLDER,'joined-cases.bin'),(exporter.JOIN_RUST_FOLDER,'offset-stdout.bin')]
            for folder,name in files:
                d=dest/folder;d.mkdir(exist_ok=True);(d/name).write_bytes((root/folder/name).read_bytes())
            for folder,name in [(exporter.JOIN_NATIVE_FOLDER,'result.json'),(exporter.JOIN_NATIVE_FOLDER,'host-stdout.bin'),
                                (exporter.JOIN_RUST_FOLDER,'result.json'),(exporter.JOIN_RUST_FOLDER,'uv-stdout.bin'),
                                (exporter.JOIN_RUST_FOLDER,'joined-cases.bin'),(exporter.JOIN_INPUTS['offset'][0],'host-stdout.bin')]:
                path=dest/folder/name;original=path.read_bytes();bad=bytearray(original);bad[0]^=1;path.write_bytes(bad)
                with self.subTest(folder=folder,name=name),self.assertRaisesRegex(ValueError,'hash mismatch'):exporter.read_transformed_uvs(dest)
                path.write_bytes(original)
            path=dest/exporter.JOIN_NATIVE_FOLDER/'result.json';path.unlink()
            with self.assertRaises(FileNotFoundError):exporter.read_transformed_uvs(dest)

    def test_paired_uv_missing_and_mapping_rejection(self):
        artifact=exporter.compose(synthetic_streams(),{})
        self.assertNotIn('conditional_paired_transformed_uvs',artifact)
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaises(FileNotFoundError):exporter.read_paired_uvs(Path(temp))
        sources={'atlas':bytes(264),'sort':struct.pack('<48I',*(list(range(8))+[0]*40)),
                 'pair':bytes(2340),'offset':bytes(192),'offset_inputs':bytes(256)}
        with self.assertRaises(ValueError):exporter.attach_paired_uvs(artifact,bytes(704),bytes(2816),sources,{})

    @unittest.skipUnless(fixture_available(exporter.ROOT/'target') and all(
       (exporter.ROOT/'target'/folder/name).is_file() for folder,name in [
       (exporter.MULTI_FOLDER,'result.json'),(exporter.MULTI_FOLDER,'rust-stdout.bin'),
       (exporter.MULTI_MODEL_FOLDER,'model-only.bin'),(exporter.MULTI_NATIVE_FOLDER,'result.json'),
       (exporter.MULTI_NATIVE_FOLDER,'host-stdout.bin'),(exporter.MULTI_DIRECT_FOLDER,'result.json'),
       *[(exporter.MULTI_INPUT_FOLDER,n) for n in exporter.MULTI_INPUT_PINS],
       *[(folder,name) for folder,_,_,_ in exporter.PAIRED_INPUTS.values() for name in ('result.json','host-stdout.bin')],
       (exporter.PAIRED_INPUTS['offset'][0],'cases.bin'),
       *[(exporter.PAIRED_NATIVE,n) for n in ('result.json','host-stdout.bin','cases.bin')],
       *[(exporter.PAIRED_RUST,n) for n in ('result.json','uv-stdout.bin','joined-cases.bin','pair-stdout.bin','offset-stdout.bin')]]), 'paired UV fixture unavailable')
    def test_paired_uv_fixture_and_tampering(self):
        root=exporter.ROOT/'target';artifact=exporter.compose(*exporter.read_inputs(root));exporter.attach_multigroup(artifact,*exporter.read_multigroup(root))
        data,cases,sources,provenance=exporter.read_paired_uvs(root)
        section=exporter.attach_paired_uvs(artifact,data,cases,sources,provenance)['conditional_paired_transformed_uvs']
        self.assertEqual(len(section['uv_words_hex']),85);self.assertEqual(section['conditional_source_scale_f32_bits_hex'],'3ee3db63')
        self.assertIn('synthetic',section['limits']);self.assertIn('not final UVs',section['limits'])
        bad=dict(sources);x=bytearray(bad['offset_inputs']);x[0]^=1;bad['offset_inputs']=bytes(x)
        with self.assertRaisesRegex(ValueError,'inputs mismatch'):exporter.attach_paired_uvs(artifact,data,cases,bad,{})
        with tempfile.TemporaryDirectory() as temp:
            dest=Path(temp);files=[*[(folder,name) for folder,_,_,_ in exporter.PAIRED_INPUTS.values() for name in ('result.json','host-stdout.bin')],
              (exporter.PAIRED_INPUTS['offset'][0],'cases.bin'),*[(exporter.PAIRED_NATIVE,n) for n in ('result.json','host-stdout.bin','cases.bin')],
              *[(exporter.PAIRED_RUST,n) for n in ('result.json','uv-stdout.bin','joined-cases.bin','pair-stdout.bin','offset-stdout.bin')]]
            for folder,name in files:
                d=dest/folder;d.mkdir(exist_ok=True);(d/name).write_bytes((root/folder/name).read_bytes())
            for folder,name in [(exporter.PAIRED_NATIVE,'result.json'),(exporter.PAIRED_RUST,'uv-stdout.bin'),(exporter.PAIRED_RUST,'pair-stdout.bin'),(exporter.PAIRED_INPUTS['offset'][0],'cases.bin')]:
                path=dest/folder/name;original=path.read_bytes();bad=bytearray(original);bad[0]^=1;path.write_bytes(bad)
                with self.subTest(folder=folder,name=name),self.assertRaisesRegex(ValueError,'hash mismatch'):exporter.read_paired_uvs(dest)
                path.write_bytes(original)

    def test_twelve_uv_missing_and_invalid_shape(self):
        artifact=exporter.compose(synthetic_streams(),{})
        self.assertNotIn('conditional_twelve_group_transformed_uvs',artifact)
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaises(FileNotFoundError):exporter.read_twelve_uvs(Path(temp))
        with self.assertRaises(ValueError):exporter.attach_twelve_uvs(artifact,bytes(704),bytes(2816),{}, {})

    @unittest.skipUnless(fixture_available(exporter.ROOT/'target') and all(
       (exporter.ROOT/'target'/folder/name).is_file() for folder,name in [
       (exporter.MULTI_FOLDER,'result.json'),(exporter.MULTI_FOLDER,'rust-stdout.bin'),(exporter.MULTI_MODEL_FOLDER,'model-only.bin'),
       (exporter.MULTI_NATIVE_FOLDER,'result.json'),(exporter.MULTI_NATIVE_FOLDER,'host-stdout.bin'),(exporter.MULTI_DIRECT_FOLDER,'result.json'),
       *[(exporter.MULTI_INPUT_FOLDER,n) for n in exporter.MULTI_INPUT_PINS],
       *[(folder,name) for folder,_,_,_ in exporter.TWELVE_INPUTS.values() for name in ('result.json','host-stdout.bin')],
       *[(exporter.TWELVE_INPUTS[k][0],'cases.bin') for k in ('atlas','shelf','offset')],
       *[(exporter.TWELVE_NATIVE,n) for n in ('result.json','host-stdout.bin','cases.bin')],
       *[(exporter.TWELVE_RUST,n) for n in ('result.json','uv-stdout.bin','joined-cases.bin','pair-stdout.bin','offset-stdout.bin')]]), 'twelve UV fixture unavailable')
    def test_twelve_uv_fixture_and_tampering(self):
        root=exporter.ROOT/'target';artifact=exporter.compose(*exporter.read_inputs(root));exporter.attach_multigroup(artifact,*exporter.read_multigroup(root))
        data,cases,sources,provenance=exporter.read_twelve_uvs(root);section=exporter.attach_twelve_uvs(artifact,data,cases,sources,provenance)['conditional_twelve_group_transformed_uvs']
        self.assertEqual(len(section['uv_words_hex']),85);self.assertEqual(section['conditional_source_scale_f32_bits_hex'],'3f2c2996');self.assertIn('not final UVs',section['limits']);self.assertIn('frozen',section['limits'])
        for key in ('offset_inputs','atlas_inputs','shelf_inputs'):
            bad=dict(sources);x=bytearray(bad[key]);x[0]^=1;bad[key]=bytes(x)
            with self.subTest(key=key),self.assertRaises(ValueError):exporter.attach_twelve_uvs(artifact,data,cases,bad,{})
        with tempfile.TemporaryDirectory() as temp:
            dest=Path(temp);files=[*[(folder,name) for folder,_,_,_ in exporter.TWELVE_INPUTS.values() for name in ('result.json','host-stdout.bin')],
              *[(exporter.TWELVE_INPUTS[k][0],'cases.bin') for k in ('atlas','shelf','offset')],
              *[(exporter.TWELVE_NATIVE,n) for n in ('result.json','host-stdout.bin','cases.bin')],
              *[(exporter.TWELVE_RUST,n) for n in ('result.json','uv-stdout.bin','joined-cases.bin','pair-stdout.bin','offset-stdout.bin')]]
            for folder,name in files:
                d=dest/folder;d.mkdir(exist_ok=True);(d/name).write_bytes((root/folder/name).read_bytes())
            for folder,name in [(exporter.TWELVE_NATIVE,'result.json'),(exporter.TWELVE_RUST,'uv-stdout.bin'),(exporter.TWELVE_RUST,'pair-stdout.bin'),(exporter.TWELVE_INPUTS['rows'][0],'host-stdout.bin'),(exporter.TWELVE_INPUTS['offset'][0],'cases.bin')]:
                path=dest/folder/name;original=path.read_bytes();bad=bytearray(original);bad[0]^=1;path.write_bytes(bad)
                with self.subTest(folder=folder,name=name),self.assertRaisesRegex(ValueError,'hash mismatch'):exporter.read_twelve_uvs(dest)
                path.write_bytes(original)

    def test_tangent_pairs_synthetic_and_rejections(self):
        artifact=exporter.compose(synthetic_streams(),{});self.assertNotIn('conditional_tangent_pairs',artifact)
        corners=list(range(85))+[0]*167
        artifact['conditional_twelve_group_transformed_uvs']={'uv_words_hex':[['00000000','00000000'] for _ in range(85)],'corner_uv_indices':corners}
        artifact['conditional_model_only_uv_groups']={'uv_metadata':[{'conditional_corner_ordinal':i} for i in range(85)]}
        artifact['normals']=[[0.,0.,0.] for _ in range(44)];artifact['triangles']=[[0,0,0] for _ in range(84)]
        data={(exporter.TANGENT_PREFIX,'uvs.bin'):bytes(680),(exporter.TANGENT_PREFIX,'corners.bin'):struct.pack('<252I',*corners),(exporter.TANGENT_PREFIX,'indices.bin'):struct.pack('<85I',*range(85)),(exporter.TANGENT_NATIVE,'normals.bin'):bytes(528),(exporter.TANGENT_NATIVE,'normal-indices.bin'):bytes(1008)}
        raw=bytearray(2040);struct.pack_into('<I',raw,0,0x80000000)
        section=exporter.attach_tangent_pairs(artifact,bytes(raw),data,{})['conditional_tangent_pairs']
        self.assertEqual((len(section['tangents']),len(section['bitangents'])),(85,85));self.assertEqual(section['tangent_words_hex'][0][0],'80000000');self.assertIn('+6C/+58',section['limits'])
        for index,value in [(0,0x7f800000),(509,0x7fc00000)]:
            bad=bytearray(raw);struct.pack_into('<I',bad,index*4,value)
            with self.assertRaises(ValueError):exporter.attach_tangent_pairs(artifact,bytes(bad),data,{})
        bad=dict(data);bad[exporter.TANGENT_PREFIX,'indices.bin']=struct.pack('<85I',1,*range(1,85))
        with self.assertRaisesRegex(ValueError,'mapping'):exporter.attach_tangent_pairs(artifact,bytes(raw),bad,{})
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaises(FileNotFoundError):exporter.read_tangent_pairs(Path(temp))

    @unittest.skipUnless(fixture_available(exporter.ROOT/'target') and all(
       (exporter.ROOT/'target'/folder/name).is_file() for folder,name in [
       (exporter.MULTI_FOLDER,'result.json'),(exporter.MULTI_FOLDER,'rust-stdout.bin'),(exporter.MULTI_MODEL_FOLDER,'model-only.bin'),
       (exporter.MULTI_NATIVE_FOLDER,'result.json'),(exporter.MULTI_NATIVE_FOLDER,'host-stdout.bin'),(exporter.MULTI_DIRECT_FOLDER,'result.json'),
       *[(exporter.MULTI_INPUT_FOLDER,n) for n in exporter.MULTI_INPUT_PINS],
       *[(folder,name) for folder,_,_,_ in exporter.TWELVE_INPUTS.values() for name in ('result.json','host-stdout.bin')],
       *[(exporter.TWELVE_INPUTS[k][0],'cases.bin') for k in ('atlas','shelf','offset')],
       *[(exporter.TWELVE_NATIVE,n) for n in ('result.json','host-stdout.bin','cases.bin')],
       *[(exporter.TWELVE_RUST,n) for n in ('result.json','uv-stdout.bin','joined-cases.bin','pair-stdout.bin','offset-stdout.bin')],
       *[(folder,name) for folder,files in exporter.TANGENT_PINS.items() for name in files]]), 'tangent fixture unavailable')
    def test_tangent_pairs_fixture_and_tampering(self):
        root=exporter.ROOT/'target';artifact=exporter.compose(*exporter.read_inputs(root));exporter.attach_multigroup(artifact,*exporter.read_multigroup(root));exporter.attach_twelve_uvs(artifact,*exporter.read_twelve_uvs(root))
        stream,data,provenance=exporter.read_tangent_pairs(root);section=exporter.attach_tangent_pairs(artifact,stream,data,provenance)['conditional_tangent_pairs'];self.assertEqual(len(section['tangent_words_hex']),85);self.assertEqual(section['tangent_words_hex'][0],[f'{x:08x}' for x in struct.unpack_from('<3I',stream)])
        bad=dict(data);x=bytearray(bad[exporter.TANGENT_NATIVE,'normal-indices.bin']);x[0]^=1;bad[exporter.TANGENT_NATIVE,'normal-indices.bin']=bytes(x)
        with self.assertRaisesRegex(ValueError,'normal-index'):exporter.attach_tangent_pairs(artifact,stream,bad,{})
        with tempfile.TemporaryDirectory() as temp:
            dest=Path(temp)
            for folder,files in exporter.TANGENT_PINS.items():
                d=dest/folder;d.mkdir(exist_ok=True)
                for name in files:(d/name).write_bytes((root/folder/name).read_bytes())
            for folder,name in [(exporter.TANGENT_NATIVE,'result.json'),(exporter.TANGENT_NATIVE,'normals.bin'),(exporter.TANGENT_PREFIX,'corners.bin'),(exporter.TANGENT_RUST,'rust-stdout.bin')]:
                path=dest/folder/name;original=path.read_bytes();bad=bytearray(original);bad[0]^=1;path.write_bytes(bad)
                with self.subTest(folder=folder,name=name),self.assertRaisesRegex(ValueError,'hash mismatch'):exporter.read_tangent_pairs(dest)
                path.write_bytes(original)

    def test_eight_group_missing_and_rejections(self):
        artifact=exporter.compose(synthetic_streams(),{})
        self.assertNotIn('conditional_eight_group_export',artifact)
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaises(FileNotFoundError):exporter.read_eight_export(Path(temp))
        with self.assertRaisesRegex(ValueError,'missing conditional'):exporter.attach_eight_export(artifact,bytes(3740),{}, {})

    @unittest.skipUnless(fixture_available(exporter.ROOT/'target') and all((exporter.ROOT/'target'/folder/name).is_file() for folder,files in exporter.EIGHT_PINS.items() for name in files), 'eight-group fixture unavailable')
    def test_eight_group_fixture_mapping_and_tampering(self):
        root=exporter.ROOT/'target'
        try:
            artifact=exporter.compose(*exporter.read_inputs(root));exporter.attach_multigroup(artifact,*exporter.read_multigroup(root));exporter.attach_twelve_uvs(artifact,*exporter.read_twelve_uvs(root));exporter.attach_tangent_pairs(artifact,*exporter.read_tangent_pairs(root))
        except FileNotFoundError:self.skipTest('upstream fixture unavailable')
        out,data,prov=exporter.read_eight_export(root);section=exporter.attach_eight_export(artifact,out,data,prov)['conditional_eight_group_export']
        self.assertEqual((len(section['groups']),len(section['positions']),sum(len(g['local_triangles']) for g in section['groups'])),(8,85,84));self.assertIn('four earlier',section['limits']);self.assertEqual(section['attribute_words_hex'][3][0],artifact['conditional_tangent_pairs']['tangent_words_hex'][struct.unpack_from('<3I',data[exporter.EIGHT_NATIVE,'triples.bin'])[2]])
        bad=dict(data);raw=bytearray(bad[exporter.EIGHT_TRIPLES,'host-stdout.bin']);struct.pack_into('<I',raw,135*4,0xffffffff);bad[exporter.EIGHT_TRIPLES,'host-stdout.bin']=bytes(raw)
        with self.assertRaisesRegex(ValueError,'local triple'):exporter.attach_eight_export(artifact,out,bad,{})
        with tempfile.TemporaryDirectory() as temp:
            dest=Path(temp)
            for folder,files in exporter.EIGHT_PINS.items():
                d=dest/folder;d.mkdir(exist_ok=True)
                for name in files:(d/name).write_bytes((root/folder/name).read_bytes())
            for folder,name in [(exporter.EIGHT_NATIVE,'result.json'),(exporter.EIGHT_NATIVE,'triples.bin'),(exporter.EIGHT_TRIPLES,'host-stdout.bin'),(exporter.EIGHT_RUST,'rust-stdout.bin')]:
                path=dest/folder/name;original=path.read_bytes();bad=bytearray(original);bad[0]^=1;path.write_bytes(bad)
                with self.subTest(folder=folder,name=name),self.assertRaisesRegex(ValueError,'hash mismatch'):exporter.read_eight_export(dest)
                path.write_bytes(original)

class SameObjectExportTests(unittest.TestCase):
    def test_synthetic_mapping_and_rejections(self):
        groups=[];offset=0
        for i in range(8):
            count=10 if i<7 else 15
            groups.append({'vertex_offset':offset,'vertex_count':count,'local_triangles':[[0,1,2]]*(11 if i<4 else 10)})
            offset+=count
        artifact={'conditional_eight_group_export':{'groups':groups,'attribute_words_hex':[[['00000000']*n for _ in range(85)] for n in (3,3,2,3)]}}
        indices=[g['vertex_offset']+x for g in groups for t in g['local_triangles'] for x in t]
        out=bytes(27468)+struct.pack('<252I',*indices);records=bytes(2640);triples=bytes(1020)
        section=exporter.attach_same_object_export(artifact,out,records,triples,{})['conditional_same_object_export']
        self.assertEqual((len(section['global_indices']),len(section['weight_words_hex'])),(252,85))
        self.assertEqual(section['explicit_passes'],2)
        bad=bytearray(out);bad[-1]^=1
        with self.assertRaisesRegex(ValueError,'index mismatch'):exporter.attach_same_object_export(artifact,bytes(bad),records,triples,{})
        bad=bytearray(out);bad[25428]=1
        with self.assertRaisesRegex(ValueError,'record mapping'):exporter.attach_same_object_export(artifact,bytes(bad),records,triples,{})
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaises(FileNotFoundError):exporter.read_same_object_export(Path(temp))

    @unittest.skipUnless(fixture_available(exporter.ROOT/'target') and all((exporter.ROOT/'target'/folder/name).is_file() for folder,files in exporter.SAME_OBJECT_PINS.items() for name in files), 'same-object fixture unavailable')
    def test_pinned_stream_and_tampering(self):
        root=exporter.ROOT/'target';out,records,provenance=exporter.read_same_object_export(root)
        try:
            artifact=exporter.compose(*exporter.read_inputs(root));exporter.attach_multigroup(artifact,*exporter.read_multigroup(root));exporter.attach_twelve_uvs(artifact,*exporter.read_twelve_uvs(root));exporter.attach_tangent_pairs(artifact,*exporter.read_tangent_pairs(root));eight,data,prov=exporter.read_eight_export(root);exporter.attach_eight_export(artifact,eight,data,prov)
        except FileNotFoundError:self.skipTest('upstream fixture unavailable')
        section=exporter.attach_same_object_export(artifact,out,records,data[exporter.EIGHT_NATIVE,'triples.bin'],provenance)['conditional_same_object_export']
        self.assertEqual((len(section['global_indices']),len(section['index_offset_words_hex'])),(252,85))
        self.assertIn('synthetic owner',section['limits'])
        with tempfile.TemporaryDirectory() as temp:
            dest=Path(temp)
            for folder,files in exporter.SAME_OBJECT_PINS.items():
                d=dest/folder;d.mkdir(exist_ok=True)
                for name in files:(d/name).write_bytes((root/folder/name).read_bytes())
            for folder,name in [(exporter.SAME_OBJECT_FOLDER,'host-stdout.bin'),(exporter.SAME_OBJECT_ENTRY_FOLDER,'result.json'),(exporter.SAME_OBJECT_ENTRY_FOLDER,'host-stdout.bin')]:
                path=dest/folder/name;original=path.read_bytes();bad=bytearray(original);bad[-1]^=1;path.write_bytes(bad)
                with self.subTest(folder=folder,name=name),self.assertRaisesRegex(ValueError,'hash mismatch'):exporter.read_same_object_export(dest)
                path.write_bytes(original)

class TriangleBoundsTests(unittest.TestCase):
    def test_synthetic_ties_and_rejections(self):
        positions=[['00000000']*3 for _ in range(85)]
        positions[0][0]='80000000';positions[1][0]='00000000';positions[2][0]='80000000'
        indices=[0,1,2]*84
        rows=exporter.triangle_extrema_words(positions,indices)
        self.assertEqual((rows[0][0],rows[0][3]),(0x80000000,0x80000000))
        artifact={'conditional_eight_group_export':{'attribute_words_hex':[positions,[],[],[]]},'conditional_same_object_export':{'global_indices':indices}}
        same=bytearray(28476);same[21688:22708]=b''.join(struct.pack('<I',int(v,16)) for row in positions for v in row);same[27468:28476]=struct.pack('<252I',*indices)
        selector=bytes(same)+bytes(28964-28476);out=selector+b''.join(struct.pack('<6I',*r) for r in rows)
        section=exporter.attach_triangle_bounds(artifact,out,selector,bytes(same),{})['conditional_triangle_bounds']
        self.assertEqual((len(section['extrema_words_hex']),section['extrema_words_hex'][0][0]),(84,'80000000'))
        bad=bytearray(out);bad[-1]^=1
        with self.assertRaisesRegex(ValueError,'bounds mapping'):exporter.attach_triangle_bounds(artifact,bytes(bad),selector,bytes(same),{})
        bad=bytearray(selector);bad[0]^=1
        with self.assertRaisesRegex(ValueError,'prefix mismatch'):exporter.attach_triangle_bounds(artifact,out,bytes(bad),bytes(same),{})
        positions[0][0]='7fc00000'
        with self.assertRaisesRegex(ValueError,'nonfinite'):exporter.triangle_extrema_words(positions,indices)
        positions[0][0]='00000000'
        with self.assertRaisesRegex(ValueError,'indices'):exporter.triangle_extrema_words(positions,[85]+indices[1:])
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaises(FileNotFoundError):exporter.read_triangle_bounds(Path(temp))

    @unittest.skipUnless(all((exporter.ROOT/'target'/folder/name).is_file() for folder,files in exporter.TRIANGLE_BOUNDS_PINS.items() for name in files), 'triangle fixture unavailable')
    def test_pinned_stream_and_tampering(self):
        root=exporter.ROOT/'target';out,selector,prov=exporter.read_triangle_bounds(root);same=selector[:28476]
        words=struct.unpack('<255I',same[21688:22708]);positions=[[f'{v:08x}' for v in words[i:i+3]] for i in range(0,255,3)]
        indices=list(struct.unpack('<252I',same[27468:28476]));artifact={'conditional_eight_group_export':{'attribute_words_hex':[positions,[],[],[]]},'conditional_same_object_export':{'global_indices':indices}}
        section=exporter.attach_triangle_bounds(artifact,out,selector,same,prov)['conditional_triangle_bounds']
        self.assertEqual(len(section['extrema_words_hex']),84);self.assertIn('non-remap',section['limits'])
        with tempfile.TemporaryDirectory() as temp:
            dest=Path(temp)
            for folder,files in exporter.TRIANGLE_BOUNDS_PINS.items():
                d=dest/folder;d.mkdir(exist_ok=True)
                for name in files:(d/name).write_bytes((root/folder/name).read_bytes())
            for folder,name in [(exporter.TRIANGLE_BOUNDS_FOLDER,'host-stdout.bin'),(exporter.POSITION_SELECTOR_FOLDER,'result.json')]:
                path=dest/folder/name;original=path.read_bytes();bad=bytearray(original);bad[-1]^=1;path.write_bytes(bad)
                with self.subTest(folder=folder,name=name),self.assertRaisesRegex(ValueError,'hash mismatch'):exporter.read_triangle_bounds(dest)
                path.write_bytes(original)

class AggregateBoundsTests(unittest.TestCase):
    def test_synthetic_ties_and_invalid_first(self):
        zero=[0x80000000,0,0,0,0,0];other=[0,0,0,0x80000000,0,0]
        self.assertEqual(exporter.reduce_bounds_words([zero,other]),zero)
        invalid=[0x40000000,0,0,0x3f800000,0,0];valid=[0xbf800000,0,0,0x3f800000,0,0]
        self.assertEqual(exporter.reduce_bounds_words([invalid,valid]),valid)
        rows=[zero]*84;tri=bytes(28964)+b''.join(struct.pack('<6I',*r) for r in rows)
        controls=struct.pack('<24I',*(zero+other+invalid+valid));out=tri+struct.pack('<18I',*(zero+zero+valid))
        artifact={'conditional_triangle_bounds':{'extrema_words_hex':[[f'{v:08x}' for v in row] for row in rows]}}
        section=exporter.attach_aggregate_bounds(artifact,out,controls,tri,{})['conditional_aggregate_bounds'];self.assertEqual(section['extrema_words_hex'][0],'80000000')
        bad=bytearray(out);bad[-1]^=1
        with self.assertRaisesRegex(ValueError,'bounds mapping'):exporter.attach_aggregate_bounds(artifact,bytes(bad),controls,tri,{})
        bad=bytearray(tri);bad[0]^=1
        with self.assertRaisesRegex(ValueError,'prefix mismatch'):exporter.attach_aggregate_bounds(artifact,out,controls,bytes(bad),{})
        with self.assertRaisesRegex(ValueError,'nonfinite'):exporter.reduce_bounds_words([[0x7fc00000,0,0,0,0,0]])
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaises(FileNotFoundError):exporter.read_aggregate_bounds(Path(temp))

    @unittest.skipUnless(all((exporter.ROOT/'target'/folder/name).is_file() for folder,files in exporter.AGGREGATE_BOUNDS_PINS.items() for name in files) and all((exporter.ROOT/'target'/folder/name).is_file() for folder,files in exporter.TRIANGLE_BOUNDS_PINS.items() for name in files), 'aggregate fixture unavailable')
    def test_pinned_stream_and_tampering(self):
        root=exporter.ROOT/'target';out,controls,prov=exporter.read_aggregate_bounds(root);tri,_,_=exporter.read_triangle_bounds(root)
        rows=[[f'{v:08x}' for v in row] for row in struct.iter_unpack('<6I',tri[28964:])];artifact={'conditional_triangle_bounds':{'extrema_words_hex':rows}}
        section=exporter.attach_aggregate_bounds(artifact,out,controls,tri,prov)['conditional_aggregate_bounds'];self.assertEqual(section['extrema_words_hex'],['be82cccd','bec90000','bee68cce','3e82cccc','3ec6999a','3d8ffffe'])
        with tempfile.TemporaryDirectory() as temp:
            dest=Path(temp)
            for folder,files in exporter.AGGREGATE_BOUNDS_PINS.items():
                d=dest/folder;d.mkdir(exist_ok=True)
                for name in files:(d/name).write_bytes((root/folder/name).read_bytes())
            for name in ('result.json','host-stdout.bin','controls-reduce.bin'):
                path=dest/exporter.AGGREGATE_BOUNDS_FOLDER/name;original=path.read_bytes();bad=bytearray(original);bad[-1]^=1;path.write_bytes(bad)
                with self.subTest(name=name),self.assertRaisesRegex(ValueError,'hash mismatch'):exporter.read_aggregate_bounds(dest)
                path.write_bytes(original)

class SyntheticDeformationTests(unittest.TestCase):
    def test_synthetic_mapping_fp_and_rejections(self):
        positions=[['3f800000','40000000','40400000'] for _ in range(85)]
        offsets=[['0000']*4 for _ in range(85)];weights=[['3f800000','00000000','00000000','00000000'] for _ in range(85)]
        matrix=[1.,0.,0.,0.5,0.,1.,0.,-0.25,0.,0.,1.,0.125];palette=struct.pack('<120f',*(matrix*10))
        rows=exporter.deform_position_words(positions,offsets,weights,palette)
        self.assertEqual(rows[0],[0x3fc00000,0x3fe00000,0x40480000]);self.assertEqual(exporter.f32_round(1.0+2**-24),1.0)
        artifact={'conditional_eight_group_export':{'attribute_words_hex':[positions,[],[],[]]},'conditional_same_object_export':{'index_offset_words_hex':offsets,'weight_words_hex':weights}}
        same=bytearray(28476);same[21688:22708]=b''.join(struct.pack('<I',int(v,16)) for row in positions for v in row);same[25428:26108]=b''.join(struct.pack('<4H',*(int(v,16) for v in row)) for row in offsets);same[26108:27468]=b''.join(struct.pack('<4I',*(int(v,16) for v in row)) for row in weights)
        selector=bytes(same)+bytes(488);out=selector+b''.join(struct.pack('<3I',*row) for row in rows)
        section=exporter.attach_synthetic_deformation(artifact,out,selector,palette,bytes(same),{})['conditional_synthetic_cpu_deformation'];self.assertEqual(section['position_words_hex'][0],['3fc00000','3fe00000','40480000'])
        bad=bytearray(out);bad[-1]^=1
        with self.assertRaisesRegex(ValueError,'output mismatch'):exporter.attach_synthetic_deformation(artifact,bytes(bad),selector,palette,bytes(same),{})
        bad=bytearray(selector);bad[0]^=1
        with self.assertRaisesRegex(ValueError,'prefix mismatch'):exporter.attach_synthetic_deformation(artifact,out,bytes(bad),palette,bytes(same),{})
        offsets[0][3]='001c'
        with self.assertRaisesRegex(ValueError,'offsets'):exporter.deform_position_words(positions,offsets,weights,palette)
        offsets[0][3]='0000';weights[0][0]='7f800000'
        with self.assertRaisesRegex(ValueError,'nonfinite'):exporter.deform_position_words(positions,offsets,weights,palette)
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaises(FileNotFoundError):exporter.read_synthetic_deformation(Path(temp))

    @unittest.skipUnless(all((exporter.ROOT/'target'/folder/name).is_file() for folder,files in exporter.DEFORM_PINS.items() for name in files), 'deformation fixture unavailable')
    def test_pinned_stream_and_tampering(self):
        root=exporter.ROOT/'target';out,selector,palette,prov=exporter.read_synthetic_deformation(root);same=selector[:28476]
        words=struct.unpack('<255I',same[21688:22708]);positions=[[f'{v:08x}' for v in words[i:i+3]] for i in range(0,255,3)]
        offsets=[[f'{v:04x}' for v in row] for row in struct.iter_unpack('<4H',same[25428:26108])];weights=[[f'{v:08x}' for v in row] for row in struct.iter_unpack('<4I',same[26108:27468])]
        artifact={'conditional_eight_group_export':{'attribute_words_hex':[positions,[],[],[]]},'conditional_same_object_export':{'index_offset_words_hex':offsets,'weight_words_hex':weights}}
        section=exporter.attach_synthetic_deformation(artifact,out,selector,palette,same,prov)['conditional_synthetic_cpu_deformation'];self.assertEqual(len(section['position_words_hex']),85);self.assertIn('not deformed',section['limits'])
        with tempfile.TemporaryDirectory() as temp:
            dest=Path(temp)
            for folder,files in exporter.DEFORM_PINS.items():
                d=dest/folder;d.mkdir(exist_ok=True)
                for name in files:(d/name).write_bytes((root/folder/name).read_bytes())
            for folder,name in [(exporter.DEFORM_FOLDER,'synthetic-palette.bin'),(exporter.DEFORM_FOLDER,'host-stdout.bin'),(exporter.POSITION_SELECTOR_FOLDER,'result.json')]:
                path=dest/folder/name;original=path.read_bytes();bad=bytearray(original);bad[-1]^=1;path.write_bytes(bad)
                with self.subTest(folder=folder,name=name),self.assertRaisesRegex(ValueError,'hash mismatch'):exporter.read_synthetic_deformation(dest)
                path.write_bytes(original)

class RowCategoryTests(unittest.TestCase):
    def test_synthetic_branch_priority_and_rejections(self):
        rows=bytearray(1400)
        cases=[(5,8,1),(3,9,2),(3,0,3),(4,8,3),(2,0,4),(2,8,5),(1,0,4),(5,0,1),(3,1,2),(4,0,3)]
        for i,(kind,flags,_) in enumerate(cases):
            struct.pack_into('<H',rows,i*140+8,flags);rows[i*140+11]=kind
        prefix=bytes(28476);values=[x[2] for x in cases];out=prefix+struct.pack('<10I',*values);artifact={'conditional_same_object_export':{}}
        prefix_out=out+struct.pack('<56I',*(v for n in (10,8,0,10) for v in ([10,n,n,n]+values)))
        section=exporter.attach_row_categories(artifact,out,bytes(rows),prefix_out,prefix,{})['conditional_row_categories']
        self.assertEqual(section['categories_in_frozen_row_order'],values);self.assertEqual(section['conditional_prefix_length'],10);self.assertIn('not palette IDs',section['limits'])
        bad=bytearray(out);bad[-1]^=1
        with self.assertRaisesRegex(ValueError,'mapping mismatch'):exporter.attach_row_categories(artifact,bytes(bad),bytes(rows),bytes(bad)+prefix_out[28516:],prefix,{})
        bad=bytearray(out);bad[0]^=1
        with self.assertRaisesRegex(ValueError,'prefix mismatch'):exporter.attach_row_categories(artifact,bytes(bad),bytes(rows),prefix_out,prefix,{})
        bad=bytearray(prefix_out);bad[-1]^=1
        with self.assertRaisesRegex(ValueError,'prefix mapping'):exporter.attach_row_categories(artifact,out,bytes(rows),bytes(bad),prefix,{})
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaises(FileNotFoundError):exporter.read_row_categories(Path(temp))

    @unittest.skipUnless(all((exporter.ROOT/'target'/folder/name).is_file() for folder,files in exporter.ROW_CATEGORIES_PINS.items() for name in files), 'row category fixture unavailable')
    def test_pinned_categories_and_tampering(self):
        root=exporter.ROOT/'target';out,rows,prefix_out,prov=exporter.read_row_categories(root)
        try:same,_,_=exporter.read_same_object_export(root)
        except FileNotFoundError:self.skipTest('same-object fixture unavailable')
        section=exporter.attach_row_categories({'conditional_same_object_export':{}},out,rows,prefix_out,same,prov)['conditional_row_categories']
        self.assertEqual(section['categories_in_frozen_row_order'],[1,1,1,1,1,1,4,4,4,4]);self.assertEqual(section['conditional_prefix_length'],10)
        with tempfile.TemporaryDirectory() as temp:
            dest=Path(temp)
            for folder,files in exporter.ROW_CATEGORIES_PINS.items():
                d=dest/folder;d.mkdir(exist_ok=True)
                for name in files:(d/name).write_bytes((root/folder/name).read_bytes())
            for folder,name in [(exporter.ROW_CATEGORIES_FOLDER,n) for n in ('result.json','host-stdout.bin','rows.bin')]+[(exporter.ROW_PREFIX_FOLDER,n) for n in ('result.json','host-stdout.bin')]:
                path=dest/folder/name;original=path.read_bytes();bad=bytearray(original);bad[-1]^=1;path.write_bytes(bad)
                with self.subTest(folder=folder,name=name),self.assertRaisesRegex(ValueError,'hash mismatch'):exporter.read_row_categories(dest)
                path.write_bytes(original)

if __name__ == '__main__':
    unittest.main()

