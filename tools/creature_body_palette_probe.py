"""Check a bounded creature/body-palette relation in existing source reports.

The 62xx-to-71xx namespace substitution is an explicit candidate join, not a
recovered runtime resource-resolution rule. It must never select a skeleton or
drive runtime indices. All source-bearing reports stay under ignored target/.
"""

import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--creatures", type=Path, required=True)
    parser.add_argument("--models", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    source_bytes = args.creatures.read_bytes()
    model_bytes = args.models.read_bytes()
    source_report = json.loads(source_bytes)
    model_report = json.loads(model_bytes)
    if source_report["failures"] or model_report["errors"] or model_report["index_errors"]:
        raise ValueError("input reports contain source failures")
    by_instance = defaultdict(list)
    native_first_group = Counter()
    for model in model_report["records"]:
        by_instance[int(model["key"].split(":")[-1], 16)].append(model)
        tail = model.get("tail")
        if tail is None:
            native_first_group["core_only_models"] += 1
        elif not tail["parts"]:
            native_first_group["empty_part_models"] += 1
        elif tail["parts"][0]["model_type"] == 0xFFFFFFFF:
            count = tail["parts"][0]["field"] & 0xFF
            native_first_group["first_part_absent_resource"] += 1
            native_first_group["first_absent_count_uses_sign_bit"] += count >= 128
            native_first_group["max_first_absent_count"] = max(
                native_first_group["max_first_absent_count"], count)
        else:
            native_first_group["first_part_external_resource"] += 1
    rows, unmatched, stats = [], [], Counter()
    for creature in source_report["creatures"]:
        parts = creature["parts"]
        classes = [p["fields_u16"][5] >> 8 for p in parts]
        non1 = [i for i, cls in enumerate(classes) if cls != 1]
        is_prefix = non1 == list(range(len(non1)))
        stats["source_creatures"] += 1
        stats["source_nonclass1_is_prefix"] += is_prefix
        instance = int(creature["key"].split(":")[-1], 16)
        candidate = (instance & ~(0xFF00 << 32)) | (0x7100 << 32)
        matches = [m for m in by_instance[candidate] if m.get("tail") is not None]
        if not matches:
            unmatched.append(creature["key"])
            continue
        stats["creatures_with_candidate_model"] += 1
        for model in matches:
            absent_parts = [p for p in model["tail"]["parts"] if p["model_type"] == 0xFFFFFFFF]
            count = sum(p["field"] & 0xFF for p in absent_parts)
            first_part = model["tail"]["parts"][0] if model["tail"]["parts"] else None
            first_absent = first_part is not None and first_part["model_type"] == 0xFFFFFFFF
            first_count = (first_part["field"] & 0xFF) if first_absent else None
            # 742D90 creates this absent-resource group; 73B7C0 clamps the
            # supplied count to its signed-byte count before 740530 gathers it.
            generated_prefix = (min(len(parts), first_count)
                                if first_count is not None and first_count < 128 else None)
            agrees = count == len(non1)
            stats["candidate_joins"] += 1
            stats["body_count_matches_nonclass1"] += agrees
            stats["body_count_matches_all_rows"] += count == len(parts)
            stats["matching_body_count_and_prefix"] += agrees and is_prefix
            stats["candidate_first_part_absent_resource"] += first_absent
            stats["native_clamped_prefix_matches_nonclass1"] += generated_prefix == len(non1)
            rows.append({"creature": creature["key"], "creature_package": creature["package"],
                         "model": model["key"], "model_package": model["package"],
                         "source_rows": len(parts), "classes": dict(Counter(classes)),
                         "body_count": count, "nonclass1_rows": len(non1),
                         "first_part_absent_resource": first_absent,
                         "first_group_count": first_count,
                         "native_generated_prefix_count": generated_prefix,
                         "nonclass1_is_prefix": is_prefix, "count_agrees": agrees})
    report = {
        "scope": "candidate 62xx-to-71xx namespace join; not native runtime mapping proof",
        "inputs": {"creatures_sha256": hashlib.sha256(source_bytes).hexdigest(),
                   "models_sha256": hashlib.sha256(model_bytes).hexdigest()},
        "statistics": stats, "candidate_joins": rows,
        "native_first_group_source_constraints": native_first_group,
        "unmatched_source_creatures": unmatched,
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps({"scope": report["scope"], "statistics": stats,
                      "native_first_group_source_constraints": native_first_group,
                      "unmatched_source_creatures": len(unmatched)}, indent=2))


if __name__ == "__main__":
    main()
