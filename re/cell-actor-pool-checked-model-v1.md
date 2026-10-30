# Checked Cell actor-pool model

`spore_core::cell_actor_pool_checked_v1` is registered in `spore-core`. It is
source-guided pool groundwork, not an integrated Cell actor lifecycle. The
static instruction trace and its limits are in
`cell-actor-pool-handle-static-v1.md`; it refers to installed base
`SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.

The source allocates or reuses slots in `B71C50`, with LIFO free-list reuse and
the 32-bit handle formed from counter and index. `B71CC0` checks a handle for
lookup; `B71D50` releases by slot index; `B71C10` and `B71D20` scan occupied
slots in ascending order; and `B71C00` resets pool counts and free-list head
while retaining the counter. The static trace does not establish actor teardown,
mutation during iteration or all runtime caller preconditions.

The typed model requires an explicit capacity and counter seed. It adds
occupancy checks, strict `index < high_water` validation, stale and double-free
rejection, and a conservative error before encoded generation wraps through
zero. It excludes slot index `FFFF` and returns removed/reset values for
caller-owned teardown. These are defensive policies, not claims that the native
pool validates invalid handles or refuses rollover. Native `B71CC0` tests
`index > high_water`; the checked model deliberately does not reproduce the
resulting read at `index == high_water`. Raw handles do not establish cross-pool
identity, and the model does not establish safety across pool reinitialization.

## Validation

Seven focused checked-pool unit tests passed. Separately, the production module
was compared with a Python reference using a dictionary and free-index stack.
The retained synthetic corpus contains **131 cases and 8,220 sequential
operations**, with capacities 1–8. It covers allocation and full capacity,
LIFO reuse, release and stale/high-water/zero handles, ascending iteration,
reset and counter preservation, and rollover refusal. Every reply byte matched.

| Evidence | SHA-256 |
| --- | --- |
| `target/cell-actor-pool-model-v3/input.bin` | `3a53bd1920ad6700667dc7545c801481987acb472198d4ecad9957c568edad06` |
| `target/cell-actor-pool-model-v3/rust.bin` | `80c63c9b22a658c1a7805d2cd4aeb0cc1d80828a5924a6250928f4d3623314a1` |

The same directory retains `expected.bin` and `manifest.json`, which pins the
model, Python reference and Rust adapter. The Python verifier compares the
entire reply after checking those pins and corpus hashes; it does not
authenticate who produced an arbitrary supplied reply. This is a synthetic
model-to-model comparison, not execution of original instructions, native
pool acceptance, proof of runtime actor identity or actor/event integration.
Iteration during mutation, reentrancy, teardown and cross-pool identity remain
outside the admitted domain.

## Reproduction

From the repository root, use a fresh output directory:

```powershell
python -I -S tools/cell_actor_pool_reference_v1.py prepare target/cell-actor-pool-model-repro
tools\build.bat test -p spore-core cell_actor_pool_checked_v1::tests
tools\build.bat run -p spore-core --example cell_actor_pool_compare_v1 -- target/cell-actor-pool-model-repro/input.bin target/cell-actor-pool-model-repro/rust.bin
python -I -S tools/cell_actor_pool_reference_v1.py verify target/cell-actor-pool-model-repro target/cell-actor-pool-model-repro/rust.bin
```
