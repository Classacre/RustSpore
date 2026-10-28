"""Compare explicit-package Rust lighting output with the independent audit.

The audit is produced by cell_world_lighting_candidate_v1.py. This checks
saved value bits only; it does not select the active package or prove rendering.
"""
import argparse
import json
import re
import struct
from pathlib import Path


def bits(value):
    return f"{struct.unpack('<I', struct.pack('<f', value))[0]:08X}"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("audit", type=Path)
    parser.add_argument("package_name")
    parser.add_argument("rust_log", type=Path)
    args = parser.parse_args()

    audit = json.loads(args.audit.read_text(encoding="utf-8"))
    records = [r for r in audit["records"] if r["package"] == args.package_name]
    if len(records) != 1:
        raise ValueError("expected exactly one matching package audit")
    fields = {f["identifier"]: f for f in records[0]["fields"]}
    expected = [[bits(x) for x in row] for row in fields["0100EAC5"]["raw_le_components"]]
    exposure = fields["0100EAC4"]["bits"][0]
    lines = args.rust_log.read_text(encoding="utf-8").splitlines()
    headers = [line for line in lines if " exposure_bits=" in line]
    rows = [re.fullmatch(r"(\d\d): ([0-9A-F]{8}) ([0-9A-F]{8}) ([0-9A-F]{8})", line) for line in lines]
    rows = [m for m in rows if m]
    if len(headers) != 1 or len(rows) != 25 or len(expected) != 25:
        raise ValueError("unexpected output shape")
    if f"exposure_bits={exposure}" not in headers[0]:
        raise ValueError("exposure mismatch")
    for index, row in enumerate(rows):
        if int(row.group(1)) != index or list(row.groups()[1:]) != expected[index]:
            raise ValueError(f"coefficient {index} mismatch")
    print(f"{args.package_name}: 75 RGB components and exposure match exactly")


if __name__ == "__main__":
    main()
