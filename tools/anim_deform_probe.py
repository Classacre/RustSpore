"""Independent native ANIM descriptor-binding measurements.

Reads the user's packages without modifying them. Source data and reports belong
under ignored target/. This models descriptor preparation, not authored curves,
generated body construction or mesh deformation. The caller explicitly selects
the empty initial fallback established by the native animation initializer.
"""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct

from dbpf_extract import parse_index, payload
from part_capabilities_probe import inspect as inspect_dictionary


def select_layout(selector, layouts):
    flags, capability, alternate, _reserved = selector
    mode = flags & 0x100003
    if mode == 1:
        capability = int.from_bytes(b"root", "little")
    elif mode != 0:
        return None
    if flags & 0x801C00 in (0x1800, 0x1C00, 0x800400):
        capability = alternate
    return capability if capability and capability in layouts else None


def prepare(data, layouts, stats):
    if data[:4] != b"ANIM" or struct.unpack_from("<I", data, 4)[0] != len(data):
        raise ValueError("invalid ANIM header")
    count, table = struct.unpack_from("<2I", data, 324)
    if table + count * 4 > len(data):
        raise ValueError("channel table overrun")
    # Native template is copied once per clip. Only active IDs are replaced in
    # each channel; a stale match outside the current layout is unsupported.
    template = [int.from_bytes(tag, "little") for tag in (b"INFO", b"POS\0", b"ROT\0")]
    template += [0] * 6
    feed = bytearray(b"native-anim-descriptors-v1\0")
    feed.extend(struct.pack("<I", count))
    for index in range(count):
        start = struct.unpack_from("<I", data, table + index * 4)[0]
        if data[start:start + 4] != b"CHAN":
            raise ValueError("missing CHAN")
        selector = struct.unpack_from("<4I", data, start + 140)
        selected = select_layout(selector, layouts)
        layout = layouts.get(selected, [])
        if len(layout) > 6:
            raise ValueError("more than six deform slots")
        for slot, entry in enumerate(layout, 3):
            template[slot] = entry[0]
        n, descriptors = struct.unpack_from("<2I", data, start + 220)
        if descriptors + n * 32 > len(data):
            raise ValueError("descriptor table overrun")
        feed.extend(struct.pack("<4I", *selector))
        feed.extend(struct.pack("<2I", int(selected is not None), selected or 0))
        feed.extend(struct.pack("<I", len(layout)))
        for entry in layout:
            feed.extend(struct.pack("<4I", *entry))
        feed.extend(struct.pack("<I", n))
        stats["channels"] += 1
        stats["fallback_channels"] += selected is None
        for ordinal in range(n):
            words = list(struct.unpack_from("<8I", data, descriptors + ordinal * 32))
            source_flags, tag, _offset, _stride, source_slot, *_reserved = words
            matches = [slot for slot, candidate in enumerate(template)
                       if min(slot, 3) == (source_flags & 15) and candidate
                       and (tag == candidate or tag == 0xFFFFFFFF)]
            if not matches:
                words[0] &= ~0x10
                words[4] = 0xFFFFFFFF
                stats["unmatched_components"] += 1
            else:
                slot = matches[0]
                words[4] = slot
                stats["matched_components"] += 1
                if slot >= 3:
                    if slot - 3 >= len(layout):
                        raise ValueError(f"channel {index} component {ordinal} matches stale slot {slot}")
                    params = layout[slot - 3][3]
                    words[0] = (words[0] & ~0xC0) | ((params & 1) << 7) | ((params & 2) << 5)
                    stats["deform_components"] += 1
                else:
                    stats["transform_info_components"] += 1
            stats["components"] += 1
            stats["flags_changed"] += words[0] != source_flags
            stats["slots_changed"] += words[4] != source_slot
            stats["enabled_components"] += bool(words[0] & 0x10)
            feed.extend(struct.pack("<8I", *words))
    digest = 0xCBF29CE484222325
    for byte in feed:
        digest = ((digest ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return digest


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("root", nargs="?", type=Path, default=Path("D:/Games/SPORE Collection"))
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--digest-out", required=True, type=Path)
    parser.add_argument("--rust-digest", type=Path)
    parser.add_argument("--native-initial", "--default-empty", dest="native_initial", action="store_true",
                        help="select the measured native initial empty fallback")
    args = parser.parse_args()
    if not args.native_initial:
        parser.error("--native-initial is required for this initial fallback context")
    package = args.root / "Data/PatchData.package"
    entries, _ = parse_index(package)
    dictionary_entries = [e for e in entries if e["type"] == 0x7C19AA7A
                          and e["group"] == 0x10000 and e["instance"] == 0x82D0142D3338]
    if len(dictionary_entries) != 1:
        raise ValueError("explicit patched dictionary is not unique")
    source = payload(package, dictionary_entries[0])
    dictionary = inspect_dictionary(source)
    layouts = {}
    for group in dictionary["deforms"]:
        capability = int(group["capability"], 16)
        if capability in layouts:
            raise ValueError("duplicate deformation capability")
        layouts[capability] = [[int(c["id"], 16), *[int(p, 16) for p in c["raw_parameters"]]]
                               for c in group["channels"]]
    digests, failures, excluded = [], [], []
    stats = Counter()
    packages = {}
    for package in sorted(args.root.rglob("*.package")):
        try:
            entries, _ = parse_index(package)
        except ValueError as exc:
            excluded.append({"package": str(package), "error": str(exc)})
            continue
        local = 0
        for entry in entries:
            if entry["type"] != 0xEE17C6AD:
                continue
            local += 1
            key = f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X}"
            try:
                digest = prepare(payload(package, entry), layouts, stats)
                digests.append(f"{package.name} {key} {digest:016X}")
            except Exception as exc:
                failures.append({"package": package.name, "key": key, "error": repr(exc)})
        if local:
            packages[package.name] = local
    normalized = "\n".join(sorted(digests)) + "\n"
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.digest_out.parent.mkdir(parents=True, exist_ok=True)
    args.digest_out.write_text(normalized, encoding="utf-8")
    report = {"context": "patched dictionary with measured native initial empty fallback",
              "dictionary_sha256": hashlib.sha256(source).hexdigest(),
              "resources": len(digests), "packages": packages,
              "statistics": stats, "failures": failures, "excluded_packages": excluded,
              "digest_file_sha256": hashlib.sha256(normalized.encode()).hexdigest()}
    if args.rust_digest:
        rust = sorted(args.rust_digest.read_text(encoding="utf-8").splitlines())
        report["rust_resources"] = len(rust)
        report["rust_digest_matches"] = rust == sorted(digests)
    args.out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps({k: v for k, v in report.items() if k != "excluded_packages"}, indent=2))
    if failures or not digests or report.get("rust_digest_matches") is False:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
