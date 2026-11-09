"""Reviewed-runner candidate. Do not run until independently reviewed.

Runs only the fixed, hash-pinned synthetic x86 host and writes a new receipt.
The receipt records observed output, not gameplay or full-path equivalence.
"""
import hashlib
import json
import os
import struct
import subprocess
import sys
from pathlib import Path

import pefile

RUNNER = Path(__file__).absolute()
ROOT = RUNNER.parents[1]
PREP = ROOT / "target/cell-impact-flag-host-prep-20261105"
SOURCE = Path(r"D:\Games\SPORE Collection\SporeBin\SporeApp.exe")
RECEIPT = ROOT / "target/cell-impact-flag-host-prep-20261105/hidden-run-receipt-v1.json"
PINS = {
    SOURCE: "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
    PREP / "host-x86.exe": "037b4a808b14e5cf55aa428850e194d8837e1790bd90b0db1638a195e15ddfb2",
    PREP / "host-x86.asm": "bcedcbd54f6230fedfc571d2cf728610c389bbb284da20dcfd414c50adb5bc64",
    PREP / "prepare.py": "bff6f61b58feaafcc753c8c346a3e4d559b885a6e02ea05565c2569cc08a2992",
}
FRAGMENT = bytes.fromhex("0f2f417076058d4a01eb0233c988481c")
DISTANCES = (0x40A00000, 0x40A00001, 0x409FFFFF, 1, 0x7FC00001, 0x7F800001)
LIMITS = (0x40A00000, 0x40A00000, 0x40A00000, 0, 0x40A00000, 0x40A00000)


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def no_reparse(path):
    for part in (path, *path.parents):
        attrs = part.lstat().st_file_attributes
        require(not (attrs & 0x400), f"reparse point rejected: {part}")


def verify_pins(runner_hash):
    paths = {**PINS, RUNNER: runner_hash}
    for path, expected in paths.items():
        no_reparse(path)
        require(hashlib.sha256(path.read_bytes()).hexdigest() == expected, f"pin mismatch: {path}")


def main():
    require(sys.platform == "win32", "Windows required")
    require(len(sys.argv) == 2 and len(sys.argv[1]) == 64 and all(c in "0123456789abcdef" for c in sys.argv[1]), "supply independently reviewed runner SHA-256")
    runner_hash = sys.argv[1]
    no_reparse(RECEIPT.parent)
    require(not RECEIPT.exists(), "receipt already exists; refusing overwrite")
    verify_pins(runner_hash)

    source = pefile.PE(str(SOURCE), fast_load=True)
    require(source.OPTIONAL_HEADER.ImageBase == 0x400000, "unexpected source image base")
    require(source.get_data(0xE67543 - 0x400000, len(FRAGMENT)) == FRAGMENT, "source fragment mismatch")
    host = pefile.PE(str(PREP / "host-x86.exe"))
    require(host.FILE_HEADER.Machine == 0x14C and host.OPTIONAL_HEADER.ImageBase == 0x400000, "host architecture or base mismatch")
    require(host.OPTIONAL_HEADER.AddressOfEntryPoint == 0x1000, "host entry mismatch")
    require(host.get_data(0x1103, 17) == FRAGMENT + b"\xc3", "host fragment or return mismatch")
    imports = {entry.dll.lower(): {item.name for item in entry.imports} for entry in host.DIRECTORY_ENTRY_IMPORT}
    require(imports == {b"kernel32.dll": {b"GetStdHandle", b"WriteFile", b"ExitProcess"}}, "unexpected imports")
    data = host.get_data(0x3000, 0x15A)
    require(len(data) == 0x15A, "host data truncated")
    require(struct.unpack_from("<6I", data, 0) == DISTANCES, "distance fixture mismatch")
    require(struct.unpack_from("<6I", data, 0x18) == LIMITS, "threshold fixture mismatch")
    require(struct.unpack_from("<H", data, 0x30)[0] == 0x027F and struct.unpack_from("<I", data, 0x32)[0] == 0x1F80, "floating-point policy mismatch")
    require(all(struct.unpack_from("<I", data, offset)[0] == 0xA5A5A5A5 for offset in (0x36, 0xAE, 0xB2, 0xD6)), "guard fixture mismatch")
    require(data[0x3A:0xAE] == bytes(0x74) and data[0xB6:0xD2] == bytes(0x1C), "synthetic region mismatch")
    require(struct.unpack_from("<I", data, 0xD2)[0] == 0xCCCCCCCC, "output sentinel mismatch")

    result = subprocess.run([str(PREP / "host-x86.exe")], capture_output=True, timeout=5, creationflags=subprocess.CREATE_NO_WINDOW, check=False)
    require(result.returncode == 0 and len(result.stdout) == 120 and not result.stderr, "host failed or output malformed")
    rows = list(struct.iter_unpack("<IIIII", result.stdout))
    require(tuple(r[0] for r in rows) == (0xCCCCCC00, 0xCCCCCC01, 0xCCCCCC00, 0xCCCCCC01, 0xCCCCCC00, 0xCCCCCC00), "unexpected result words")
    require(tuple(r[4] for r in rows) == LIMITS, "observed thresholds differ")
    require(all(r[1] == 0x0000027F for r in rows), "unexpected x87 control or status")
    require(tuple(r[2] for r in rows) == (0x1F80, 0x1F80, 0x1F80, 0x1F82, 0x1F81, 0x1F81), "unexpected MXCSR")
    require(all((r[3] & 0x400) == 0 for r in rows), "direction flag set after guard checks")
    verify_pins(runner_hash)
    receipt = {"pins": {str(k): v for k, v in PINS.items()}, "runner_sha256": runner_hash, "fragment_hex": FRAGMENT.hex(), "distances": [f"{v:08X}" for v in DISTANCES], "rows_result_cw_sw_mxcsr_post_guard_eflags_threshold": [[f"{v:08X}" for v in row] for row in rows], "exit_code": result.returncode, "stdout_sha256": hashlib.sha256(result.stdout).hexdigest(), "limits": "Synthetic fixed inputs only; EFLAGS including DF are observed after guard checks, not at fragment return; no live contact or full-path claim. Hash checks do not prevent transient replacement between reads."}
    encoded = (json.dumps(receipt, indent=2) + "\n").encode("utf-8")
    with RECEIPT.open("xb") as out:
        out.write(encoded)
        out.flush()
        os.fsync(out.fileno())
    require(RECEIPT.read_bytes() == encoded, "receipt readback mismatch")
    print(f"Wrote {RECEIPT}")


if __name__ == "__main__":
    main()
