"""Compare pinned executables on two explicitly pinned source-derived fixtures.

A=clip and B=model is a caller-selected hypothesis, not a runtime association.
"""
import argparse
import hashlib
import json
import struct
import subprocess

from cell_hierarchy_76f930_compare_v1 import TARGET, digest, expected_hash, inside_target
from cell_hierarchy_76f930_corpus_v2 import validate


def fixture(path, expected, placement):
    if not path.is_file() or digest(path) != expected:
        raise ValueError(f"fixture hash mismatch: {path}")
    data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError(f"fixture changed while reading: {path}")
    # 16-byte header and 27 little-endian i32 values, with no trailing bytes.
    if len(data) != 124 or data[:4] != b"H930":
        raise ValueError(f"invalid fixture framing: {path}")
    if struct.unpack_from("<III", data, 4) != (9, 9, placement):
        raise ValueError(f"invalid fixture header: {path}")
    words = struct.unpack_from("<27i", data, 16)
    validate(words[:9], words[9:18], words[18:])
    return data


def invoke(path, data, timeout):
    result = subprocess.run([str(path)], input=data, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, cwd=str(TARGET), timeout=timeout,
                            creationflags=subprocess.CREATE_NO_WINDOW, check=False)
    output = result.stdout.replace(b"\r\n", b"\n")
    if result.returncode or result.stderr or output not in (b"0\n", b"1\n"):
        raise ValueError(f"invalid process result: {path}, exit {result.returncode}")
    return int(output[:1])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for label in ("host", "rust", "left", "right"):
        parser.add_argument("--" + label, type=inside_target, required=True)
        parser.add_argument("--" + label + "-sha256", type=expected_hash, required=True)
    parser.add_argument("--out", type=inside_target, required=True)
    parser.add_argument("--timeout", type=float, default=10.0)
    args = parser.parse_args()
    paths = [(name, getattr(args, name), getattr(args, name + "_sha256"))
             for name in ("host", "rust", "left", "right")]
    if len({path for _, path, _ in paths}) != 4 or not 0 < args.timeout <= 60:
        parser.error("paths must be distinct and timeout must be in (0, 60]")
    args.out.mkdir(parents=True, exist_ok=False)
    report = {"matched": False, "completed": 0,
              "scope": "caller-selected A=clip, B=model hypothesis; runtime association and pose unverified"}
    try:
        for name, path, expected in paths:
            if not path.is_file() or digest(path) != expected:
                raise ValueError(f"{name} file missing or SHA-256 mismatch")
            if name in ("host", "rust") and path.suffix.lower() != ".exe":
                raise ValueError(f"{name} must be an executable")
            report[name + "_sha256_before"] = expected
        payloads = [fixture(args.left, args.left_sha256, 0), fixture(args.right, args.right_sha256, 1)]
        if payloads[0][16:] != payloads[1][16:]:
            raise ValueError("fixture arrays differ between placements")
        outcomes = []
        for payload in payloads:
            host = invoke(args.host, payload, args.timeout)
            rust = invoke(args.rust, payload, args.timeout)
            outcomes.append((host, rust))
            report["completed"] += 1
            if host != rust:
                raise ValueError("host and Rust results differ")
        if outcomes[0] != outcomes[1]:
            raise ValueError("results differ across placements")
        report["result"] = outcomes[0][0]
        report["matched"] = True
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        report["error"] = str(exc)
        raise
    finally:
        for name, path, expected in paths:
            try:
                after = digest(path)
                report[name + "_sha256_after"] = after
                if after != expected:
                    report["matched"] = False
                    report["error"] = f"{name} changed or failed pin verification"
            except OSError as exc:
                report["matched"] = False
                report["error"] = str(exc)
        (args.out / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    if not report["matched"]:
        raise ValueError(report.get("error", "comparison failed"))
    print(json.dumps(report))


if __name__ == "__main__":
    main()
