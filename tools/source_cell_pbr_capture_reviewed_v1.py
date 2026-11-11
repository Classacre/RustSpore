"""Review-gated, private-desktop Jawhead PBR inspection capture; never retail parity.

No builds or pin generation occur here. Supply a separately reviewed JSON file with
the runner's SHA-256 and exact SHA-256 pins for every path in REQUIRED. The review
must also contain approved=true and a nonempty reviewer field. The coordinator must
pass the review file's independently verified SHA-256 as --review-sha256.
Hashes before and after do not prove stable-file admission during execution.
"""
import argparse
import ctypes
from ctypes import wintypes
import hashlib
import json
from pathlib import Path
import platform
import re
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[1]
INSTALL = Path(r"D:\Games\SPORE Collection")
EXE = ROOT / "target/debug/rustspore.exe"
REQUIRED = {
    "exe": EXE,
    "source_cell": ROOT / "crates/spore-bevy/src/source_cell.rs",
    "assets": ROOT / "crates/spore-bevy/src/assets.rs",
    "offscreen": ROOT / "crates/spore-bevy/src/offscreen.rs",
    "raster": ROOT / "crates/spore-core/src/raster.rs",
    "cargo_lock": ROOT / "Cargo.lock",
    **{name: INSTALL / "Data" / (name + ".package") for name in
       ("PatchData", "Spore_Game", "Spore_Content", "Spore_Graphics")},
}


def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--review", type=Path, required=True)
    p.add_argument("--review-sha256", required=True)
    p.add_argument("--output", type=Path, required=True, help="fresh directory under target/source-cell")
    p.add_argument("--timeout", type=int, default=120, choices=range(10, 301))
    a = p.parse_args()
    if sys.platform != "win32":
        raise RuntimeError("Windows required")
    if not re.fullmatch(r"[0-9a-f]{64}", a.review_sha256) or sha(a.review) != a.review_sha256:
        raise ValueError("review hash mismatch")
    review = json.loads(a.review.read_text(encoding="utf-8"))
    if review.get("approved") is not True or not isinstance(review.get("reviewer"), str) or not review["reviewer"].strip():
        raise ValueError("independent approval and reviewer required")
    if review.get("runner_sha256") != sha(Path(__file__)):
        raise ValueError("reviewed runner hash mismatch")
    pins = review.get("pins")
    if not isinstance(pins, dict) or set(pins) != set(REQUIRED):
        raise ValueError("review must pin exactly the required inputs")
    base = (ROOT / "target/source-cell").resolve()
    output = a.output.resolve()
    if output == base or not output.is_relative_to(base):
        raise ValueError("output must be a fresh child of target/source-cell")
    output.mkdir(parents=False, exist_ok=False)
    receipt = {"status": "failed", "review_sha256": a.review_sha256,
               "reviewer": review["reviewer"], "platform": platform.platform(),
               "limitations": "PBR inspection, not retail appearance. Hashes do not establish stable-file admission or executable build provenance."}
    try:
        before = {name: sha(path) for name, path in REQUIRED.items()}
        receipt["inputs"] = {name: {"path": str(REQUIRED[name]), "sha256": value} for name, value in before.items()}
        if before != pins:
            raise ValueError("input pin mismatch")
        user32 = ctypes.WinDLL("user32", use_last_error=True)
        user32.CreateDesktopW.argtypes = [wintypes.LPCWSTR, wintypes.LPCWSTR, ctypes.c_void_p, wintypes.DWORD, wintypes.DWORD, ctypes.c_void_p]
        user32.CreateDesktopW.restype = wintypes.HANDLE
        user32.CloseDesktop.argtypes = [wintypes.HANDLE]
        user32.CloseDesktop.restype = wintypes.BOOL
        desktop_name = "RustSporePbr-" + uuid.uuid4().hex
        desktop = user32.CreateDesktopW(desktop_name, None, None, 0, 0x00c3, None)
        if not desktop:
            raise OSError(ctypes.get_last_error(), "CreateDesktopW failed")
        receipt["desktop"] = {"name": desktop_name, "switched": False}
        stdout = stderr = b""
        try:
            startup = subprocess.STARTUPINFO()
            startup.lpDesktop = desktop_name
            startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            startup.wShowWindow = 0
            cmd = [str(EXE), "--source-cell", "--install", str(INSTALL), "--cell", "7D00E384", "--pbr-inspection", "--shot", str(output / "jawhead.png")]
            receipt["command"] = cmd
            try:
                result = subprocess.run(cmd, cwd=ROOT, capture_output=True, timeout=a.timeout,
                                        creationflags=subprocess.CREATE_NO_WINDOW, startupinfo=startup)
                stdout, stderr, receipt["exit"] = result.stdout, result.stderr, result.returncode
            except subprocess.TimeoutExpired as exc:
                stdout, stderr, receipt["exit"] = exc.stdout or b"", exc.stderr or b"", "timeout"
        finally:
            receipt["desktop"]["closed"] = bool(user32.CloseDesktop(desktop))
            (output / "stdout.bin").write_bytes(stdout)
            (output / "stderr.bin").write_bytes(stderr)
        after = {name: sha(path) for name, path in REQUIRED.items()}
        receipt["post_hashes"] = after
        log = stdout.decode("utf-8", errors="replace")
        receipt["adapter_lines"] = [line for line in log.splitlines() if "SOURCE_CELL_GPU " in line]
        receipt["mip_lines"] = [line for line in log.splitlines() if "SOURCE_CELL_PREVIEW_MIPS " in line]
        image = output / "jawhead.png"
        if image.exists():
            receipt["image_sha256"] = sha(image)
        if (receipt["exit"] != 0 or not receipt["desktop"]["closed"] or before != after
                or not receipt["adapter_lines"] or not receipt["mip_lines"] or not image.is_file()):
            raise RuntimeError("capture acceptance failed; inspect logs and receipt")
        receipt["status"] = "observed"
    except Exception as exc:
        receipt["error"] = repr(exc)
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(receipt, indent=2))
    return 0 if receipt["status"] == "observed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
