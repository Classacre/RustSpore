"""Audit the sampled-channel corpus against retained native clock frames.

This reads existing evidence only. It does not execute the original code and
does not treat a predicted endpoint as a tested endpoint.
"""
import argparse
import collections
import hashlib
import json
import struct
from pathlib import Path

import cell_runtime_fresh_sampled_channel_inputs as fixture

ROOT = Path(__file__).resolve().parent.parent
CELL = ROOT / "target/source-cell"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    output = args.out.resolve()
    output.relative_to(CELL.resolve())
    inputs = CELL / "runtime-fresh-sampled-channel-inputs-v1"
    clock_dir = CELL / "runtime-fresh-loop-native-run-v1"
    native_dir = CELL / "runtime-fresh-sampled-channel-native-run-v2"
    clock = (clock_dir / "native-projection792.bin").read_bytes()
    proof = json.loads((clock_dir / "rust-comparison-v1.json").read_text())
    cases = json.loads((inputs / "cases.json").read_text())
    requests = (inputs / "input.bin").read_bytes()
    sources = fixture.host.source_cases()
    assert proof["accepted"] and proof["records"] == 300 and proof["snapshots"] == 2700
    assert sha(clock) == proof["projection_sha256"]
    assert len(clock) == 300 * 792 and len(requests) == 300 * 256 and len(cases) == 300
    counts = collections.Counter()
    per_clip = [collections.Counter() for _ in sources]
    evidence_matches = 0
    for index, case in enumerate(cases):
        source_index = case["source_case"]
        source = sources[source_index]
        channel, data = source["channels"][0], source["raw"]
        count = channel["count"]
        stride = fixture.word(data, channel["descriptors"] + 12)
        timestamps = [fixture.word(data, channel["keys"] + k * stride) for k in range(count)]
        assert count > 1 and timestamps == sorted(timestamps)
        for snapshot in range(9):
            frame_bytes = clock[index * 792 + snapshot * 88 + 16:index * 792 + snapshot * 88 + 20]
            frame = struct.unpack("<f", frame_bytes)[0]
            upper = sum(timestamp <= frame for timestamp in timestamps)
            left = min(count - 1, max(0, upper - 1))
            right = min(count - 1, upper)
            kind = "endpoint" if left == right else "interpolation"
            counts[kind] += 1
            per_clip[source_index][kind] += 1
            if index % 60 < 5:
                label = "first_genuine_history" if index == 0 else f"source{source_index}_history{index % 60}"
                native = (native_dir / "protocol" / f"{label}-output.bin").read_bytes()
                assert len(native) == 5452
                actual = native[448 + snapshot * 556 + 16:448 + snapshot * 556 + 20]
                assert frame_bytes == actual
                evidence_matches += 1
    assert counts["endpoint"] == 0 and counts["interpolation"] == 2700
    assert evidence_matches == 225
    report = {"schema": "sampled-channel-corpus-clock-coverage-v1",
              "old_clock_projection_sha256": sha(clock), "input_sha256": sha(requests),
              "counts": dict(counts), "per_clip": [dict(x) for x in per_clip],
              "actual_sampled_native_frames_match": evidence_matches,
              "endpoint_tested": False, "new_native_execution": False,
              "scope": "Predicts key selection for this retained clock corpus; does not validate sampler output."}
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("x", encoding="utf-8", newline="\n") as handle:
        json.dump(report, handle, indent=2)
        handle.write("\n")
    print(json.dumps(report))


if __name__ == "__main__":
    main()
