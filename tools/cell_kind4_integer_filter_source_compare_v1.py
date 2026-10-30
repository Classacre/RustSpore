"""Compare bounded integer set projection with a direct source-record predicate.

Synthetic ordinals and unadjusted list words are not native registration or selection.
Only aggregate hashes and counts are written under target; source records stay in memory.
"""
import argparse
import collections
import hashlib
import json
import struct
import subprocess
import threading
from pathlib import Path

from cell_hierarchy_76f930_compare_v1 import TARGET, digest, expected_hash, inside_target
from cell_kind4_list_source_verify_v1 import native_rows
from refpack import _read_header, decompress

PIN = '3de1933446c61546f8afcf44e22c1010a5405baf353d0d5c9259ea6071f4e7c6'
MAX_CASE = 1024 * 1024
MAX_BATCH = 64 * 1024 * 1024
INTEGER = 0x02E1A75D
TYPE_FIELD, TYPE_VALUE = 0x02DD90AF, 0x3D97A8E4
R0, R1 = 0xDDDD0000, 0xDDDD0001
# Source list words before unknown global adjustment, used as illustrations only.
PAIRS = [(-1,4,2,5), (-1,1,3,-1), (-1,2,-1,4), (-1,4,-1,2),
         (-1,4,1,4), (-1,4,2,4), (-1,4,3,8), (2,-1,1,5)]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def adjusted(lower, upper):
    if lower < 0 and upper < 0:
        return None
    return max(0, lower), (10 if upper == -1 else min(10, upper)) + 1


