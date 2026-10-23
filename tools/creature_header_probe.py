"""Independent native CreatureData header/RLE census; outputs stay in target/.

Native 4C0BB0 selects a 128-byte header for versions 9/10. Native 4C0A20
establishes literal/escape/repeat behavior, including state across chunks.
This reader does not use the earlier 68-byte framing helper.
"""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct

from dbpf_extract import parse_index, payload


def header(data, start, length=128):
    at = start
    decoded = bytearray()
    previous = 0
    chunks = 0
    while len(decoded) < length:
        if at + 2 > len(data):
            raise ValueError("missing RLE chunk size")
        size = struct.unpack_from("<H", data, at)[0]
        at += 2
        if not 1 <= size <= 0x800 or at + size > len(data):
            raise ValueError("invalid/truncated RLE chunk")
        chunk = data[at:at + size]
        at += size
        chunks += 1
        index = 0
        while index < size and len(decoded) < length:
            byte = chunk[index]
            index += 1
            if byte != 0xFE:
                decoded.append(byte)
                previous = byte
            else:
                if index >= size:
                    raise ValueError("escape has no count in its own chunk")
                count = chunk[index]
                index += 1
                if count == 0:
                    decoded.append(0xFE)
                    previous = 0xFE
                else:
                    if count > length - len(decoded):
                        raise ValueError("repeat exceeds native output extent")
                    decoded.extend(bytes([previous]) * count)
    return at, bytes(decoded), data[start:at], chunks


def inspect(data, stats):
    if len(data) < 20 or struct.unpack_from("<I", data)[0] != 0xABB455B7:
        raise ValueError("invalid CreatureData header")
    version = struct.unpack_from("<I", data, 4)[0]
    if version not in (9, 10):
        raise ValueError("unsupported version")
    count = 3 if version == 9 else 4
    counts = struct.unpack_from("<" + "I" * count, data, 8)
    if any(value > maximum for value, maximum in zip(counts, (256, 4096, 1024))):
        raise ValueError("native source count guard exceeded")
    start = 8 + count * 4
    at, decoded, encoded, chunks = header(data, start)
    total = at + counts[0] * 140 + counts[1] * 5 + counts[2] * 12 + 4
    if version == 10:
        total += counts[3] * 56
    if total != len(data):
        raise ValueError("full native header does not retain exact array framing")
    stats["resources"] += 1
    stats[f"version_{version}"] += 1
    stats["decoded_bytes"] += len(decoded)
    stats["previously_discarded_bytes"] += len(decoded) - 68
    stats["nonzero_discarded_bytes"] += sum(byte != 0 for byte in decoded[68:])
    stats["resources_with_nonzero_discarded_tail"] += any(decoded[68:])
    stats["rle_chunks"] += chunks
    stats["resources_with_multiple_chunks"] += chunks > 1
    feed = bytearray(b"creature-header-v1\0")
    feed.extend(struct.pack("<2I", version, len(encoded)))
    feed.extend(encoded)
    feed.extend(struct.pack("<I", len(decoded)))
    feed.extend(decoded)
    feed.extend(struct.pack("<2I", at, total))
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
    args = parser.parse_args()
    stats = Counter()
    digests, failures, excluded = [], [], []
    packages = {}
    for package in sorted(args.root.rglob("*.package")):
        try:
            entries, _ = parse_index(package)
        except ValueError as exc:
            excluded.append({"package": str(package), "error": str(exc)})
            continue
        local = 0
        for entry in entries:
            if entry["type"] != 0x0F43029A:
                continue
            local += 1
            key = f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X}"
            try:
                digest = inspect(payload(package, entry), stats)
                digests.append(f"{package.name} {key} {digest:016X}")
            except Exception as exc:
                failures.append({"package": package.name, "key": key, "error": repr(exc)})
        if local:
            packages[package.name] = local
    normalized = "\n".join(sorted(digests)) + "\n"
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.digest_out.parent.mkdir(parents=True, exist_ok=True)
    args.digest_out.write_text(normalized, encoding="utf-8")
    report = {"native_header_bytes": 128, "statistics": stats, "packages": packages,
              "failures": failures, "excluded_packages": excluded,
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
