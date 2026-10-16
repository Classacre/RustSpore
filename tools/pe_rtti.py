"""
Recover MSVC RTTI class names from the SPORE executables.

Why this matters: MSVC emits a TypeDescriptor for every polymorphic class, and each
one contains the mangled class name as a plain NUL-terminated string in .rdata.
Reading them back gives the engine's internal class inventory, which is the single
most useful structural fact about a closed-source C++ engine. It is also the item
the Ghidra-based analysis could not produce (analysis never finished inside the
330 second deadline), so it is recovered here by direct byte scanning instead.

TypeDescriptor layout on x86 (MSVC):
    +0x00  void* pVFTable    pointer to type_info's vftable (a known .rdata address)
    +0x04  void* spare       usually 0
    +0x08  char  name[]      mangled name, NUL terminated

Mangled name prefixes:
    .?AV  class
    .?AU  struct
    .?AW  enum
Nesting uses '@' as the separator, outermost last, so 'B@A@@' is A::B.

Usage:
    python pe_rtti.py [--json out.json] [--dump out.txt]
"""

import json
import re
import struct
import sys
from collections import Counter
from pathlib import Path

TARGETS = [
    ("base", Path(r"D:\Games\SPORE Collection\SporeBin\SporeApp.exe")),
    ("ep1", Path(r"D:\Games\SPORE Collection\SporebinEP1\SporeApp.exe")),
]

# Section raw ranges, from the measured section tables (identical in both files
# except .rsrc and .reloc, which hold no code).
SECTIONS = {
    ".text": (0x00000400, 0x00FCB000),
    ".rdata": (0x00FCB400, 0x00140200),
    ".data": (0x0110B600, 0x000C4C00),
}

# .?AV = class, .?AU = struct, .?AW = enum
PATTERN = re.compile(rb"\.\?A[VUW]([ -~]{1,400}?)\x00")


def demangle(raw: str) -> tuple[str, str]:
    """Turn 'Foo@Bar@@' style nesting into 'Bar::Foo'. Returns (kind, name)."""
    kind = {"V": "class", "U": "struct", "W": "enum"}[raw[3]]
    body = raw[4:]
    # Strip the trailing @@ and any template suffix we cannot resolve cleanly.
    body = body.rstrip("@")
    # MSVC separates nested names with '@', outermost last.
    parts = [p for p in body.split("@") if p]
    parts.reverse()
    return kind, "::".join(parts)


def scan(path: Path):
    data = path.read_bytes()
    found = {}
    for name, (off, size) in SECTIONS.items():
        chunk = data[off:off + size]
        for m in PATTERN.finditer(chunk):
            mangled = m.group(0)[:-1].decode("ascii", "replace")
            kind, pretty = demangle(mangled)
            # Record the section so we can tell code-adjacent names from data.
            found.setdefault(mangled, (kind, pretty, name, off + m.start()))
    return found


def main() -> int:
    results = {}
    for label, path in TARGETS:
        if not path.exists():
            print(f"missing: {path}")
            continue
        found = scan(path)
        results[label] = found
        print(f"=== {label}: {path.name}")
        print(f"    distinct RTTI type names: {len(found)}")

    if not results:
        return 1

    # Union across binaries, tracking which files each name appears in.
    union: dict[str, dict] = {}
    for label, found in results.items():
        for mangled, (kind, pretty, section, offset) in found.items():
            entry = union.setdefault(
                mangled, {"kind": kind, "pretty": pretty, "in": [], "sections": set()}
            )
            entry["in"].append(label)
            entry["sections"].add(section)

    print()
    print(f"union across binaries: {len(union)} distinct names")
    for label, found in results.items():
        print(f"  {label}: {len(found)}")

    # Kind breakdown.
    kinds = Counter(e["kind"] for e in union.values())
    print()
    print("by kind:", dict(kinds))

    # Namespace histogram: the first path component is usually the subsystem.
    top = Counter()
    for e in union.values():
        root = e["pretty"].split("::")[0]
        # Strip template arguments so 'Foo<int>' and 'Foo<char>' group together.
        root = root.split("<")[0]
        if root:
            top[root] += 1

    print()
    print("top 60 root namespaces by class count:")
    for name, count in top.most_common(60):
        print(f"  {count:>5}  {name}")

    # Names that look like engine subsystems.
    interesting = [
        "Simulator", "Pollinator", "Swarm", "Graphics", "Resource", "App",
        "Editors", "Editor", "cSP", "Spore", "Sim", "Planet", "Creature",
        "Space", "Game", "Render", "Model", "Terrain", "Asset", "Database",
    ]
    print()
    print("root namespaces matching known subsystem words:")
    for word in interesting:
        hits = [(n, c) for n, c in top.items() if word.lower() in n.lower()]
        if hits:
            total = sum(c for _, c in hits)
            sample = ", ".join(f"{n}({c})" for n, c in sorted(hits, key=lambda x: -x[1])[:6])
            print(f"  {word:<12} {total:>5} classes across {len(hits)} roots: {sample}")

    # Emit artifacts.
    out_dir = Path(__file__).resolve().parent.parent / "re" / "data"
    out_dir.mkdir(parents=True, exist_ok=True)

    txt = out_dir / "rtti-types.txt"
    with txt.open("w", encoding="utf-8") as fh:
        fh.write("MSVC RTTI type names recovered from the SPORE executables.\n")
        fh.write("Generated by tools/pe_rtti.py. Grades: these are measured bytes.\n\n")
        fh.write(f"union distinct names: {len(union)}\n")
        for label, found in results.items():
            fh.write(f"  {label}: {len(found)}\n")
        fh.write(f"by kind: {dict(kinds)}\n\n")
        fh.write("=" * 78 + "\nROOT NAMESPACE HISTOGRAM\n" + "=" * 78 + "\n")
        for name, count in top.most_common():
            fh.write(f"{count:>6}  {name}\n")
        fh.write("\n" + "=" * 78 + "\nALL NAMES (mangled | demangled | kind | binaries)\n")
        fh.write("=" * 78 + "\n")
        for mangled, e in sorted(union.items(), key=lambda kv: kv[1]["pretty"]):
            fh.write(f"{mangled}\t{e['pretty']}\t{e['kind']}\t{','.join(e['in'])}\n")
    print()
    print(f"wrote {txt}")

    js = out_dir / "rtti-types.json"
    with js.open("w", encoding="utf-8") as fh:
        json.dump(
            {
                "union_count": len(union),
                "per_binary": {k: len(v) for k, v in results.items()},
                "by_kind": dict(kinds),
                "root_namespaces": dict(top.most_common()),
                "names": [
                    {"mangled": m, "demangled": e["pretty"], "kind": e["kind"],
                     "binaries": e["in"], "sections": sorted(e["sections"])}
                    for m, e in union.items()
                ],
            },
            fh,
            indent=1,
        )
    print(f"wrote {js}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
