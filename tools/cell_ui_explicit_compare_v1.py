"""Compare explicit Rust Cell UI intake with the retained Python inventory."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--patch", type=Path, required=True)
    parser.add_argument("--graphics", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    for path in (args.reference, args.patch, args.graphics, args.out):
        path.resolve().relative_to((ROOT / "target").resolve())
    reference = json.loads(args.reference.read_text())
    result = {}
    for name, path, suffix in (("patch", args.patch, "PatchData.package"),
                               ("graphics", args.graphics, "Spore_Graphics.package")):
        candidates = [r for r in reference["records"] if r["package"].endswith(suffix)]
        assert len(candidates) == 1
        record = candidates[0]
        controls, images = [], []
        for line in path.read_text().splitlines():
            fields = line.split("\t")
            if fields[0] == "control":
                assert len(fields) == 4
                controls.append((int(fields[1], 16), fields[2], fields[3]))
            elif fields[0] == "image":
                assert len(fields) == 7
                images.append((int(fields[1], 16), int(fields[2])))
            else:
                assert line.startswith("#")
        expected_controls = [(r["object_index"], r["class"], r["id"]) for r in record["caller_control_id_matches"]]
        expected_images = [(r["object_index"], r["resource_index"]) for r in record["image_objects"]]
        assert sorted(controls) == sorted(expected_controls)
        assert sorted(images) == sorted(expected_images)
        result[name] = {"objects": record["objects"], "controls": len(controls), "images": len(images),
                        "rust_output_sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
    report = {"schema": "cell-ui-explicit-source-comparison-v1", "copies": result,
              "scope": "Explicit source copies; no active overlay, layout rendering, event or package precedence claim."}
    with args.out.open("x", encoding="utf-8", newline="\n") as handle:
        json.dump(report, handle, indent=2)
        handle.write("\n")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
