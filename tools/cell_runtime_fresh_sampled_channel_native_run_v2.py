"""Single reviewed successor cohort after source-proved owned event-tail fix.

The immutable v1 execution/inspection implementation is reused exactly. This
wrapper changes only the reviewed plan/artifact identities and linked auditor;
it does not retry the failed v1attempt or alter source-event/numeric oracles.
"""
import json,hashlib
from pathlib import Path
import cell_runtime_fresh_sampled_channel_native_run_v1 as frozen
import cell_runtime_fresh_sampled_channel_hybrid_audit_v3 as linked
ROOT=frozen.ROOT
PLAN=ROOT/'target/source-cell/runtime-fresh-sampled-channel-runner-plan-v2/plan.json'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 assert sha(frozen.__file__)=='e3168458dcc657882987fbcfe7c2632779f99a85a58b31460562149cf968a923'
 plan=json.loads(PLAN.read_text());assert plan['source_proved_owned_event_tail_successor']is True
 assert sha(__file__)==plan['runner_sha256']
 host=Path(plan['host']);asm=(host/'host-x86.asm').read_text();assert asm.count(linked.source.SHADOW)==3
 images=linked.source.arithmetic.Images(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll'));assert asm==linked.source.host_source(images)
 frozen.PLAN=PLAN;frozen.CARRIER=plan['carrier_sha256'];frozen.OWNED=plan['owned_sha256'];frozen.MAP=plan['map_sha256'];frozen.linked=linked
 # v1's exact-runner review check must identify this executing wrapper; v1's
 # immutable source remains separately pinned in plan and independent review.
 frozen.__file__=__file__
 frozen.main()
if __name__=='__main__':main()
