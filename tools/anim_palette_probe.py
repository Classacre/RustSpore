"""Independent per-destination palette reference on explicit synthetic poses.

The measured native first-group clamp precedes identity/evaluated fallback.
No source runtime rig or animation curve is inferred by these fixtures.
"""
from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import struct
from collections import Counter
from pathlib import Path


IDENTITY = [0x3F800000 if i in (0, 5, 10) else 0 for i in range(12)]


def matrix(kind: int, bone: int) -> list[int]:
    return [0x80000000 if bone == 0 and component == 0 else
            0x7FA12345 + kind if bone == 0 and component == 1 else
            0x3F000000 + kind * 0x00800000 + bone * 0x1000 + component
            for component in range(12)]


def reference(values: tuple[int, ...]) -> tuple[int, int, list[list[int]]]:
    first, second, requested, supplied, first_evaluated, second_evaluated, ready = values
    bound = min(first, requested)
    copied = bound if supplied and ready else 0
    result = []
    if ready:
        # Select each destination directly, independently from the Rust
        # implementation's prefix and group-range copy operations.
        for index in range(first + second):
            if supplied and index < bound:
                result.append(matrix(0, index))
            elif index < first:
                result.append(matrix(1, index) if first_evaluated else IDENTITY)
            else:
                result.append(matrix(2, index - first) if second_evaluated else IDENTITY)
    return bound, copied, result


def digest(values: tuple[int, ...], result: tuple[int, int, list[list[int]]]) -> str:
    bound, copied, matrices = result
    feed = bytearray(b"runtime-palette-v1\0")
    feed.extend(struct.pack("<10I", *values, bound, copied, len(matrices)))
    for matrix_values in matrices:
        feed.extend(struct.pack("<12I", *matrix_values))
    value = 0xCBF29CE484222325
    for byte in feed:
        value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{value:016X}"


def fixtures():
    for first in range(128):
        # Every possible nonnegative native count, with the arithmetic branch
        # boundaries and larger source-table counts tested explicitly.
        requests = sorted({0, 1, max(0, first - 1), first, first + 1, 127, 128, 255, 256})
        for requested in requests:
            second = 1 + (first + requested) % 3
            for supplied, first_evaluated, second_evaluated, ready in itertools.product(range(2), repeat=4):
                yield (first, second, requested, supplied, first_evaluated, second_evaluated, ready)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixtures", type=Path, required=True)
    parser.add_argument("--digest-out", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--source-inventory", type=Path)
    parser.add_argument("--rust-digest", type=Path)
    args = parser.parse_args()
    for path in (args.fixtures, args.digest_out, args.out):
        path.parent.mkdir(parents=True, exist_ok=True)
    # The source-row count may exceed the body group count. The two extra
    # generated inputs must not overwrite the following evaluated group.
    golden = reference((3, 2, 5, 1, 0, 1, 1))
    assert golden[:2] == (3, 3)
    assert golden[2] == [matrix(0, 0), matrix(0, 1), matrix(0, 2), matrix(2, 0), matrix(2, 1)]
    assert reference((3, 2, 2, 1, 0, 1, 1))[2][2] == IDENTITY
    lines, counts = [], Counter()
    total_matrices = 0
    with args.fixtures.open("w", encoding="utf-8", newline="\n") as output:
        output.write("# Synthetic inputs: id, first, second, requested, supplied, first_eval, second_eval, ready\n")
        for index, values in enumerate(fixtures()):
            case_id = f"{index:06d}"
            result = reference(values)
            lines.append(f"{case_id}\t{digest(values, result)}")
            output.write(case_id + "\t" + "\t".join(map(str, values)) + "\n")
            total_matrices += len(result[2])
            counts["first_group_clamped"] += values[2] > values[0]
            counts["null_generated_table"] += not values[3]
            counts["ineligible_storage"] += not values[6]
            counts["partial_generated_prefix"] += bool(values[3] and 0 < values[2] < values[0])
    normalized = "\n".join(lines) + "\n"
    args.digest_out.write_text(normalized, encoding="utf-8", newline="\n")
    inventory = None
    if args.source_inventory:
        data = args.source_inventory.read_bytes()
        source = json.loads(data)
        inventory = {
            "report": str(args.source_inventory), "report_sha256": hashlib.sha256(data).hexdigest(),
            "source_input_digests": source["inputs"],
            "constraints": source["native_first_group_source_constraints"],
            "association_scope": source["scope"],
        }
    comparison = None
    problems = []
    if args.rust_digest:
        rust = args.rust_digest.read_text(encoding="utf-8").splitlines()
        problems = [{"line": i + 1, "expected": a, "rust": b}
                    for i, (a, b) in enumerate(itertools.zip_longest(lines, rust)) if a != b]
        comparison = {"rust_digest": str(args.rust_digest), "exact_match": not problems,
                      "expected_records": len(lines), "rust_records": len(rust)}
    report = {
        "scope": "explicit runtime groups and synthetic supplied/evaluated matrices; no source rig inferred",
        "native_procedures": ["73B7C0", "740530"],
        "cases": len(lines), "assembled_matrices": total_matrices,
        "nonnegative_first_group_counts": 128, "branch_counts": counts,
        "normalized_digest_sha256": hashlib.sha256(normalized.encode()).hexdigest(),
        "source_inventory": inventory, "rust_comparison": comparison,
        "problems": problems[:20], "problem_count": len(problems),
    }
    args.out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: report[key] for key in ("cases", "assembled_matrices",
                                                "normalized_digest_sha256", "problem_count")}))
    if problems:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
