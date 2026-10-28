"""Pinned formatting-only probe wrapper for unchanged sparse carrier builder."""
import hashlib
from pathlib import Path
import cell_runtime_fresh_loop_sparse_carrier as base
import cell_runtime_fresh_loop_native_probe_v2 as probe
assert hashlib.sha256(Path(base.__file__).read_bytes()).hexdigest()=='20dad0c2846995ffc62ee102fccf26698be4641f11903f1aa2049917470d974e'
assert hashlib.sha256(Path(probe.__file__).read_bytes()).hexdigest()=='a42ceb8ab65aa89e70f68917a0492c2767f8f5ac5bdf76ef43c60917d18f9b98'
base.probe=probe;base.SOURCE_TOOL_SHA='a42ceb8ab65aa89e70f68917a0492c2767f8f5ac5bdf76ef43c60917d18f9b98'
if __name__=='__main__':base.main()
