"""Run prepared font requests through the separately reviewed hidden fixture.

Only proved pre-source mapping failures may be retried. Every attempt remains
under target. Native failures stop the cohort; no glyph is silently excluded.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cohort', type=Path, required=True)
    parser.add_argument('--runner-review', type=Path, required=True)
    parser.add_argument('--execute-reviewed', action='store_true')
    args = parser.parse_args()
    cohort = args.cohort.resolve()
    cohort.relative_to((ROOT / 'target').resolve())
    manifest_path = cohort / 'manifest.json'
    manifest = json.loads(manifest_path.read_text())
    rows = manifest['requests']
    assert len(rows) == manifest['cases'] == 80
    assert sum(len(row['policy']['glyphs']) for row in rows) == 4624
    runner = ROOT / 'tools/native_font_glyph_debug_run.py'
    review_path = args.runner_review.resolve()
    review_path.relative_to((ROOT / 'target').resolve())
    assert json.loads(review_path.read_text())['runner_sha256'] == sha(runner)
    pins = {str(path): sha(path) for path in [manifest_path, runner, review_path, Path(__file__)]}
    for row in rows:
        request = (ROOT / row['request']).resolve()
        request.relative_to((ROOT / 'target').resolve())
        assert sha(request) == row['request_sha256']
        pins[str(request)] = row['request_sha256']
    if not args.execute_reviewed:
        print(json.dumps(dict(prepared=True, executed=False, cases=80, glyph_observations=4624)))
        return
    results = cohort / 'results'
    results.mkdir(exist_ok=False)
    report_path = cohort / 'run-v1.json'
    assert not report_path.exists()
    report = dict(schema='original-font-glyph-complete-cohort-run-v1',
        requested_cases=80, requested_glyph_observations=4624,
        completed_cases=0, glyph_observations=0, unique_glyphs=0,
        max_mapping_attempts_per_case=8, attempts=[], semantic_differences=[],
        whole_cohort_completed=False, original_module_loaded=False,
        desktop_input_used=False, fixture='Frozen font v2, explicit15ppem, debug startup')
    baseline, context_baseline = {}, {}
    report['heap_policy_differences'] = []
    save(cohort / 'execution-pins-v1.json', pins)
    try:
        for row in rows:
            success = None
            for attempt in range(1, 9):
                out = results / row['name'] / f'attempt-{attempt}'
                command = [sys.executable, str(runner), '--request-file', str(ROOT / row['request']),
                    '--out', str(out), '--peer-review', str(review_path), '--execute-reviewed']
                child = subprocess.run(command, cwd=ROOT, stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE, creationflags=subprocess.CREATE_NO_WINDOW, timeout=40)
                assert out.is_dir(), 'Runner failed before retaining its attempt directory'
                (out / 'wrapper.stdout.txt').write_bytes(child.stdout)
                (out / 'wrapper.stderr.txt').write_bytes(child.stderr)
                detail = json.loads((out / 'report.json').read_text())
                record = dict(case=row['name'], attempt=attempt, exit=child.returncode,
                    report=str(out / 'report.json'),
                    native_entry_authorized=detail['native_font_entry_started'],
                    native_error=detail.get('native_error'), mapping_failure=detail.get('mapping_failure'))
                report['attempts'].append(record)
                save(report_path, report)
                assert detail['frozen_inputs_unchanged']
                if child.returncode == 0 and detail.get('glyph_success'):
                    success = out
                    break
                mapping = detail.get('mapping_failure')
                retryable = (mapping is not None and mapping['stage'] in
                    ['reservation_failure', 'commit_failure'] and mapping['actual_return'] == '00000000'
                    and detail['completed_stage'] == 1 and not detail['native_font_entry_started']
                    and not detail.get('unexpected_exceptions')
                    and (out / 'reply.bin').stat().st_size == 0
                    and (out / 'stderr.bin').stat().st_size == 0)
                if not retryable:
                    report['failed_case'] = row['name']
                    raise RuntimeError('Native/protocol/context failure; no retry or exclusion')
            if success is None:
                report['failed_case'] = row['name']
                raise RuntimeError('Bounded pre-source mapping attempts exhausted')
            parsed = json.loads((success / 'parsed.json').read_text())
            assert parsed['success'] and parsed['count'] == len(row['policy']['glyphs'])
            for glyph in parsed['glyphs']:
                index, semantic = glyph['glyph_index'], glyph['semantic_sha256']
                key = (row['policy']['mode'], tuple(row['policy']['glyphs']), glyph['ordinal'])
                if key in context_baseline and context_baseline[key] != semantic:
                    report['heap_policy_differences'].append(dict(case=row['name'], glyph=index,
                        baseline=context_baseline[key], actual=semantic))
                else:
                    context_baseline[key] = semantic
                if index in baseline and baseline[index] != semantic:
                    report['semantic_differences'].append(dict(case=row['name'], glyph=index,
                        baseline=baseline[index], actual=semantic))
                else:
                    baseline[index] = semantic
            report['completed_cases'] += 1
            report['glyph_observations'] += len(parsed['glyphs'])
            report['unique_glyphs'] = len(baseline)
            save(report_path, report)
            print(json.dumps(dict(completed_cases=report['completed_cases'], unique_glyphs=len(baseline),
                semantic_differences=len(report['semantic_differences']))), flush=True)
        assert report['glyph_observations'] == 4624 and len(baseline) == 578
        # Fresh and sequential rendering are distinct native input states.
        # Preserve their differences; require only identical ordered inputs to
        # agree across the four allocation/poison policies.
        assert len(context_baseline) == 1156
        assert not report['heap_policy_differences'], 'Identical glyph/context inputs depend on heap policy'
        report['fresh_vs_sequence_differing_glyphs'] = sorted({
            row['glyph'] for row in report['semantic_differences']})
        lines = ''.join(f'{index:03d}\t{baseline[index]}\n' for index in sorted(baseline))
        (cohort / 'semantic-glyphs-v1.tsv').write_bytes(lines.encode())
        report['semantic_glyphs_sha256'] = hashlib.sha256(lines.encode()).hexdigest()
        report['whole_cohort_completed'] = True
    except BaseException as error:
        report['error'] = repr(error)
        raise
    finally:
        changed = [path for path, digest in pins.items() if sha(Path(path)) != digest]
        report['pinned_inputs_unchanged'] = not changed
        report['changed_files'] = changed
        save(report_path, report)
        assert not changed


if __name__ == '__main__':
    main()
