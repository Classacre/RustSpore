"""Compare retained installed-texture digest runs; this does not prove live file stability."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "target/texture-digest-comparison-20261109-v1.json"
FILES = {
    "raster_python": ROOT / "target/raster-python-20261109-v1.txt",
    "raster_rust": ROOT / "target/raster-rust-20261109-v2.txt",
    "rw4_python": ROOT / "target/rw4-texture-python-20261109-v1.txt",
    "rw4_rust": ROOT / "target/rw4-texture-rust-20261109-v2.txt",
    "rust_binary": ROOT / "target/rustspore-raster-digest-20261109-v2.exe",
    "rust_decoder": ROOT / "crates/spore-core/src/raster.rs",
    "raster_reference": ROOT / "tools/raster_extract.py",
    "raster_digest": ROOT / "tools/raster_digest.py",
    "rw4_reference": ROOT / "tools/rw4_texture_digest.py",
    "comparison_script": Path(__file__).resolve(),
    "content_package": Path(r"D:\Games\SPORE Collection\Data\Spore_Content.package"),
    "graphics_package": Path(r"D:\Games\SPORE Collection\Data\Spore_Graphics.package"),
}


def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main():
    if OUTPUT.exists():
        raise FileExistsError(OUTPUT)
    before = {name: sha(path) for name, path in FILES.items()}
    report = {
        "status": "failed",
        "inputs": {name: {"path": str(FILES[name]), "sha256": value} for name, value in before.items()},
        "limits": "Equality compares per-resource FNV-1a RGBA digests, not decoded byte streams or hardware output. Package hashes were captured after scan start; stable paths and unchanged package reads are admitted, not proven. Endpoint hashes do not prove continuous stability or build provenance.",
    }
    try:
        expected_inputs = {
            "rust_binary": "9588eb2b0e65fe7e26396126a4638493920da08a51b0a81697fd33d065a37739",
            "content_package": "0b849bd79d9c13dec5737159655bf7e2b7763cc2b79949c81370fa0490256e8c",
            "graphics_package": "e01bb86d8ddf02c56d52394b4e1db7181c1e1ebe1878bf6fb72c4ee523fcedd1",
        }
        if any(before[name] != expected for name, expected in expected_inputs.items()):
            raise ValueError("pinned binary or package digest differs")
        report["comparisons"] = {}
        for name, count in (("raster", 2954), ("rw4", 8915)):
            left = FILES[f"{name}_python"].read_text(encoding="utf-8").splitlines()
            right = FILES[f"{name}_rust"].read_text(encoding="utf-8").splitlines()
            mismatches = [i + 1 for i, (a, b) in enumerate(zip(left, right)) if a != b]
            report["comparisons"][name] = {
                "python_rows": len(left), "rust_rows": len(right),
                "expected_rows": count, "mismatch_count": len(mismatches),
                "first_mismatches": mismatches[:10],
            }
            if len(left) != count or len(right) != count or mismatches:
                raise ValueError(f"{name} counts or digest rows differ")
        report["status"] = "matched"
    except Exception as exc:
        report["error"] = repr(exc)
    finally:
        report["inputs_after"] = {name: sha(path) for name, path in FILES.items()}
        if report["inputs_after"] != before:
            report["status"] = "failed"
            report["post_error"] = "an input changed while this comparison ran"
        with OUTPUT.open("x", encoding="utf-8") as stream:
            json.dump(report, stream, indent=2)
            stream.write("\n")
    print(json.dumps(report, indent=2))
    if report["status"] != "matched":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
