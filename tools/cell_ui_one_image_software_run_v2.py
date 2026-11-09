"""Run the reviewed Cell single-image fixture with source and output hash binding.

This driver is for later use after the Rust example is built and reviewed. It
invokes only the explicit fixture executable supplied by the caller. Original
packages are opened read-only; PNG and JSON are created under target. Matching
before/after hashes do not rule out a transient modification during execution.
"""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from dbpf_extract import payload
from gmdl_skin_probe import indexes

ROOT = Path(__file__).resolve().parent.parent
TARGET = (ROOT / 'target').resolve()


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def bound_target(path):
    value = path.resolve()
    value.relative_to(TARGET)
    if value == TARGET:
        raise ValueError('target itself is not an output file')
    return value


def resource_hash(path, typ, instance):
    matches = [e for e in indexes(path) if e['type'] == typ and e['instance'] == instance]
    if len(matches) != 1:
        raise ValueError('expected a unique explicitly selected resource')
    return hashlib.sha256(payload(path, matches[0])).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--executable', required=True, type=Path)
    p.add_argument('--layout', required=True, type=Path)
    p.add_argument('--atlas', required=True, type=Path)
    p.add_argument('--width', required=True, type=int)
    p.add_argument('--height', required=True, type=int)
    p.add_argument('--parent-span-x', required=True)
    p.add_argument('--parent-span-y', required=True)
    p.add_argument('--png', required=True, type=Path)
    p.add_argument('--report', required=True, type=Path)
    a = p.parse_args()
    exe, png, report = map(bound_target, (a.executable, a.png, a.report))
    layout, atlas = a.layout.resolve(strict=True), a.atlas.resolve(strict=True)
    if png == report or png.exists() or report.exists():
        raise FileExistsError('output paths must be distinct and new')
    before = {str(x): digest(x) for x in (exe, layout, atlas)}
    layout_payload = resource_hash(layout, 0x250fe9a2, 0x4046420026b068d2)
    atlas_payload = resource_hash(atlas, 0x2f7d0004, 0x31a44893b8b50133)
    cmd = [str(exe), str(layout), str(atlas), str(a.width), str(a.height),
           a.parent_span_x, a.parent_span_y, str(png)]
    result = subprocess.run(cmd, check=True, capture_output=True, text=True)
    after = {str(x): digest(x) for x in (exe, layout, atlas)}
    if before != after:
        raise ValueError('an input changed during rendering; output is unbound')
    manifest = {
        'schema': 'cell-ui-one-image-software-v2-run-v1',
        'scope': 'Approximate single-element software image; explicit parent spans and canvas, not a retail frame.',
        'arguments': cmd, 'source_and_executable_sha256_before_after': before,
        'layout_resource_sha256': layout_payload, 'atlas_resource_sha256': atlas_payload,
        'png_sha256': digest(png), 'stdout': result.stdout,
        'limitations': ['Matching before and after hashes do not exclude transient input mutation.',
                        'No active package, native sampling, composition, runtime frame or complete HUD established.'],
    }
    with report.open('x', encoding='utf-8', newline='\n') as out:
        json.dump(manifest, out, indent=2)
        out.write('\n')
    print(f'PNG and run report created: {png}, {report}')

if __name__ == '__main__':
    main()
