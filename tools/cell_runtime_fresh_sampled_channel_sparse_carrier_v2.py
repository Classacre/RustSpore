"""Audit-only local-symbol correction; carrier/source/owned bytes unchanged."""
import cell_runtime_fresh_sampled_channel_sparse_carrier as previous
import cell_runtime_fresh_sampled_channel_native_probe_v2 as probe
previous.mechanical.probe=probe
if __name__=='__main__':
 previous.mechanical.SOURCE_TOOL_SHA=probe.sha(probe.__file__)
 previous.mechanical.main()
