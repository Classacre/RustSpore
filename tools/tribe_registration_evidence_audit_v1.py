"""Audit bounded Tribe-related registration evidence in retained REA dossiers.

This is a static evidence auditor, not an implementation of gameplay. It reads
local analysis output and the original executable, without modifying either.
The identifiers are observed arguments, not proven class layouts or behavior.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

EXE_SHA256 = "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a"
DEFAULT_ROOT = Path(__file__).resolve().parents[1]
RECORDS = {
    "cTribeFoodMat": ("0x629bafe", "0x629baec", "0xb251e0", "0x145fe7c"),
    "cTribe": ("&DAT_018c6d19", "0x4f396a66", "0xb25337", "0x145fe10"),
    "cTribeTool": ("&DAT_018c8f0c", "0x116d858", "0xb25368", "0x145fe04"),
    "cTribeHut": ("0x1e4daae", "&LAB_00ee02c7", "0xb258f5", "0x145fc24"),
    "cTribePlanner": ("0x3098af98", "0x9098aeb2", "0xb25926", "0x145fc14"),
    "cTribeToTribeMission": ("0x5436411a", "0x34364118", "0xb259b9", "0x145fbc8"),
}


def dossier(path: Path, address: str) -> dict:
    envelope = json.loads(path.read_text(encoding="utf-8"))
    evidence = json.loads(envelope["content"][0]["text"])
    if evidence["subject"]["digest"]["sha256"] != EXE_SHA256:
        raise ValueError(f"unexpected executable digest in {path}")
    if evidence["provider"]["id"] != "ghidra":
        raise ValueError(f"unexpected provider in {path}")
    result = evidence["normalized_result"]
    if result["procedure"]["address"] != address:
        raise ValueError(f"unexpected function in {path}")
    return result


def require(condition: bool, detail: str) -> None:
    if not condition:
        raise ValueError(detail)


def observed_ascii_lower_hash(data: bytes, seed: int = 0x811c9dc5) -> int:
    """Model q392 flag 1 only for ASCII NUL-free inputs and a 32-bit seed."""
    if not 0 <= seed <= 0xffffffff or any(byte == 0 or byte > 0x7f for byte in data):
        raise ValueError("requires a 32-bit seed and NUL-free ASCII")
    value = seed
    for byte in data:
        lower = byte + 32 if 65 <= byte <= 90 else byte
        value = ((value * 0x1000193) & 0xffffffff) ^ lower
    return value


def audit(root: Path, executable: Path) -> dict:
    h = hashlib.sha256()
    with executable.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    require(h.hexdigest() == EXE_SHA256, "executable digest mismatch")
    evidence_dir = root / "target" / "rea" / "20261028-source-runtime-v6"
    reg = dossier(evidence_dir / "119-analyze_function.json", "0xb245d0")
    init = dossier(evidence_dir / "122-analyze_function.json", "0xb25b60")
    observations = []
    for name, (arg1, arg2, site, string_addr) in RECORDS.items():
        expected = f'FUN_00b23cf0({arg1},{arg2},"{name}");'
        require(expected in reg["pseudocode"], f"missing call: {name}")
        require(any(s["address"] == string_addr and s["source_address"] == site and s["value"] == name
                    for s in reg["referenced_strings"]), f"missing string reference: {name}")
        require(any(a.lower() == f"{site}: push {string_addr}" for a in reg["assembly"]),
                f"missing string push: {name}")
        observations.append({"name": name, "call_argument_1": arg1, "call_argument_2": arg2,
                             "string_address": string_addr, "push_address": site})
    # The registration function writes these labels to a returned slot after
    # registering the named key. Ghidra does not recognize them as procedures.
    factory_leads = {"cTribeFoodMat": ("0x629bafe", "0xb1eee0", "LAB_00b1eee0"),
                     "cTribeTool": ("&DAT_018c8f0c", "0xb1f060", "LAB_00b1f060")}
    for name, (key, entry, label) in factory_leads.items():
        registration = f'"{name}");'
        begin = reg["pseudocode"].index(registration) + len(registration)
        tail = reg["pseudocode"][begin:begin + 240]
        require(key in tail and f"*puVar1 = &{label};" in tail,
                f"missing slot association: {name}")
    require('FUN_00b245d0();' in init["pseudocode"], "initializer does not call registry")
    require('FUN_00b23af0(0xef3da0d7,"cCreatureTribeMember");' in init["pseudocode"],
            "missing member registration")
    require(any(a.lower() == "0xb25bdf: push 0xef3da0d7" for a in init["assembly"]),
            "missing member argument push")
    ctor = dossier(evidence_dir / "379-analyze_function.json", "0xc9bc50")
    hash_fn = dossier(evidence_dir / "392-analyze_function.json", "0x932b30")
    require("param_2 = param_2 * 0x1000193 ^ uVar1;" in hash_fn["pseudocode"]
            and "uVar1 = tolower(uVar1);" in hash_fn["pseudocode"], "hash helper changed")
    hashes = {}
    for name, expected in (("trg_sacrifice_altar1", 0x1e42e0c6), ("TribalTools", 0xf37c24e6)):
        require(f'FUN_00932b30("{name}",0x811c9dc5,1)' in ctor["pseudocode"],
                f"missing constructor hash input: {name}")
        result = observed_ascii_lower_hash(name.encode("ascii"))
        require(result == expected, f"unexpected model output: {name}")
        hashes[name] = f"0x{result:08x}"
    return {"executable_sha256": EXE_SHA256, "registry_function": "0xb245d0",
            "initializer_function": "0xb25b60", "observed_registrations": observations,
            "member_registration": {"name": "cCreatureTribeMember", "argument": "0xef3da0d7"},
            "unresolved_slot_targets": {name: entry for name, (_, entry, _) in factory_leads.items()},
            "calculated_ascii_hashes": hashes,
            "limitations": ["Static registration evidence only; names do not prove runtime use or semantics.",
                "Arguments may be interpreted by Ghidra as addresses; their meanings remain unknown.",
                "No selection, orders, economy, diplomacy, actor layout or stage entry behavior is established.",
                "The initializer dossier has a terminal-call boundary limitation; later code is not assessed."]}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    parser.add_argument("--executable", type=Path, default=Path(r"D:\Games\SPORE Collection\SporeBin\SporeApp.exe"))
    args = parser.parse_args()
    print(json.dumps(audit(args.root, args.executable), indent=2))


if __name__ == "__main__":
    main()
