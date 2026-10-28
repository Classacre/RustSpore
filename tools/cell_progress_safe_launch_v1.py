"""Launch the reviewed progress runner with inherited Windows error dialogs suppressed."""
import ctypes
import runpy
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RUNNER = ROOT / "tools/cell_progress_native_run_v3.py"


def main():
    if sys.platform != "win32":
        raise SystemExit("This runner requires Windows")
    # The mode is local to this process and inherited by its child processes.
    # The runner still uses CREATE_NO_WINDOW for each child.
    ctypes.windll.kernel32.SetErrorMode(0x0001 | 0x0002)
    sys.argv = [str(RUNNER), *sys.argv[1:]]
    runpy.run_path(str(RUNNER), run_name="__main__")


if __name__ == "__main__":
    main()
