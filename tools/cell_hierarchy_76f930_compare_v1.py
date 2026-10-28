"""Compare two already-built, explicitly hash-pinned fixture executables on Windows.

This wrapper does not build either executable or establish any runtime association.
"""
import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGET = (ROOT / "target").resolve()


def digest(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def inside_target(value):
    path = Path(value).resolve()
    try:
        path.relative_to(TARGET)
    except ValueError as exc:
        raise argparse.ArgumentTypeError("path must be under target") from exc
    if path == TARGET:
        raise argparse.ArgumentTypeError("path must be a child of target")
    return path


def expected_hash(value):
    if not re.fullmatch(r"[0-9a-fA-F]{64}", value):
        raise argparse.ArgumentTypeError("expected a SHA-256 hex digest")
    return value.lower()


def run_binary(path, expected, timeout):
    if not path.is_file() or path.suffix.lower() != ".exe":
        raise ValueError(f"expected an executable file: {path}")
    before = digest(path)
    if before != expected:
        raise ValueError(f"SHA-256 mismatch before execution: {path}")
    try:
        result = subprocess.run(
            [str(path)], cwd=str(TARGET), stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout,
            creationflags=subprocess.CREATE_NO_WINDOW, check=False,
        )
    finally:
        after = digest(path)
        if after != before:
            raise ValueError(f"executable changed during execution: {path}")
    if result.returncode != 0:
        raise ValueError(f"executable returned {result.returncode}: {path}")
    if result.stderr:
        raise ValueError(f"executable wrote to stderr: {path}")
    try:
        output = result.stdout.decode("ascii").replace("\r\n", "\n")
    except UnicodeDecodeError as exc:
        raise ValueError(f"non-ASCII stdout: {path}") from exc
    if not re.fullmatch(r"(?:[01]\n){8}", output):
        raise ValueError(f"expected exactly eight binary result lines: {path}")
    return output, before, after


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", required=True, type=inside_target)
    parser.add_argument("--host-sha256", required=True, type=expected_hash)
    parser.add_argument("--rust", required=True, type=inside_target)
    parser.add_argument("--rust-sha256", required=True, type=expected_hash)
    parser.add_argument("--out", required=True, type=inside_target)
    parser.add_argument("--timeout", type=float, default=10.0)
    args = parser.parse_args()
    if not 0 < args.timeout <= 60:
        parser.error("timeout must be greater than zero and at most 60 seconds")
    if args.host == args.rust:
        parser.error("host and Rust executable must be distinct files")
    # Reserve a fresh destination before either executable is started.
    args.out.mkdir(parents=True, exist_ok=False)
    report = {"matched": False, "scope": "controlled fixtures only; no live association or pose claim"}
    try:
        # Validate both inputs before starting either process.
        for path, expected in ((args.host, args.host_sha256), (args.rust, args.rust_sha256)):
            if not path.is_file() or path.suffix.lower() != ".exe" or digest(path) != expected:
                raise ValueError(f"missing executable or SHA-256 mismatch: {path}")
        host, host_before, host_after = run_binary(args.host, args.host_sha256, args.timeout)
        (args.out / "host.stdout.txt").write_bytes(host.encode("ascii"))
        rust, rust_before, rust_after = run_binary(args.rust, args.rust_sha256, args.timeout)
        (args.out / "rust.stdout.txt").write_bytes(rust.encode("ascii"))
        report.update(host_sha256_before=host_before, host_sha256_after=host_after,
                      rust_sha256_before=rust_before, rust_sha256_after=rust_after,
                      host=str(args.host), rust=str(args.rust), matched=host == rust,
                      fixture_count=8)
        if host != rust:
            raise ValueError("fixture outputs differ")
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        report["error"] = str(exc)
        raise
    finally:
        (args.out / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report))


if __name__ == "__main__":
    main()
