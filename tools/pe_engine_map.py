"""
Build a structured engine map from the string inventory.

The executables were built with RTTI disabled, so class names cannot be recovered
from TypeDescriptors. They are present anyway, as string literals: the game
registers its simulation classes, scripting functions and console commands by
name at startup, and those registrations are plain strings.

This script reads re/data/strings-classified.txt and pulls the registries out of
it into separate, readable files. Every entry is a measured byte sequence; the
grouping is by literal prefix, which is how the game itself namespaces them.

Usage:
    python pe_engine_map.py
"""

import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
STRINGS = ROOT / "re" / "data" / "strings-classified.txt"
OUT = ROOT / "re" / "data"

# Registries the game namespaces by literal prefix.
REGISTRIES = {
    "simulator_classes": re.compile(r"(?:SP_)?Simulator/([A-Za-z_][A-Za-z0-9_]*)"),
    "argscript_functions": re.compile(r"ArgScript/([A-Za-z_][A-Za-z0-9_]*)"),
    "eapd_modules": re.compile(r"EAPd/([A-Za-z_][A-Za-z0-9_]*)"),
    "renderware": re.compile(r"\b(RW[A-Za-z0-9_]+)"),
}

# Source paths leaked by assert messages, which name EA's shared libraries.
SOURCE_PATH = re.compile(r"[\w.\\/]*\.(?:cpp|cxx|h|hpp)\b")

# Console commands, which appear as "[verb ...]" or "verb <arg> -- description".
CONSOLE = re.compile(r"^\s{2,}([a-z][a-z_]{2,}(?:\s+[a-z_<>\-|]+)*)")


def main() -> int:
    if not STRINGS.exists():
        print(f"missing {STRINGS}; run tools/pe_strings.py first")
        return 1

    text = STRINGS.read_text(encoding="utf-8", errors="replace")
    lines = text.splitlines()

    found = {name: Counter() for name in REGISTRIES}
    source_paths = Counter()
    commands = Counter()

    for line in lines:
        stripped = line.strip()
        if not stripped or stripped.startswith("#") or stripped.startswith("====="):
            continue
        for name, pattern in REGISTRIES.items():
            for m in pattern.finditer(stripped):
                found[name][m.group(1)] += 1
        for m in SOURCE_PATH.finditer(stripped):
            source_paths[m.group(0)] += 1
        m = CONSOLE.match(line)
        if m:
            commands[m.group(1).strip()] += 1

    OUT.mkdir(parents=True, exist_ok=True)

    # 1. Simulation classes: the game's own object model.
    sims = sorted(found["simulator_classes"])
    p = OUT / "simulator-classes.txt"
    with p.open("w", encoding="utf-8") as fh:
        fh.write("Simulation class names registered by the SPORE executables.\n")
        fh.write("Source: string literals matching '(SP_)?Simulator/<Name>'.\n")
        fh.write(f"Distinct names: {len(sims)}\n\n")
        for name in sims:
            fh.write(f"{name}\n")
    print(f"{p.name}: {len(sims)} distinct class names")

    # 2. ArgScript function registry: the scripting API surface.
    args = sorted(found["argscript_functions"])
    p = OUT / "argscript-functions.txt"
    with p.open("w", encoding="utf-8") as fh:
        fh.write("ArgScript function and property names registered by SPORE.\n")
        fh.write("ArgScript is EA's scripting language; the one RTTI name that\n")
        fh.write("survived in the binaries is EA::ArgScript::cError, which\n")
        fh.write("independently confirms the namespace.\n")
        fh.write(f"Distinct names: {len(args)}\n\n")
        for name in args:
            fh.write(f"{name}\n")
    print(f"{p.name}: {len(args)} distinct names")

    # 3. EA shared-library source paths.
    paths = sorted(source_paths)
    p = OUT / "ea-source-paths.txt"
    with p.open("w", encoding="utf-8") as fh:
        fh.write("Source file paths leaked by assert messages in the SPORE executables.\n")
        fh.write("These name the libraries the game was linked against.\n")
        fh.write(f"Distinct paths: {len(paths)}\n\n")
        for path in paths:
            fh.write(f"{path}\n")
    print(f"{p.name}: {len(paths)} distinct source paths")

    # 4. Console commands.
    cmds = [c for c, _ in commands.most_common() if " " not in c or len(c) > 8]
    p = OUT / "console-commands.txt"
    with p.open("w", encoding="utf-8") as fh:
        fh.write("Console command tokens recovered from usage strings.\n")
        fh.write("Regex-based, so treat as indicative rather than complete.\n")
        fh.write(f"Candidates: {len(cmds)}\n\n")
        for c in sorted(set(cmds)):
            fh.write(f"{c}\n")
    print(f"{p.name}: {len(set(cmds))} candidates")

    # Console output for the operator.
    print()
    print("EAPd audio modules:", len(found["eapd_modules"]))
    for name, count in found["eapd_modules"].most_common(12):
        print(f"  {count:>3}  {name}")

    print()
    print("RenderWare symbols seen:", len(found["renderware"]))
    for name, count in found["renderware"].most_common(12):
        print(f"  {count:>3}  {name}")

    print()
    print("top EA shared-library source paths:")
    for path, count in source_paths.most_common(20):
        print(f"  {count:>3}  {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
