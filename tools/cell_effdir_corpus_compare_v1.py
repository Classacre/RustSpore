"""Compare Rust EFFDIR framing with the independent installed-data audit.

Decoded game payloads and reports are written only under an absent target
directory. This compares explicit files, not runtime overlay selection.
"""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path

from cell_skybox_effdir_audit_v1 import PACKAGES, TYPE, audit
from dbpf_extract import payload
from gmdl_skin_probe import indexes


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("install", type=Path)
    p.add_argument("executable", type=Path)
    p.add_argument("--out", required=True, type=Path)
    args = p.parse_args()
    target = (Path(__file__).resolve().parent.parent / "target").resolve()
    args.out.resolve().relative_to(target)
    args.out.mkdir(parents=True, exist_ok=False)
    executable = args.executable.resolve()
    exe_hash = sha(executable.read_bytes())
    rows = []
    for package_name in PACKAGES:
        package = args.install / "Data" / package_name
        for entry in indexes(package):
            if entry["type"] != TYPE:
                continue
            data = payload(package, entry)
            if len(data) != entry["dec"]:
                raise ValueError("decoded length mismatch")
            reference = audit(data)
            if reference["string_assignments_word"] != "FFFFFFFF":
                raise ValueError("unsupported assignment framing")
            expected = []
            for match in reference["matches"]:
                if match["group"] != "00000000":
                    continue
                for block in match.get("visual", {}).get("blocks", []):
                    screen = block.get("screen")
                    if screen is not None:
                        expected.append(
                            f"visual_block={block['index']} mode={screen['mode']} "
                            f"flags={screen['flags']} texture={screen['texture_group']}:"
                            f"{screen['texture_instance']}"
                        )
            path = args.out / f"{len(rows):03}.bin"
            path.write_bytes(data)
            result = subprocess.run(
                [str(executable), str(path)], capture_output=True, text=True,
                timeout=60, check=False,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            )
            if result.returncode != 0 or result.stdout.splitlines() != expected:
                raise ValueError(f"Rust/reference mismatch at {package_name} {entry['instance']:016X}: {result.stderr}")
            if sha(path.read_bytes()) != sha(data) or sha(executable.read_bytes()) != exe_hash:
                raise ValueError("input or executable changed during comparison")
            rows.append({"package": package_name, "group": f"{entry['group']:08X}",
                         "instance": f"{entry['instance']:016X}", "sha256": sha(data),
                         "visuals": reference["visual_count"], "exports": reference["export_count"],
                         "matching_screen_records": len(expected)})
    report = {"schema": "cell-effdir-corpus-comparison-v1", "executable_sha256": exe_hash,
              "directories": len(rows), "rows": rows,
              "scope": "Framing and selected screen outputs only; other sections opaque, no runtime selection."}
    (args.out / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"{len(rows)} directories match; {sum(r['matching_screen_records'] for r in rows)} selected screens")


if __name__ == "__main__":
    main()
