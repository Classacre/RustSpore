"""Recheck the reviewed, already-built CID comparison executable and inputs.

The retained cases use source body geometry and synthetic stage values. This
recheck does not launch the original executable or establish live animation.
"""
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PLAN = ROOT / "target/animation/cid-child-mode2-rust-comparison-plan-v1.json"
REVIEW = ROOT / "target/animation/cid-child-mode2-rust-optimized-executable-independent-v1.json"
EXE = ROOT / "target/animation/cid-child-mode2-rust-build-v1/release/examples/anim_cid_child_mode2_compare_v1.exe"
EXPECTED_EXE = "6450cd0a642f3fcbac6f48759d6f9692c4d2986c14e21e5b88d8c701df678e6c"
OUT = ROOT / "target/animation/cid-child-mode2-rust-pinned-compare-v1"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    plan = json.loads(PLAN.read_text(encoding="utf-8"))
    pins = {Path(path): value for path, value in plan["pins"].items()}
    pins.update({PLAN: sha(PLAN), REVIEW: sha(REVIEW), EXE: EXPECTED_EXE})
    for path, expected in pins.items():
        if sha(path) != expected:
            raise ValueError(f"input changed before run: {path}")
    OUT.mkdir(parents=True, exist_ok=False)
    output = OUT / "rust-output.bin"
    input_path = ROOT / "target/animation/cid-child-mode2-rust-inputs-v1/input.bin"
    native_path = ROOT / "target/animation/cid-child-mode2-native-execution-v3/native-output.bin"
    completed = subprocess.run(
        [str(EXE), str(input_path), str(native_path), str(output)],
        cwd=ROOT, capture_output=True, timeout=60, check=False,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
    )
    (OUT / "stdout.bin").write_bytes(completed.stdout)
    (OUT / "stderr.bin").write_bytes(completed.stderr)
    if completed.returncode != 0 or not output.is_file():
        raise RuntimeError(f"comparison failed with exit {completed.returncode}")
    for path, expected in pins.items():
        if sha(path) != expected:
            raise ValueError(f"input changed during run: {path}")
    if output.read_bytes() != native_path.read_bytes():
        raise ValueError("complete Rust and retained native records differ")
    result = {
        "schema": "cid-child-mode2-rust-pinned-compare-v1",
        "cases": 64,
        "record_bytes": 60,
        "synthetic_stage": True,
        "actual_actor_association": False,
        "binary_sha256": sha(EXE),
        "output_sha256": sha(output),
        "pins_unchanged": len(pins),
        "exit_code": completed.returncode,
    }
    (OUT / "report.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
