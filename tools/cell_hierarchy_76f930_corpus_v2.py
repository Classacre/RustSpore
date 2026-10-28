"""Compare pinned fixture executables on a deterministic synthetic H930 corpus.

The corpus is synthetic and does not establish a live resource association.
"""
import argparse
import hashlib
import json
import random
import struct
import subprocess
from pathlib import Path

from cell_hierarchy_76f930_compare_v1 import TARGET, digest, expected_hash, inside_target

CAP = 256


def validate(a, b, mapping):
    if len(a) > CAP or len(b) > CAP or len(mapping) != len(a):
        raise ValueError("invalid array lengths")
    if any(p < -1 or p >= i for i, p in enumerate(a)):
        raise ValueError("invalid A parent")
    if any(p < -1 or p >= len(b) for p in b):
        raise ValueError("invalid B parent")
    if any(m < -1 or m >= len(b) for m in mapping):
        raise ValueError("invalid mapping")
    for start in range(len(b)):
        seen = set()
        node = start
        while node != -1:
            if node in seen:
                raise ValueError("cyclic B parents")
            seen.add(node)
            node = b[node]


def corpus():
    base = [
        ([], [], []), ([-1], [-1], [-1]), ([-1], [-1], [0]),
        ([-1, 0, 1], [-1, 0], [0, -1, 1]),
        ([-1, 0, 1], [-1, -1], [0, -1, 1]),
        ([-1, 0], [-1, 0], [-1, 1]), ([-1, 0], [-1, 0], [1, 0]),
        ([-1, 0, 1], [-1, 0, 1], [0, 1, 2]),
        ([-1, 0], [-1], [0, 0]),
        ([-1] + list(range(255)), [-1] + list(range(255)), list(range(256))),
    ]
    cases = [("baseline", *case) for case in base]
    cases.extend([
        ("branching_forest_match", [-1, 0, 0, -1, 3], [-1, 0, 0, -1, 3], [0, 1, 2, 3, 4]),
        ("branching_forest_mismatch", [-1, 0, 0, -1, 3], [-1, 0, 1, -1, 3], [0, 1, 2, 3, 4]),
        ("multiple_unmapped_ancestors_match", [-1, 0, 1, 2], [-1, 0], [0, -1, -1, 1]),
        ("multiple_unmapped_ancestors_mismatch", [-1, 0, 1, 2], [-1, -1], [0, -1, -1, 1]),
        ("mapped_ancestor_mismatch", [-1, 0, 1], [-1, 0, 0], [0, 1, 2]),
        ("unmapped_root_failure", [-1, 0, 1], [-1], [-1, -1, 0]),
        ("unmapped_root_separate_tree_match", [-1, -1, 1], [-1, 0], [-1, 0, 1]),
        ("duplicate_ordinals_match", [-1, -1], [-1], [0, 0]),
        ("duplicate_ordinals_mismatch", [-1, 0], [-1], [0, 0]),
        ("forward_b_parent_match", [-1, 0, 1], [1, 2, -1], [2, 1, 0]),
        ("forward_b_parent_mismatch", [-1, 0, 1], [2, 2, -1], [2, 1, 0]),
    ])
    rng = random.Random(0x76F930)
    for _ in range(54):
        na, nb = rng.randrange(0, 33), rng.randrange(0, 33)
        a = [rng.randrange(-1, i) for i in range(na)]
        b = [rng.randrange(-1, i) for i in range(nb)]
        mapping = [rng.randrange(-1, nb) for _ in range(na)]
        cases.append(("seeded_random", a, b, mapping))
    return cases


def encode(case, placement):
    a, b, mapping = case
    validate(a, b, mapping)
    values = a + b + mapping
    return b"H930" + struct.pack("<III", len(a), len(b), placement) + struct.pack(f"<{len(values)}i", *values)


def invoke(executable, payload, timeout):
    result = subprocess.run([str(executable)], input=payload, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, timeout=timeout, cwd=str(TARGET),
                            creationflags=subprocess.CREATE_NO_WINDOW, check=False)
    if result.returncode or result.stderr:
        raise ValueError(f"{executable} returned {result.returncode} or wrote stderr")
    output = result.stdout.replace(b"\r\n", b"\n")
    if output not in (b"0\n", b"1\n"):
        raise ValueError(f"invalid output from {executable}")
    return int(output[:1])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", required=True, type=inside_target)
    parser.add_argument("--host-sha256", required=True, type=expected_hash)
    parser.add_argument("--rust", required=True, type=inside_target)
    parser.add_argument("--rust-sha256", required=True, type=expected_hash)
    parser.add_argument("--out", required=True, type=inside_target)
    parser.add_argument("--timeout", type=float, default=10.0)
    args = parser.parse_args()
    if not 0 < args.timeout <= 60 or args.host == args.rust:
        parser.error("invalid timeout or identical executable paths")
    args.out.mkdir(parents=True, exist_ok=False)
    report = {"matched": False, "scope": "deterministic synthetic fixtures; no live association or pose claim",
              "seed": "0x76f930", "completed": 0}
    records = []
    try:
        for path, expected in ((args.host, args.host_sha256), (args.rust, args.rust_sha256)):
            if not path.is_file() or path.suffix.lower() != ".exe" or digest(path) != expected:
                raise ValueError(f"missing executable or SHA-256 mismatch: {path}")
        cases = corpus()
        report.update(case_count=len(cases), placements=2, host_sha256_before=digest(args.host),
                      rust_sha256_before=digest(args.rust))
        for index, (category, a, b, mapping) in enumerate(cases):
            case = (a, b, mapping)
            for placement in (0, 1):
                payload = encode(case, placement)
                native = invoke(args.host, payload, args.timeout)
                rust = invoke(args.rust, payload, args.timeout)
                records.append({"case": index, "category": category, "placement": placement, "input_sha256": hashlib.sha256(payload).hexdigest(),
                                "host": native, "rust": rust})
                report["completed"] += 1
                if native != rust:
                    raise ValueError(f"fixture mismatch: case {index}, placement {placement}")
        report["matched"] = True
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        report["error"] = str(exc)
        raise
    finally:
        for label, path, expected in (("host", args.host, args.host_sha256), ("rust", args.rust, args.rust_sha256)):
            try:
                after = digest(path)
                report[label + "_sha256_after"] = after
                if after != expected:
                    report["matched"] = False
                    report["error"] = label + " executable changed"
            except OSError as exc:
                report["matched"] = False
                report["error"] = str(exc)
        (args.out / "results.json").write_text(json.dumps(records, indent=2) + "\n", encoding="utf-8")
        (args.out / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    if not report["matched"]:
        raise ValueError(report.get("error", "comparison failed"))
    print(json.dumps(report))


if __name__ == "__main__":
    main()