def run_bounded(executable, batch, timeout):
    # Read stdout/stderr concurrently with stdin, stopping at fixed limits.
    process = subprocess.Popen([str(executable)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, cwd=str(TARGET), creationflags=subprocess.CREATE_NO_WINDOW)
    outputs = [bytearray(), bytearray()]
    errors = []
    def drain(stream, output, limit):
        try:
            while True:
                chunk = stream.read(65536)
                if not chunk:
                    break
                if len(output) + len(chunk) > limit:
                    errors.append('subprocess output limit')
                    process.kill()
                    break
                output.extend(chunk)
        except (OSError, ValueError) as exc:
            errors.append(f'pipe read: {exc}')
        finally:
            stream.close()
    def feed():
        try:
            process.stdin.write(batch)
            process.stdin.close()
        except (OSError, ValueError) as exc:
            errors.append(f'pipe write: {exc}')
    threads = [threading.Thread(target=drain, args=(process.stdout, outputs[0], MAX_BATCH)),
               threading.Thread(target=drain, args=(process.stderr, outputs[1], 65536)),
               threading.Thread(target=feed)]
    for thread in threads: thread.start()
    try:
        process.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()
        errors.append('subprocess timeout')
    finally:
        for thread in threads: thread.join()
    if errors or process.returncode or outputs[1]:
        raise ValueError('subprocess failed: ' + ', '.join(errors))
    return bytes(outputs[0])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--census', type=inside_target, required=True)
    parser.add_argument('--example', type=inside_target, required=True)
    parser.add_argument('--example-sha256', type=expected_hash, required=True)
    parser.add_argument('--out', type=inside_target, required=True)
    parser.add_argument('--timeout', type=float, default=120)
    a = parser.parse_args()
    if not 0 < a.timeout <= 300:
        parser.error('invalid timeout')
    root = a.install.resolve(strict=True)
    if not a.census.is_file() or a.census.stat().st_size > 128 * 1024 * 1024:
        raise ValueError('invalid census')
    data = a.census.read_bytes()
    if sha(data) != PIN or not a.example.is_file() or a.example.suffix.lower() != '.exe' or digest(a.example) != a.example_sha256:
        raise ValueError('input pin mismatch')
    selected = []
    for line in data.decode('utf-8').splitlines():
        r = line.split('\t')
        if r[0] == 'R':
            if len(r) != 12:
                raise ValueError('bad census row')
            if r[4] == '02D5C9AF':
                selected.append(r)
    selected.sort(key=lambda r: (r[1], int(r[3],16), int(r[4],16), int(r[5],16)))
    identities = [(r[1], r[3], r[4], r[5]) for r in selected]
    if len(selected) != 2590 or len(set(identities)) != len(selected):
        raise ValueError('unexpected or duplicate census identities')
    packages = {r[1]: (root / r[1]).resolve(strict=True) for r in selected}
    if len(packages) != 4 or any(not p.is_relative_to(root) or not p.is_file() for p in packages.values()):
        raise ValueError('invalid package paths')
    before = {name: digest(path) for name, path in packages.items()}
    a.out.mkdir(parents=True, exist_ok=False)
    report = {'matched': False, 'scope': 'synthetic ordinal set projection; not runtime registration, native ordering, selection or playback',
              'census_sha256': PIN, 'example_sha256_before': a.example_sha256, 'package_sha256_before': before}
    try:
        indexes = {}
        for name, path in packages.items():
            rows = native_rows(path)
            indexes[name] = {r[:3]: r for r in rows}
            if len(indexes[name]) != len(rows):
                raise ValueError('duplicate live native key')
        records = []
        for r in selected:
            path = packages[r[1]]
            offset, stored, decoded = map(int, r[6:9])
            key = tuple(int(r[i],16) for i in (3,4,5))
            if min(offset, stored, decoded) < 0 or max(stored, decoded) > MAX_CASE or offset + stored > path.stat().st_size:
                raise ValueError('invalid payload bounds')
            live = indexes[r[1]].get(key)
            if live is None or live[3:7] != (offset, stored, decoded, 0xffff) or int(r[9],16) != 0xffff:
                raise ValueError('live index mismatch')
            with path.open('rb') as f:
                f.seek(offset)
                encoded = f.read(stored)
            if len(encoded) != stored or _read_header(encoded, 0)[2] != decoded:
                raise ValueError('invalid compressed payload')
            raw, _, _ = decompress(encoded, out_size=decoded, header=True)
            if len(raw) != decoded or len(raw) < 12 or struct.unpack_from('>I',raw)[0] != 4:
                raise ValueError('invalid version or decoded size')
            pos, sections = 4, []
            for _ in range(2):
                if pos + 4 > len(raw): raise ValueError('truncated count')
                count = struct.unpack_from('>I',raw,pos)[0]; pos += 4
                if count > 4096 or pos + count * 12 > len(raw): raise ValueError('invalid extent')
                section = [struct.unpack_from('>III',raw,pos + 12*i) for i in range(count)]
                pos += count * 12
                sections.append(section)
            if pos != len(raw): raise ValueError('trailing decoded data')
            records.append([(field, value if value < 0x80000000 else value - 0x100000000)
                            for field, marker, value in sections[0] if marker == INTEGER])
        # Deliberately add duplicate candidate and posting; the oracle checks set behavior.
        candidates = list(range(len(records))) + [0]
        postings = [(field, value, ordinal) for ordinal, rows in enumerate(records) for field,value in rows]
        postings.append(postings[0])
        if len(postings) > 65536 or len(postings) > 1_000_000:
            raise ValueError('projection posting/entry cap exceeded')
        cases = [('type', [(0,TYPE_FIELD,TYPE_VALUE,TYPE_VALUE+1)])]
        for i,(lo1,hi1,lo0,hi0) in enumerate(PAIRS):
            cases.append((f'list_words_{i}', [(0,TYPE_FIELD,TYPE_VALUE,TYPE_VALUE+1), (1,R1,lo1,hi1), (1,R0,lo0,hi0)]))
        cases += [('empty_universe', []), ('omitted',[(1,R1,-1,-2)]), ('sentinel',[(1,R1,0,-1)]),
                  ('negative_upper',[(1,R1,1,-2)]), ('reversed',[(0,TYPE_FIELD,5,-5)]),
                  ('empty',[(0,TYPE_FIELD,0,0)]), ('missing',[(0,0xFFFFFFFF,-2147483648,2147483647)]),
                  ('signed_full',[(0,TYPE_FIELD,-2147483648,2147483647)])]
        # Exercise actual source field/value boundaries, independently of the posting representation.
        observed = sorted({pair for rows in records for pair in rows})
        for field,value in observed[:8]:
            if value < 2147483647:
                cases.append((f'observed_{field:08X}_{value}',[(0,field,value,value+1)]))
        words = ['K4F1',str(len(candidates)),*(str(x) for x in candidates),str(len(postings))]
        for field,value,ordinal in postings:
            words.extend((str(field),str(value),'1',str(ordinal)))
        words.append(str(len(cases)))
        expected = ['K4O1']
        summaries = []
        for name, specs in cases:
            empty_universe = name == 'empty_universe'
            words.extend((str(int(empty_universe)), str(len(specs))))
            active = []
            for mode,field,lower,upper in specs:
                words.extend(map(str,(mode,field,lower,upper)))
                bounds = adjusted(lower,upper) if mode else (lower,upper)
                if bounds is not None: active.append((field,*bounds))
            # Direct existential scan over each original candidate's source triples.
            matches = [i for i,rows in enumerate(records) if not empty_universe
                       if all(any(f == field and lo <= v < hi for f,v in rows)
                              for field,lo,hi in active)]
            line = ' '.join([str(len(matches)),*(str(i) for i in matches)])
            expected.append(line)
            summaries.append({'case': name, 'count': len(matches), 'ordinals_sha256': sha((line+'\n').encode())})
        batch = (' '.join(words)+'\n').encode()
        output = ('\n'.join(expected)+'\n').encode()
        if max(len(batch),len(output)) > MAX_BATCH: raise ValueError('batch limit')
        if digest(a.example) != a.example_sha256: raise ValueError('example changed before run')
        rust_output = run_bounded(a.example, batch, a.timeout)
        if rust_output != output:
            raise ValueError('Rust/Python output mismatch')
        manifest = ''.join(f'{i}\t{p}\t{low}:{typ}:{scope}\n' for i,(p,low,typ,scope) in enumerate(identities)).encode()
        report.update(matched=True, records=len(records), integer_triples=sum(map(len,records)), cases=summaries,
                      ordinal_manifest_sha256=sha(manifest), input_sha256=sha(batch), python_output_sha256=sha(output),
                      rust_output_sha256=sha(rust_output))
    except Exception as exc:
        report['error'] = f'{type(exc).__name__}: {exc}'
    finally:
        try:
            report['package_sha256_after'] = {name:digest(path) for name,path in packages.items()}
            report['census_sha256_after'] = digest(a.census)
            report['example_sha256_after'] = digest(a.example)
            if report['package_sha256_after'] != before or report['census_sha256_after'] != PIN or report['example_sha256_after'] != a.example_sha256:
                report['matched'] = False
                report['error'] = 'input changed during run'
        except OSError as exc:
            report['matched'] = False
            report['error'] = f'rehash failed: {exc}'
        (a.out / 'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    if not report['matched']:
        raise ValueError('comparison failed; see report')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
