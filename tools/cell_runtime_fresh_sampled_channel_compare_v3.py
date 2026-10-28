"""Compare the retained full Cell sampling projection with the Rust result."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CELL = ROOT / "target/source-cell"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-dir", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    run = args.run_dir.resolve()
    output = args.out.resolve()
    run.relative_to(CELL.resolve())
    output.relative_to(CELL.resolve())
    report = json.loads((run / "report.json").read_text())
    immutable = json.loads((run / "immutability-v1.json").read_text())
    native, rust = run / "native-projection4896.bin", run / "rust-v2.bin"
    log = (run / "rust-v2.log").read_text(errors="replace")
    expected = "CELL_RUNTIME_FRESH_SAMPLED_CHANNEL_VERIFY cases=300 snapshots=2700 projection_bytes=4896 incoming_states=3 positive_typed_guards=2700 actual_source_events=true"
    assert report["native_executed"] and report["records"] == 300 and report["snapshots"] == 2700
    assert report["counts"] == {"endpoint_fullslots": 0, "interpolation": 2700, "authored_events": 369, "snapshots": 2700}
    assert report["retained_clock_frames_reproduced"] == 2700 and report["retained_smoke_histories_reproduced"] == 25
    assert report["protocol_rejections"] == 28 and report["framing_checks"] == 5
    assert report["runner_independently_reviewed"] is False and immutable["unchanged"]
    assert len(native.read_bytes()) == 300 * 4896 and native.read_bytes() == rust.read_bytes()
    assert sha(native) == report["native_projection_sha256"] and log.count(expected) == 1
    sources = [ROOT / "crates/spore-core/src/cell_runtime_fresh_sampled_channel_v2.rs",
               ROOT / "crates/spore-core/examples/cell_runtime_fresh_sampled_channel_full_verify_v2.rs",
               ROOT / "tools/cell_runtime_fresh_sampled_channel_corpus_run_v3.py"]
    result = {"schema": "fresh-sampled-channel-rust-comparison-v3", "matched": True,
              "records": 300, "snapshots": 2700, "source_clips": 5, "x87_contexts": 12,
              "incoming_sse_states": 3, "typed_guard_checks": 2700,
              "source_event_occurrences": 369, "projection_sha256": sha(native),
              "raw_native_sha256": report["raw_output_sha256"],
              "source_sha256": {str(path.relative_to(ROOT)): sha(path) for path in sources},
              "native_pins_unchanged": immutable["unchanged"], "native_pins": immutable["pinned_files"],
              "endpoint_branch_covered": False, "runner_independently_reviewed": False,
              "scope": "Fresh explicit first-channel interpolation and event records; remaining channel masks zero. No endpoint, actor, scheduling or event-dispatch acceptance."}
    with output.open("x", encoding="utf-8", newline="\n") as handle:
        json.dump(result, handle, indent=2)
        handle.write("\n")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
