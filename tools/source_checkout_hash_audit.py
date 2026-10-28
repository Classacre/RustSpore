"""Verify Git checkout filters preserve the bytes used by source hash pins.

Writes only content-addressed Git blobs and a new ignored target report. It
does not change the index, HEAD, source files, configuration or remote state.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def git(*args, data=None):
    return subprocess.check_output(['git', *args], cwd=ROOT, input=data)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    output = args.out.resolve()
    output.relative_to((ROOT/'target').resolve())
    assert not output.exists()
    names = set(git('ls-files', '-z').decode().split('\0'))
    names.update(git('ls-files', '--others', '--exclude-standard', '-z').decode().split('\0'))
    rows, counts, failures = [], Counter(), []
    for name in sorted(names):
        path = ROOT/name
        if not path.is_file() or not name.endswith(('.rs', '.py', '.rs.pending', '.bat')):
            continue
        raw = path.read_bytes()
        digest = hashlib.sha256(raw).hexdigest()
        lf, crlf = raw.count(b'\n'), raw.count(b'\r\n')
        style = 'mixed' if 0 < crlf < lf else 'crlf' if crlf else 'lf'
        # Apply clean and checkout filters through Git itself, without staging.
        blob = git('hash-object', '-w', '--stdin', '--path='+name, data=raw).decode().strip()
        restored = git('cat-file', '--filters', '--path='+name, blob)
        checkout_digest = hashlib.sha256(restored).hexdigest()
        if raw != restored:
            failures.append(name)
        counts[style] += 1
        rows.append(dict(path=name, style=style, working_sha256=digest,
            git_blob=blob, checkout_sha256=checkout_digest, exact=raw == restored))
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open('x') as report:
        json.dump(dict(schema='hash-bound-source-git-checkout-audit-v1', files=len(rows),
            exact=not failures, failures=failures, styles=dict(counts), rows=rows,
            index_HEAD_source_config_remote_unchanged=True), report, indent=2)
        report.write('\n')
    print(json.dumps(dict(files=len(rows), exact=not failures, failures=failures, styles=dict(counts))))
    if failures:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
