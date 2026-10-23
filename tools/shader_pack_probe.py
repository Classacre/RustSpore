"""Bounded investigation of the installed header-7 shader packs.

Retains original shader bytecode only under the requested ignored output path.
Unknown framing fails rather than manufacturing a shader recipe.
"""
import argparse
import json
import hashlib
from pathlib import Path
from shader_fragment_probe import Reader


def inspect(data):
    reader = Reader(data)
    version, count = reader.word(), reader.word()
    assert version == 7 and count <= 4096
    records, error = [], None
    for index in range(count):
        start = reader.offset
        try:
            metadata = reader.take(28)
            name = reader.string() if metadata[-1] & 0x10 else ''
            variants = []
            while True:
                render_type = reader.take(1)[0]
                if render_type == 255:
                    break
                assert len(variants) < 256
                vertex_size = reader.word()
                assert vertex_size <= len(data) - reader.offset
                vertex = reader.take(vertex_size)
                pixel_size = reader.word()
                assert pixel_size <= len(data) - reader.offset
                pixel = reader.take(pixel_size)
                uniforms = []
                for stage in range(2):
                    uniform_count = reader.word()
                    assert uniform_count <= 2048
                    uniforms.append([reader.take(12).hex() for _ in range(uniform_count)])
                variants.append(dict(render_type=render_type,vertex_bytes=vertex_size,pixel_bytes=pixel_size,
                                     vertex_uniforms=uniforms[0],pixel_uniforms=uniforms[1],
                                     vertex_prefix=vertex[:12].hex(),pixel_prefix=pixel[:12].hex()))
            records.append(dict(index=index,offset=start,end=reader.offset,metadata=metadata.hex(),name=name,
                                variants=variants))
        except Exception as failure:
            error = dict(index=index,start=start,offset=reader.offset,error=str(failure),
                         context=data[max(0,reader.offset-16):reader.offset+64].hex(' '))
            break
    standard_end = reader.offset
    builders, builder_count = [], None
    if error is None:
        try:
            builder_count = reader.word()
            assert builder_count <= 4096
            for index in range(builder_count):
                start = reader.offset
                metadata = reader.take(28)
                name = reader.string() if metadata[-1] & 0x10 else ''
                variants = []
                while True:
                    render_type = reader.take(1)[0]
                    if render_type == 255:
                        break
                    assert len(variants) < 256
                    selectors = []
                    for stage in range(2):
                        selector_count = reader.word()
                        assert selector_count <= 4096
                        selectors.append([reader.take(25).hex() for _ in range(selector_count)])
                    variants.append(dict(render_type=render_type,vertex_selectors=selectors[0],pixel_selectors=selectors[1]))
                builders.append(dict(index=index,offset=start,end=reader.offset,metadata=metadata.hex(),name=name,variants=variants))
        except Exception as failure:
            error = dict(table='builders',index=len(builders),offset=reader.offset,error=str(failure),
                         context=data[max(0,reader.offset-16):reader.offset+96].hex(' '))
    builder_end = reader.offset
    compiled = []
    if error is None:
        try:
            for stage in range(2):
                stage_word, compiled_count = reader.word(), reader.word()
                assert compiled_count <= 100_000
                programs = []
                compiled.append(dict(stage_word=stage_word,count=compiled_count,programs=programs))
                for index in range(compiled_count):
                    start = reader.offset
                    signature = reader.take(32)
                    size = reader.word()
                    bytecode = reader.take(size)
                    uniform_count = reader.word()
                    assert uniform_count <= 2048
                    uniforms = [reader.take(12).hex() for _ in range(uniform_count)]
                    registers = [reader.word() for _ in range(uniform_count)]
                    flags = reader.word()
                    programs.append(dict(index=index,offset=start,end=reader.offset,signature=signature.hex(),
                                         bytecode_bytes=size,bytecode_prefix=bytecode[:12].hex(),uniforms=uniforms,
                                         registers=registers,flags=flags))
        except Exception as failure:
            error = dict(table='compiled',stage=len(compiled)-1,index=len(compiled[-1]['programs']) if compiled else 0,
                         offset=reader.offset,error=str(failure),context=data[max(0,reader.offset-16):reader.offset+96].hex(' '))
    pack_name = None
    if error is None:
        try:
            pack_name = reader.string()
            assert reader.offset == len(data), 'trailing bytes'
        except Exception as failure:
            error = dict(table='name',offset=reader.offset,error=str(failure))
    return dict(version=version,count=count,records=records,standard_end=standard_end,
                builder_count=builder_count,builders=builders,builder_end=builder_end,
                compiled=compiled,pack_name=pack_name,error=error,consumed=reader.offset,bytes=len(data),
                exact=reader.offset==len(data) and error is None)


def validate_compiled(pack):
    """Check the native cache key and uniform mask independently of Rust."""
    checked = 0
    for stage in pack['compiled']:
        recipes = set()
        for program in stage['programs']:
            signature = bytes.fromhex(program['signature'])
            assert 0 in signature, 'unterminated compiled fragment recipe'
            recipe = signature[:signature.index(0)]
            assert recipe not in recipes, 'ambiguous compiled fragment recipe'
            recipes.add(recipe)
            flags = 0
            for uniform in program['uniforms']:
                flags |= int.from_bytes(bytes.fromhex(uniform)[8:12], 'big')
            assert flags == program['flags'], 'compiled uniform mask mismatch'
            checked += 1
    return dict(programs=checked, terminated_unique_recipes=True,
                uniform_mask_agreement=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('file',type=Path)
    ap.add_argument('--out',type=Path,required=True)
    args=ap.parse_args()
    if args.file.is_dir():
        from shader_resource_probe import digest
        results, digests = [], []
        for file in sorted(args.file.glob('*-0469A3F7-40212004*.bin')):
            data = file.read_bytes()
            result = inspect(data)
            assert result['exact'], (str(file), result['error'])
            validation = validate_compiled(result)
            package, _, instance = file.stem.rsplit('-', 2)
            vertex, pixel = [len(stage['programs']) for stage in result['compiled']]
            digests.append(f"{package}.package\t{instance}\t{len(result['records'])}\t{len(result['builders'])}\t{vertex}\t{pixel}\t{digest(data):016X}")
            results.append(dict(file=str(file),standard=len(result['records']),builders=len(result['builders']),
                                vertex=vertex,pixel=pixel,exact=True,compiled_validation=validation))
            print(file.name,len(result['records']),len(result['builders']),vertex,pixel,'exact')
        assert results, 'no extracted header-7 source files'
        text = '\n'.join(sorted(digests)) + '\n'
        args.out.write_text(json.dumps(dict(files=results,digest_sha256=hashlib.sha256(text.encode()).hexdigest()),indent=2),encoding='utf-8')
        args.out.with_suffix('.tsv').write_text(text,encoding='utf-8')
        return
    result=inspect(args.file.read_bytes())
    if result['exact']:
        result['compiled_validation'] = validate_compiled(result)
    args.out.write_text(json.dumps(result,indent=2),encoding='utf-8')
    print('records',len(result['records']),'/',result['count'],'consumed',result['consumed'],'/',result['bytes'],'error',result['error'])
    for record in result['records'][:6]:
        print(record['index'],record['offset'],record['name'],len(record['variants']))
    print('builders',len(result['builders']),'/',result['builder_count'])
    for record in result['builders'][:6]:
        print(record['index'],record['offset'],record['name'],len(record['variants']))
    for stage in result['compiled']:
        print('compiled stage',stage['stage_word'],len(stage['programs']),'/',stage['count'])


if __name__=='__main__':
    main()
