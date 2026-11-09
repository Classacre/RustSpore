"""Compare a controlled source-segment shader readback with software sampling.

Assumes straight RGBA software input and SRCALPHA/INVSRCALPHA over transparent
with separate-alpha disabled. This is not evidence of retail device state.
"""
import argparse
import hashlib
import json
from pathlib import Path
from PIL import Image


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('software', type=Path)
    p.add_argument('shader', type=Path)
    p.add_argument('scissor', type=Path)
    p.add_argument('--out', type=Path, required=True)
    args = p.parse_args()
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent / 'target').resolve())
    if args.out.exists(): raise FileExistsError(args.out)
    paths = [args.software, args.shader, args.scissor]
    raw = [x.read_bytes() for x in paths]
    images = [Image.open(x).convert('RGBA') for x in paths]
    if any(i.size != (300, 100) for i in images): raise ValueError('unexpected fixture size')
    if raw[1] != raw[2]: raise ValueError('clip branches differ')
    a, b = [list(i.getdata()) for i in images[:2]]
    coverage_a = {i for i,pixel in enumerate(a) if pixel[3]}
    coverage_b = {i for i,pixel in enumerate(b) if pixel[3]}
    errors = []
    for src, actual in zip(a,b):
        expected = [(channel * src[3] + 127) // 255 for channel in src]
        errors.append(max(abs(x-y) for x,y in zip(expected,actual)))
    if not coverage_a or not coverage_b or max(errors) > 1:
        raise ValueError('coverage empty or channel error exceeds controlled tolerance 1')
    report = {'scope': __doc__, 'fixture_size': [300,100], 'pixels': len(a),
              'software_sha256': hashlib.sha256(raw[0]).hexdigest(),
              'shader_sha256': hashlib.sha256(raw[1]).hexdigest(),
              'scissor_sha256': hashlib.sha256(raw[2]).hexdigest(),
              'software_nonzero_alpha': len(coverage_a), 'gpu_nonzero_alpha': len(coverage_b),
              'shared_coverage': len(coverage_a & coverage_b),
              'software_only': len(coverage_a - coverage_b), 'gpu_only': len(coverage_b - coverage_a),
              'maximum_error_after_alpha_blend': max(errors),
              'pixels_over_one': sum(e > 1 for e in errors), 'clip_branch_png_bytes_equal': raw[1] == raw[2]}
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open('x',encoding='utf-8') as out:
        json.dump(report,out,indent=2); out.write('\n')
    print(json.dumps(report,indent=2))


if __name__ == '__main__': main()
