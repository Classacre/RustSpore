"""Pinned formatting-only probe wrapper for unchanged hybrid linked auditor."""
import hashlib
from pathlib import Path
import cell_runtime_fresh_loop_hybrid_carrier_audit_v1 as base
import cell_runtime_fresh_loop_native_probe_v2 as source
assert hashlib.sha256(Path(base.__file__).read_bytes()).hexdigest()=='f10cd18c3d62436cb46136af4f26cb6a8b770db21752dba6cd9166f53efa12ef'
assert hashlib.sha256(Path(source.__file__).read_bytes()).hexdigest()=='a42ceb8ab65aa89e70f68917a0492c2767f8f5ac5bdf76ef43c60917d18f9b98'
base.source=source
if __name__=='__main__':base.main()
