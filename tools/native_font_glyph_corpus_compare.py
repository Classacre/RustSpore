"""Audit retained glyph replies, preserving fresh/sequence state differences.

No native process runs here. Heap-policy equality is tested only for identical
ordered glyph/context inputs. Fresh versus sequence is measured separately.
"""
import argparse
import hashlib
import json
from pathlib import Path

from native_font_glyph_reply import parse_reply, parse_request

ROOT = Path(__file__).resolve().parent.parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cohort', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    cohort = args.cohort.resolve()
    cohort.relative_to((ROOT / 'target').resolve())
    args.out.resolve().relative_to((ROOT / 'target').resolve())
    assert not args.out.exists()
    manifest = json.loads((cohort / 'manifest.json').read_text())
    run = json.loads((cohort / 'run-v1.json').read_text())
    assert run['completed_cases'] == manifest['cases'] == 80
    assert run['glyph_observations'] == 4624 and run['pinned_inputs_unchanged']
    groups, observations, native_runs = {}, {}, []
    source_pins = {}
    contexts = checks = observation_count = allocation_retries = 0
    for row in manifest['requests']:
        request_path = ROOT / row['request']
        assert sha(request_path) == row['request_sha256']
        request = parse_request(request_path.read_bytes())
        attempts = [r for r in run['attempts'] if r['case'] == row['name']]
        succeeded = [r for r in attempts if r['exit'] == 0]
        assert len(succeeded) == 1 and succeeded[-1] == attempts[-1]
        for attempt in attempts[:-1]:
            directory = Path(attempt['report']).parent
            detail = json.loads((directory / 'report.json').read_text())
            mapping = detail['mapping_failure']
            assert mapping['stage'] in ['reservation_failure', 'commit_failure']
            assert mapping['actual_return'] == '00000000' and detail['completed_stage'] == 1
            assert not detail['native_font_entry_started'] and detail['frozen_inputs_unchanged']
            assert not (directory / 'reply.bin').read_bytes() and not (directory / 'stderr.bin').read_bytes()
            allocation_retries += 1
        directory = Path(succeeded[0]['report']).parent
        detail = json.loads((directory / 'report.json').read_text())
        assert detail['exit'] == 0 and detail['glyph_success'] and detail['completed_stage'] == 2
        assert detail['native_font_entry_started'] and detail['frozen_inputs_unchanged']
        assert not detail.get('unexpected_exceptions') and not (directory / 'stderr.bin').read_bytes()
        assert (directory / 'request.bin').read_bytes() == request_path.read_bytes()
        for path, digest in json.loads((directory / 'pins-v1.json').read_text()).items():
            assert path not in source_pins or source_pins[path] == digest
            source_pins[path] = digest
        exports = []
        parsed = parse_reply((directory / 'reply.bin').read_bytes(), request,
            active_sink=lambda item, pixels: exports.append((item, pixels)))
        assert parsed['success'] and parsed['count'] == len(request['glyphs'])
        contexts += len(parsed['contexts'])
        checks += parsed['final']['guard_checks']
        for glyph, pixels in exports:
            active = directory / 'active' / f"glyph-{glyph['ordinal']:02d}-{glyph['glyph_index']:03d}.alpha"
            assert active.read_bytes() == pixels
            key = (request['mode'], tuple(request['glyphs']), glyph['ordinal'])
            semantic = glyph['semantic_sha256']
            if key in groups:
                assert groups[key] == semantic, 'Identical context/glyph inputs depend on heap policy'
            else:
                groups[key] = semantic
                observations[request['mode'], glyph['glyph_index']] = glyph
            observation_count += 1
        native_runs.append(dict(case=row['name'], request_sha256=sha(request_path),
            reply_sha256=parsed['reply_sha256'], native_report_sha256=sha(directory / 'report.json')))
    assert observation_count == 4624 and len(groups) == 1156 and len(observations) == 1156
    assert contexts == 2352
    for path, digest in source_pins.items():
        assert sha(Path(path)) == digest, path
    differences = []
    for glyph in range(578):
        fresh, sequence = observations[0, glyph], observations[1, glyph]
        if fresh['semantic_sha256'] != sequence['semantic_sha256']:
            fields = ['width', 'height', 'offset_e4_raw', 'offset_e0_raw', 'advance_d0_raw',
                'bitmap_present', 'active_sha256', 'semantic_sha256']
            differences.append(dict(glyph=glyph,
                fresh={k: fresh[k] for k in fields}, sequence={k: sequence[k] for k in fields}))
    lines = ''.join(f'{mode}\t{glyph:03d}\t{observations[mode,glyph]["semantic_sha256"]}\n'
        for mode in [0, 1] for glyph in range(578))
    digest_path = args.out.with_suffix('.tsv')
    with digest_path.open('xb') as output:
        output.write(lines.encode())
    result = dict(schema='original-font-native-context-preserving-corpus-audit-v1',
        all_native_observations_complete=True, cases=80, glyph_observations=4624,
        unique_glyphs=578, font_contexts=contexts, guard_checks=checks,
        exact_heap_policy_comparison_groups=1156, heap_policies_per_group=4,
        pre_source_mapping_retries=allocation_retries, all_current_pins_exact=True,
        unique_pinned_files=len(source_pins), semantic_records_sha256=sha(digest_path),
        fresh_vs_sequence_differences=differences, native_runs=native_runs,
        original_executor_assertion_error=run.get('error'),
        original_failed_assertion_retained=True, original_module_loaded=False,
        Rust_glyph_pixels_compared=False,
        scope='One original font,15ppem,mode3/flags8B,CW037F/MXCSR1F80,reviewed frozen v2 host under debug startup. Sequence means ascending chunks of at most64 glyphs, not one578-glyph context. Native heap-policy equality is exact within each ordered input. Fresh/sequence differences remain source outcomes; their cause and retail reset policy remain open.')
    with args.out.open('x') as output:
        json.dump(result, output, indent=2)
        output.write('\n')
    print(json.dumps({k: v for k, v in result.items() if k not in ['native_runs', 'fresh_vs_sequence_differences']}))
    print(json.dumps(dict(differing_glyphs=[r['glyph'] for r in differences])))


if __name__ == '__main__':
    main()
