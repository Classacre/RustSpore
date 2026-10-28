"""Clean-build the bounded CID node verifier with source and inputs pinned."""
import argparse
import hashlib
import json
import os
import subprocess
from pathlib import Path

from cell_runtime_data_native_run import pinned

ROOT = Path(__file__).resolve().parent.parent
ANIM = ROOT / "target/animation"
NAME = "anim_cid_node_init_verify_v1"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    output = args.out.resolve()
    output.relative_to(ANIM.resolve())
    inputs = ANIM / "cid-node-init-actual-body-inputs-v1"
    native_dir = ANIM / "cid-node-init-native-run-v2"
    native, expected = native_dir / "corpus-output.bin", native_dir / "native-projection.bin"
    report = json.loads((native_dir / "report.json").read_text())
    assert report["native_executed"] and report["cases"] == 1322 and report["nodes"] == 46256
    assert sha(native) == report["raw_sha256"] and sha(expected) == report["projection_sha256"]
    assert sha(expected) == sha(inputs / "expected.bin")
    files = [Path(__file__), ROOT / "tools/cell_runtime_data_native_run.py", ROOT / "tools/build.bat",
             ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "rust-toolchain.toml",
             ROOT / "crates/spore-core/Cargo.toml", ROOT / "crates/spore-core/src/anim_cid_node_init.rs.pending",
             ROOT / "crates/spore-core/examples" / (NAME + ".rs"),
             native, expected, native_dir / "report.json", native_dir / "immutability-v1.json",
             Path("D:/Games/SPORE Collection/SporeBin/SporeApp.exe")]
    files.extend((ROOT / "crates/spore-core/src").rglob("*.rs"))
    files.extend(path for path in inputs.iterdir() if path.is_file())
    files = list(dict.fromkeys(path.resolve() for path in files))
    assert all(path.is_file() for path in files)
    output.mkdir(parents=True, exist_ok=False)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(output / "cargo")
    hidden = getattr(subprocess, "CREATE_NO_WINDOW", 0)
    try:
        with pinned(files, output):
            with (output / "build.log").open("xb") as log:
                built = subprocess.run(["cmd.exe", "/d", "/c", str(ROOT / "tools/build.bat"),
                    "build", "-p", "spore-core", "--example", NAME], cwd=ROOT, env=env,
                    stdout=log, stderr=subprocess.STDOUT, creationflags=hidden, check=False)
            assert built.returncode == 0, f"fresh build failed: {built.returncode}"
            exe = output / "cargo/debug/examples" / (NAME + ".exe")
            rust = output / "rust-projection.bin"
            with (output / "run.log").open("xb") as log:
                ran = subprocess.run([str(exe), str(inputs), str(native), str(rust)], cwd=ROOT, env=env,
                    stdout=log, stderr=subprocess.STDOUT, creationflags=hidden, check=False)
            assert ran.returncode == 0, f"Rust comparison failed: {ran.returncode}"
            assert rust.read_bytes() == expected.read_bytes()
            marker = "CID_NODE_INIT_VERIFY cases=1322 nodes=46256 typed_guards=7932 incoming_CSR_states=3 full_node_bytes=700"
            assert marker in (output / "run.log").read_text(errors="replace")
            result = {"schema": "cid-node-init-pinned-rust-v1", "matched": True,
                      "fresh_target_directory": True, "cases": 1322, "nodes": 46256,
                      "typed_guard_checks": 7932, "incoming_sse_states": 3,
                      "projection_sha256": sha(rust), "executable_sha256": sha(exe),
                      "pinned_file_count": len(files), "new_native_execution": False,
                      "full_function_or_actor_accepted": False}
            with (output / "result.json").open("x", encoding="utf-8", newline="\n") as handle:
                json.dump(result, handle, indent=2)
                handle.write("\n")
    except BaseException as error:
        (output / "failure.json").write_text(json.dumps({"error": repr(error)}, indent=2) + "\n")
        raise
    print(json.dumps(result))


if __name__ == "__main__":
    main()
