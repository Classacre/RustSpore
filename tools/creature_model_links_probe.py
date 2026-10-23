"""Audit direct named model-LOD keys at actual source creature PROP instances.

The source instance comes from a retained CreatureData inventory and is checked
against all current package indexes. No namespace substitution selects a model.
Reports and source-bearing digests belong under ignored target/.
"""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
import hashlib
import itertools
import json
from pathlib import Path
import struct

from anim_rig_mapping_probe import key_values, properties
from dbpf_extract import payload
from prop_numeric_probe import fields, index
from refpack_bulk import install as bulk_backreferences

CREATURE, PROP, GMDL = 0x0F43029A, 0x00B1B104, 0x00E6BCE5
PARENT = 0x00B2CCCB
MODEL_SLOTS = (0x00F9EFBB, 0x00F9EFBC, 0x00F9EFBD, 0x00F9EFBE)
SELECTED = {PARENT, *MODEL_SLOTS}


def resource_key(entry: dict) -> str:
    return f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X}"


def selected_keys(data: bytes) -> tuple[int, list[dict], int]:
    # Certify the entire declared named list before using the independent key
    # walk. Anonymous recovery and a shorter count are not accepted here.
    fields(data, set())
    records = properties(data)
    declared, = struct.unpack_from(">I", data)
    if len(records) != declared or any(ident is None for ident, *_ in records):
        raise ValueError("key walk does not match the complete declared named list")
    selected = []
    canonical = bytearray(b"creature-model-links-v1\0")
    canonical.extend(struct.pack("<3I", len(data), declared,
                                 sum(ident in SELECTED for ident, *_ in records)))
    for ident, typ, spec, raw in records:
        if ident not in SELECTED:
            continue
        keys = key_values(typ, spec, raw)
        canonical.extend(struct.pack("<4I", ident, typ, spec, len(keys)))
        values = []
        for instance, kind in keys:
            canonical.extend(struct.pack("<3I", instance & 0xFFFFFFFF, kind, instance >> 32))
            values.append({"instance": f"{instance:016X}", "type": f"{kind:08X}"})
        selected.append({"property": f"{ident:08X}", "type": f"{typ:04X}",
                         "specifier": f"{spec:04X}", "keys": values})
    value = 0xCBF29CE484222325
    for byte in canonical:
        value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return declared, selected, value


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("install", type=Path)
    parser.add_argument("--creatures", type=Path, required=True)
    parser.add_argument("--prefix-inventory", type=Path, required=True)
    parser.add_argument("--digest-out", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--rust-digest", type=Path)
    args = parser.parse_args()
    bulk_backreferences()
    source_bytes = args.creatures.read_bytes()
    source = json.loads(source_bytes)
    prefix_bytes = args.prefix_inventory.read_bytes()
    prefix = json.loads(prefix_bytes)
    creatures = source["creatures"]
    wanted = {int(row["key"].split(":")[-1], 16) for row in creatures}
    expected_sources = {(row["package"], row["key"]) for row in creatures}
    counts, current_sources = Counter(), set()
    resources = defaultdict(list)
    model_props = []
    index_errors = []
    for package in sorted(args.install.rglob("*.package")):
        try:
            entries = index(package)
        except Exception as error:
            index_errors.append({"package": str(package), "error": str(error)})
            continue
        counts["indexes_opened"] += 1
        relative = package.relative_to(args.install).as_posix()
        for entry in entries:
            if entry["type"] == CREATURE:
                current_sources.add((package.name, resource_key(entry)))
            if entry["type"] in {GMDL, 0x2F4E681B, 0x01C135DA}:
                resources[(entry["type"], entry["instance"])].append({"package": relative, "key": resource_key(entry)})
            if entry["type"] == PROP and entry["instance"] in wanted:
                model_props.append((package, relative, entry))
    source_mismatch = sorted(current_sources ^ expected_sources)
    counts["source_creatures"] = len(creatures)
    counts["source_instances"] = len(wanted)
    counts["indexed_source_identities"] = len(current_sources)
    records, failures, digest_rows = [], [], []
    by_instance = defaultdict(list)
    body_counts = {(Path(row["model_package"]).name, row["model"]): row["first_group_count"]
                   for row in prefix["candidate_joins"] if row["first_part_absent_resource"]}
    for package, relative, entry in model_props:
        try:
            data = payload(package, entry)
            declared, selected, value = selected_keys(data)
        except Exception as error:
            failures.append({"package": relative, "key": resource_key(entry), "error": str(error)})
            continue
        digest_rows.append(f"{relative}\t{resource_key(entry)}\t{value:016X}")
        record = {"package": relative, "key": resource_key(entry), "source_sha256": hashlib.sha256(data).hexdigest(),
                  "property_count": declared, "selected": selected, "models": []}
        counts["exact_named_model_prop_copies"] += 1
        for item in selected:
            if int(item["property"], 16) not in MODEL_SLOTS:
                continue
            for key in item["keys"]:
                kind, instance = int(key["type"], 16), int(key["instance"], 16)
                available = resources[(kind, instance)]
                model = {"slot": item["property"], **key, "available": available}
                record["models"].append(model)
                counts["direct_model_references"] += 1
                counts["available_model_references"] += bool(available)
                if item["property"] == "00F9EFBB":
                    counts["direct_lod0_references"] += 1
                    counts["direct_lod0_gmdl_available_copies"] += kind == GMDL and bool(available)
        records.append(record)
        by_instance[entry["instance"]].append(record)
    associations = []
    for creature in creatures:
        instance = int(creature["key"].split(":")[-1], 16)
        props = by_instance[instance]
        counts["source_creatures_with_direct_model_prop"] += bool(props)
        lod0_refs = [model for prop in props for model in prop["models"] if model["slot"] == "00F9EFBB"]
        counts["source_creatures_with_direct_lod0"] += bool(lod0_refs)
        counts["source_creatures_with_available_direct_lod0_gmdl"] += any(
            model["type"] == f"{GMDL:08X}" and model["available"] for model in lod0_refs)
        if len({(model["type"], model["instance"]) for model in lod0_refs}) > 1:
            counts["source_creatures_with_conflicting_lod0_copies"] += 1
        nonclass1 = sum(part["fields_u16"][5] >> 8 != 1 for part in creature["parts"])
        bounded = []
        for model in lod0_refs:
            for available in model["available"]:
                first = body_counts.get((Path(available["package"]).name, available["key"]))
                if first is not None:
                    bounded.append({"model": available, "first_group_count": first,
                                    "native_generated_prefix_count": min(len(creature["parts"]), first)})
        counts["source_creatures_with_direct_known_body_count"] += bool(bounded)
        counts["direct_known_body_count_matches_nonclass1"] += bool(bounded) and all(
            value["native_generated_prefix_count"] == nonclass1 for value in bounded)
        associations.append({"creature_package": creature["package"], "creature": creature["key"],
                             "source_rows": len(creature["parts"]), "nonclass1_rows": nonclass1,
                             "model_props": [{"package": prop["package"], "key": prop["key"]} for prop in props],
                             "lod0": lod0_refs, "known_body_counts": bounded})
    digest_rows.sort()
    normalized = "\n".join(digest_rows) + "\n"
    args.digest_out.parent.mkdir(parents=True, exist_ok=True)
    args.digest_out.write_text(normalized, encoding="utf-8", newline="\n")
    comparison = None
    differences = []
    if args.rust_digest:
        rust = args.rust_digest.read_text(encoding="utf-8").splitlines()
        differences = [{"line": i + 1, "python": a, "rust": b}
                       for i, (a, b) in enumerate(itertools.zip_longest(digest_rows, rust)) if a != b]
        comparison = {"exact_match": not differences, "python_records": len(digest_rows),
                      "rust_records": len(rust), "rust_digest": str(args.rust_digest)}
    report = {"scope": "direct authored named PROP model references at actual CreatureData instances; no namespace substitution",
              "input_hashes": {"creatures": hashlib.sha256(source_bytes).hexdigest(),
                               "prefix_inventory": hashlib.sha256(prefix_bytes).hexdigest()},
              "counts": counts, "index_errors": index_errors, "source_inventory_mismatch": source_mismatch,
              "framing_failures": failures, "records": records, "associations": associations,
              "normalized_digest_sha256": hashlib.sha256(normalized.encode()).hexdigest(),
              "rust_comparison": comparison, "differences": differences[:20], "difference_count": len(differences)}
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"counts": counts, "source_inventory_mismatch": len(source_mismatch),
                      "framing_failures": len(failures), "difference_count": len(differences),
                      "normalized_digest_sha256": report["normalized_digest_sha256"]}))
    if source_mismatch or failures or differences or index_errors:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
