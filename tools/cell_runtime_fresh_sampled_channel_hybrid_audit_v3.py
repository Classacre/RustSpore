"""Full hybrid audit of event-tail shadow successor; never execute."""
import cell_runtime_fresh_sampled_channel_hybrid_audit as previous
import cell_runtime_fresh_sampled_channel_native_probe_v3 as source
previous.source=source
if __name__=='__main__':previous.main()
