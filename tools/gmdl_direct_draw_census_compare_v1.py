#!/usr/bin/env python3
"""Compare conditional direct-draw output to the independent local census.

A passing comparison is an asset compatibility check, not a live draw, pose,
GPU upload, or proof that the admitted path ran for this particular actor.
"""
import argparse
import hashlib
import json
from pathlib import Path


def compare(gmdl, rust_output, census):
    source = json.loads(census.read_text(encoding="utf-8-sig"))
    payload = gmdl.read_bytes()
    digest = hashlib.sha256(payload).hexdigest()
    if len(payload) != source["payload_size"] or digest != source["payload_sha256"]:
        raise ValueError("decoded GMDL size or SHA-256 does not match census")
    lines = iter(rust_output.read_text(encoding="utf-8-sig").splitlines())

    def fields():
        try:
            return next(lines).split()
        except StopIteration as exc:
            raise ValueError("truncated Rust output") from exc

    meshes, descriptors = source["meshes"], source["palette_descriptors"]
    if len(meshes) != 2 or len(descriptors) != 2:
        raise ValueError("expected both Jawhead meshes and descriptors")
    if fields() != ["GMDL_DIRECT_DRAW_INFLUENCES_V1", str(len(meshes))]:
        raise ValueError("invalid header or mesh count")
    total = 0
    active_lanes = 0
    counts = []
    for ordinal, mesh in enumerate(meshes):
        if mesh["mesh"] != ordinal:
            raise ValueError("census mesh order differs from source ordinal")
        desc = descriptors[ordinal]
        vertices = mesh["triangle_source"]["referenced_vertices"]
        if vertices != sorted(set(vertices)) or len(vertices) != mesh["triangle_source"]["unique_referenced_vertices"]:
            raise ValueError(f"mesh {ordinal}: invalid referenced vertex census")
        expected_header = ["M", str(ordinal), str(desc["first_word"]), str(desc["count"]),
                           str(desc["first_word"] * 48), str(len(vertices))]
        if fields() != expected_header:
            raise ValueError(f"mesh {ordinal}: descriptor, byte offset, or count mismatch")
        raw = mesh["per_vertex_raw_indices_and_weights"]
        if len(raw) != mesh["vertex_count"]:
            raise ValueError(f"mesh {ordinal}: census vertex count mismatch")
        for vertex in vertices:
            if not isinstance(vertex, int) or not 0 <= vertex < len(raw):
                raise ValueError(f"mesh {ordinal}: referenced vertex outside census")
            offsets, weights = raw[vertex]
            if len(offsets) != 4 or len(weights) != 4:
                raise ValueError(f"mesh {ordinal}, vertex {vertex}: malformed census row")
            expected = ["V", str(vertex), *(str(x) for x in offsets + weights)]
            if fields() != expected:
                raise ValueError(f"mesh {ordinal}, vertex {vertex}: raw bytes mismatch")
            for offset, weight in zip(offsets, weights):
                if weight:
                    active_lanes += 1
                    if offset % 3 or offset // 3 >= desc["count"]:
                        raise ValueError(f"mesh {ordinal}, vertex {vertex}: incompatible active offset")
            total += 1
        counts.append(len(vertices))
    if next(lines, None) is not None:
        raise ValueError("unexpected trailing Rust output")
    return {"status": "pass", "payload_sha256": digest, "mesh_count": len(meshes),
            "referenced_rows_compared": total, "mesh_referenced_counts": counts,
            "active_lanes_checked": active_lanes, "census": str(census), "rust_output": str(rust_output),
            "scope": "conditional direct path compatibility; no live draw or pose established"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gmdl", type=Path, required=True)
    parser.add_argument("--rust-output", type=Path, required=True)
    parser.add_argument("--census", type=Path, default=Path("target/source-cell/jawhead-skin-census-v1.json"))
    parser.add_argument("--report", type=Path, default=Path("target/source-cell/gmdl-direct-draw-census-comparison-v1.json"))
    args = parser.parse_args()
    if not args.report.resolve().is_relative_to(Path("target").resolve()):
        parser.error("report must be inside the ignored target directory")
    try:
        report = compare(args.gmdl, args.rust_output, args.census)
    except (OSError, ValueError, KeyError, TypeError, IndexError) as exc:
        report = {"status": "fail", "error": str(exc)}
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
