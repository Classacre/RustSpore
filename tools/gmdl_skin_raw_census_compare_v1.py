#!/usr/bin/env python3
"""Compare Rust verifier text against the independent, local JSON census.

Example: python tools/gmdl_skin_raw_census_compare_v1.py --gmdl target/source-cell/jawhead.gmdl --rust-output target/source-cell/raw-influences.txt
The Rust text is captured separately from the example's stdout. The decoded
payload and census stay local; neither is embedded in this script.
"""
import argparse
import hashlib
import json
from pathlib import Path


def compare(gmdl, rust_output, census):
    source = json.loads(census.read_text(encoding="utf-8"))
    payload = gmdl.read_bytes()
    digest = hashlib.sha256(payload).hexdigest()
    if len(payload) != source["payload_size"] or digest != source["payload_sha256"]:
        raise ValueError("decoded GMDL size or SHA-256 does not match the census")
    lines = iter(rust_output.read_text(encoding="utf-8").splitlines())

    def next_fields():
        try:
            return next(lines).split()
        except StopIteration as exc:
            raise ValueError("truncated Rust output") from exc

    meshes = source["meshes"]
    if next_fields() != ["GMDL_RAW_INFLUENCES_V1", str(len(meshes))]:
        raise ValueError("invalid header or mesh count")
    total = 0
    for mesh in meshes:
        mesh_id, count = mesh["mesh"], mesh["vertex_count"]
        expected_header = ["M", str(mesh_id), str(mesh["vertex_buffer"]), str(count)]
        if next_fields() != expected_header:
            raise ValueError(f"mesh {mesh_id}: header mismatch")
        raw = mesh["per_vertex_raw_indices_and_weights"]
        if len(raw) != count:
            raise ValueError(f"mesh {mesh_id}: census count mismatch")
        for vertex, (indices, weights) in enumerate(raw):
            expected = ["V", str(vertex), *(str(x) for x in indices + weights)]
            if next_fields() != expected:
                raise ValueError(f"mesh {mesh_id}, vertex {vertex}: raw bytes mismatch")
            total += 1
    if next(lines, None) is not None:
        raise ValueError("unexpected trailing Rust output")
    return {"status": "pass", "payload_sha256": digest, "mesh_count": len(meshes),
            "vertex_rows_compared": total, "census": str(census), "rust_output": str(rust_output)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gmdl", type=Path, required=True)
    parser.add_argument("--rust-output", type=Path, required=True)
    parser.add_argument("--census", type=Path, default=Path("target/source-cell/jawhead-skin-census-v1.json"))
    parser.add_argument("--report", type=Path, default=Path("target/source-cell/gmdl-skin-raw-census-comparison-v1.json"))
    args = parser.parse_args()
    target = Path("target").resolve()
    if not args.report.resolve().is_relative_to(target):
        parser.error("report must be inside the ignored target directory")
    try:
        report = compare(args.gmdl, args.rust_output, args.census)
    except (OSError, ValueError, KeyError, TypeError) as exc:
        report = {"status": "fail", "error": str(exc)}
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
