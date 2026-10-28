"""Rebuild and compare the Cell endpoint sampler with source/data pinned throughout.

This runs the Rust verifier in a fresh Cargo target directory. It does not run
the original native carrier; the comparison uses retained native output.
"""
import argparse
import hashlib
import json
import os
import subprocess
from pathlib import Path

from cell_runtime_data_native_run import pinned

ROOT = Path(__file__).resolve().parent.parent
CELL = ROOT / "target/source-cell"
NAME = "cell_runtime_fresh_sampled_channel_endpoint_verify_v1"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    output = args.out.resolve()
    output.relative_to(CELL.resolve())
    corpus = CELL / "runtime-fresh-sampled-channel-endpoint-run-v3"
    inputs = CELL / "runtime-fresh-sampled-channel-endpoint-prep-v2/inputs"
    constants = CELL / "native-runtime-fresh-sampled-channel-source-v1/78534400-7248-constant.bin"
    native = corpus / "corpus-output.bin"
    expected = corpus / "native-projection4896.bin"
    files = [Path(__file__), ROOT / "tools/cell_runtime_data_native_run.py",
             ROOT / "tools/build.bat", ROOT / "Cargo.toml", ROOT / "Cargo.lock",
             ROOT / "rust-toolchain.toml", ROOT / "crates/spore-core/Cargo.toml",
             ROOT / "crates/spore-core/examples" / (NAME + ".rs"),
             inputs / "input.bin", inputs / "cases.tsv", inputs / "cases.json",
             inputs.parent / "preparation.json", constants, native, expected, corpus / "report.json",
             Path("D:/Games/SPORE Collection/SporeBin/SporeApp.exe"),
             Path("D:/Games/SPORE Collection/SporeBin/msvcr90.dll")]
    files.extend((ROOT / "crates/spore-core/src").rglob("*.rs"))
    files.extend(path for path in (inputs / "clips").iterdir() if path.is_file())
    files = list(dict.fromkeys(path.resolve() for path in files))
    assert sha(expected) == "41ee7f78b69a7552f55f7fda570b3b02629228956c9444c5a20347050c384c4e"
    assert sha(files[-1]) and all(path.is_file() for path in files)
    output.mkdir(parents=True, exist_ok=False)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(output / "cargo")
    creation = getattr(subprocess, "CREATE_NO_WINDOW", 0)
    try:
        with pinned(files, output):
            with (output / "build.log").open("xb") as log:
                built = subprocess.run(["cmd.exe", "/d", "/c", str(ROOT / "tools/build.bat"),
                    "build", "-p", "spore-core", "--example", NAME], cwd=ROOT, env=env,
                    stdout=log, stderr=subprocess.STDOUT, creationflags=creation, check=False)
            assert built.returncode == 0, f"fresh build failed: {built.returncode}"
            exe = output / "cargo/debug/examples" / (NAME + ".exe")
            assert exe.is_file()
            rust = output / "rust.bin"
            with (output / "run.log").open("xb") as log:
                ran = subprocess.run([str(exe), str(inputs), str(native), str(constants), str(rust)],
                    cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT,
                    creationflags=creation, check=False)
            assert ran.returncode == 0, f"Rust comparison failed: {ran.returncode}"
            assert rust.read_bytes() == expected.read_bytes()
            marker = "cases=12 snapshots=108 projection_bytes=4896 incoming_states=3 positive_typed_guards=108"
            assert marker in (output / "run.log").read_text(errors="replace")
            result = {"schema": "cell-sampler-endpoint-pinned-clean-build-v1", "matched": True,
                      "fresh_target_directory": True, "records": 12, "snapshots": 108,
                      "pinned_file_count": len(files), "rust_projection_sha256": sha(rust),
                      "rust_executable_sha256": sha(exe), "build_log_sha256": sha(output / "build.log"),
                      "run_log_sha256": sha(output / "run.log"), "endpoint_branch_covered": True,
                      "new_native_execution": False}
            with (output / "result.json").open("x", encoding="utf-8", newline="\n") as handle:
                json.dump(result, handle, indent=2)
                handle.write("\n")
    except BaseException as error:
        (output / "failure.json").write_text(json.dumps({"error": repr(error)}, indent=2) + "\n")
        raise
    print(json.dumps(result))


if __name__ == "__main__":
    main()
