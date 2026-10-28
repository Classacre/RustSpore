"""Run the reviewed Cell host on the retained corpus without requiring endpoints.

The prior attempt stopped after 25 successful histories because its smoke guard
required an endpoint. A separate retained-clock audit proves that these 300
histories predict no endpoints. This runner keeps that branch explicitly open.
The host and old runner have independent reviews; this new orchestration has a
documented author audit and must not be described as independently reviewed.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path
from types import SimpleNamespace

import cell_runtime_fresh_sampled_channel_native_run_v1 as previous
import cell_runtime_fresh_sampled_channel_hybrid_audit_v3 as linked
from cell_runtime_data_native_run import pinned, saved_file_run

ROOT = Path(__file__).resolve().parent.parent
CELL = ROOT / "target/source-cell"
PLAN = CELL / "runtime-fresh-sampled-channel-runner-plan-v2/plan.json"
PEER = CELL / "runtime-fresh-sampled-channel-runner-plan-v2/binding-runner-peer-v2.json"
COVERAGE = CELL / "runtime-fresh-sampled-channel-coverage-v1/audit.json"
SMOKE = CELL / "runtime-fresh-sampled-channel-native-run-v2"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def save(path, value):
    with path.open("x", encoding="utf-8", newline="\n") as handle:
        json.dump(value, handle, indent=2)
        handle.write("\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--run", action="store_true")
    args = parser.parse_args()
    output = args.out.resolve()
    output.relative_to(CELL.resolve())
    plan, peer, coverage = (json.loads(path.read_text()) for path in (PLAN, PEER, COVERAGE))
    assert plan["source_proved_owned_event_tail_successor"]
    assert peer["no_blocking_static_issue_found"] and peer["native_executed"] is False
    for path, expected in plan["pins"].items():
        assert sha(path) == expected, path
    for path, expected in peer["pins"].items():
        assert sha(path) == expected, path
    assert sha(PLAN) in peer["pins"].values() and plan["runner_sha256"] in peer["pins"].values()
    host, inputs = Path(plan["host"]), Path(plan["inputs"])
    carrier, exe, mapping = host / "carrier-x86.exe", host / "host-x86.exe", host / "host-x86.map"
    for path, key in ((carrier, "carrier_sha256"), (exe, "owned_sha256"), (mapping, "map_sha256")):
        assert sha(path) == plan[key] and plan[key] in peer["pins"].values()
    images = linked.source.arithmetic.Images(Path("D:/Games/SPORE Collection/SporeBin/SporeApp.exe"),
                                              Path("D:/Games/SPORE Collection/SporeBin/msvcr90.dll"))
    assembly = (host / "host-x86.asm").read_text()
    assert assembly.count(linked.source.SHADOW) == 3 and assembly == linked.source.host_source(images)
    audit = linked.previous.audit(SimpleNamespace(carrier=carrier, owned_exe=exe, owned_map=mapping,
        source_exe=Path("D:/Games/SPORE Collection/SporeBin/SporeApp.exe"),
        source_crt=Path("D:/Games/SPORE Collection/SporeBin/msvcr90.dll")))
    assert audit["static_pass"] and audit["carrier_sha256"] == plan["carrier_sha256"]
    assert coverage["counts"] == {"interpolation": 2700}
    assert coverage["actual_sampled_native_frames_match"] == 225 and not coverage["endpoint_tested"]
    raw = (inputs / "input.bin").read_bytes()
    cases = json.loads((inputs / "cases.json").read_text())
    protocols = json.loads((inputs / "report.json").read_text())["protocols"]
    assert len(raw) == 300 * 256 and len(cases) == 300 and sha(inputs / "input.bin") == coverage["input_sha256"]
    old_clock_path = CELL / "runtime-fresh-loop-native-run-v1/native-projection792.bin"
    old_clock = old_clock_path.read_bytes()
    assert len(old_clock) == 300 * 792 and sha(old_clock_path) == coverage["old_clock_projection_sha256"]
    for index, case in enumerate(cases):
        duration = struct.unpack("<d", bytes.fromhex(case["duration_bits"])[::-1])[0]
        previous.fixture.domain(raw[index * 256:(index + 1) * 256], case["source_case"], duration)
    assert json.loads((SMOKE / "immutability-v1.json").read_text())["unchanged"]
    failure = json.loads((SMOKE / "failure.json").read_text())
    assert "endpoint_fullslots" in failure["error"] and failure["retries"] == 0
    retained = {}
    files = [PLAN, PEER, COVERAGE, Path(__file__), inputs / "input.bin", inputs / "cases.json",
             inputs / "report.json", old_clock_path,
             CELL / "runtime-fresh-loop-native-run-v1/rust-comparison-v1.json",
             ROOT / "tools/cell_runtime_fresh_sampled_channel_coverage_audit_v1.py",
             SMOKE / "failure.json", SMOKE / "immutability-v1.json"]
    files.extend(Path(path) for path in plan["pins"])
    files.extend(Path(path) for path in peer["pins"])
    for clip in range(5):
        for family in range(5):
            index = clip * 60 + family
            label = "first_genuine_history" if index == 0 else f"source{clip}_history{family}"
            prefix = SMOKE / "protocol" / label
            request, response = Path(str(prefix) + "-input.bin"), Path(str(prefix) + "-output.bin")
            status, inspection = Path(str(prefix) + "-status.json"), Path(str(prefix) + "-inspection.json")
            result, checked = json.loads(status.read_text()), json.loads(inspection.read_text())
            assert request.read_bytes() == raw[index * 256:(index + 1) * 256]
            assert result["exit"] == 0 and not result["timeout"] and result["stderr_bytes"] == 0
            assert response.stat().st_size == 5452 and checked["counts"]["snapshots"] == 9
            retained[index] = response.read_bytes()
            files.extend((request, response, status, inspection))
    for name, expected in protocols.items():
        path = inputs / (name + ".bin")
        assert sha(path) == expected
        files.append(path)
    output.mkdir(parents=True, exist_ok=False)
    save(output / "preflight.json", {"native_executed": False, "execution_requested": args.run,
         "prior_host_and_runner_reviewed_independently": True, "this_orchestration_independent_review": False,
         "endpoint_coverage": False, "retained_smoke_histories": 25, "corpus_histories": 300,
         "single_attempt": True, "retries": 0, "runner_sha256": sha(__file__)})
    if not args.run:
        print("Preflight passed; no native execution")
        return
    try:
        with pinned(list(dict.fromkeys(path.resolve() for path in files)), output):
            protocol_out = output / "protocol"
            protocol_out.mkdir()

            def execute(label, request, exit_code, expected=None):
                path = protocol_out / (label + "-input.bin")
                path.write_bytes(request)
                request_pins = protocol_out / (label + "-pins")
                request_pins.mkdir()
                with pinned([path], request_pins):
                    status, response_path = saved_file_run(carrier, path, protocol_out, label)
                assert status["exit"] == exit_code and status["stderr_bytes"] == 0 and not status["timeout"]
                response = response_path.read_bytes()
                if expected is not None:
                    assert response == expected, label
                return response

            reference = retained[0]
            execute("empty", b"", 0, b"")
            execute("repeat_two", raw[:256] * 2, 0, reference * 2)
            sequence = execute("valid_sequence", raw[:512], 0, reference + retained[1])
            execute("partial_first", raw[:128], 7, b"")
            execute("complete_then_partial", raw[:256] + raw[:128], 7, reference)
            for name in protocols:
                execute(name, (inputs / (name + ".bin")).read_bytes(), 7, b"")
            status, response_path = saved_file_run(carrier, inputs / "input.bin", output, "corpus")
            assert status["exit"] == 0 and status["stderr_bytes"] == 0 and not status["timeout"]
            native = response_path.read_bytes()
            projection, counts, details = previous.inspect(raw, native, cases)
            assert counts["snapshots"] == 2700 and counts["endpoint_fullslots"] == 0
            assert counts["interpolation"] > 0 and counts["authored_events"] > 0
            for index in range(300):
                for snapshot in range(9):
                    prior = old_clock[index * 792 + snapshot * 88 + 16:index * 792 + snapshot * 88 + 20]
                    current = native[index * 5452 + 448 + snapshot * 556 + 16:index * 5452 + 448 + snapshot * 556 + 20]
                    assert prior == current, (index, snapshot)
            assert native[:10904] == sequence
            for index, expected in retained.items():
                assert native[index * 5452:(index + 1) * 5452] == expected
            projection_path = output / "native-projection4896.bin"
            projection_path.write_bytes(projection)
            save(output / "inspection.json", {"counts": counts, "details": details})
            save(output / "report.json", {"schema": "sampled-channel-corpus-run-v3",
                 "native_executed": True, "records": 300, "snapshots": 2700,
                 "retained_smoke_histories_reproduced": 25, "protocol_rejections": len(protocols),
                 "framing_checks": 5, "counts": counts, "endpoint_branch_covered": False,
                 "retained_clock_frames_reproduced": 2700,
                 "runner_independently_reviewed": False, "rust_accepted": False,
                 "carrier_sha256": sha(carrier), "raw_output_sha256": sha(response_path),
                 "native_projection_sha256": sha(projection_path), "retries": 0})
    except BaseException as error:
        save(output / "failure.json", {"error": repr(error), "retained": True, "retries": 0})
        raise
    print("Native corpus completed; endpoint branch and Rust comparison remain open")


if __name__ == "__main__":
    main()
