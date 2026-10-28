"""Assemble retained, successful native smoke histories for a Rust comparison.

The larger run stopped at its coverage assertion. This does not retry native
execution or claim that the full corpus or endpoint branch passed.
"""
import argparse
import hashlib
import json
import shutil
from pathlib import Path

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
    run = CELL / "runtime-fresh-sampled-channel-native-run-v2"
    protocol = run / "protocol"
    failure = json.loads((run / "failure.json").read_text())
    immutability = json.loads((run / "immutability-v1.json").read_text())
    assert failure["retained"] and failure["retries"] == 0
    assert "endpoint_fullslots" in failure["error"] and immutability["unchanged"]
    full_input = (inputs / "input.bin").read_bytes()
    cases = json.loads((inputs / "cases.json").read_text())
    rows = (inputs / "cases.tsv").read_text().splitlines()
    old_clock = (CELL / "runtime-fresh-loop-native-run-v1/native-projection792.bin").read_bytes()
    assert len(full_input) == 300 * 256 and len(cases) == 300 and len(rows) == 301
    assert len(old_clock) == 300 * 792
    output.mkdir(parents=True, exist_ok=False)
    (output / "clips").mkdir()
    requests, replies, table = bytearray(), bytearray(), [rows[0]]
    records, counts = [], {"endpoint_fullslots": 0, "interpolation": 0, "authored_events": 0, "snapshots": 0}
    for source in range(5):
        for family in range(5):
            original = source * 60 + family
            label = "first_genuine_history" if original == 0 else f"source{source}_history{family}"
            request = (protocol / f"{label}-input.bin").read_bytes()
            response = (protocol / f"{label}-output.bin").read_bytes()
            status = json.loads((protocol / f"{label}-status.json").read_text())
            inspection = json.loads((protocol / f"{label}-inspection.json").read_text())
            assert request == full_input[original * 256:(original + 1) * 256]
            assert len(response) == 5452 and status["exit"] == 0
            assert not status["timeout"] and status["stderr_bytes"] == 0
            assert cases[original]["source_case"] == source
            assert len(inspection["details"]) == 9
            for snap in range(9):
                old_frame = old_clock[original * 792 + snap * 88 + 16:original * 792 + snap * 88 + 20]
                new_frame = response[448 + snap * 556 + 16:448 + snap * 556 + 20]
                assert old_frame == new_frame
            for key in counts:
                counts[key] += inspection["counts"][key]
            fields = rows[original + 1].split("\t")
            assert len(fields) == 4 and int(fields[0]) == original
            clip = Path(fields[1].replace("\\", "/"))
            assert not clip.is_absolute() and ".." not in clip.parts
            destination = output / clip
            if not destination.exists():
                shutil.copyfile(inputs / clip, destination)
            fields[0] = str(len(records))
            table.append("\t".join(fields))
            requests.extend(request)
            replies.extend(response)
            records.append({"subset": len(records), "original": original, "label": label,
                            "request_sha256": sha(request), "response_sha256": sha(response)})
    assert counts == {"endpoint_fullslots": 0, "interpolation": 225, "authored_events": 30, "snapshots": 225}
    (output / "input.bin").write_bytes(requests)
    (output / "native.bin").write_bytes(replies)
    (output / "cases.tsv").write_text("\n".join(table) + "\n", encoding="utf-8", newline="\n")
    report = {"schema": "retained-native-sampled-channel-smoke-subset-v1", "records": records,
              "counts": counts, "request_sha256": sha(requests), "native_sha256": sha(replies),
              "previous_native_clock_frames_match": 225, "new_native_execution": False,
              "endpoint_branch_covered": False, "full_corpus_accepted": False}
    (output / "manifest.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: report[key] for key in ("counts", "request_sha256", "native_sha256")}))


if __name__ == "__main__":
    main()
