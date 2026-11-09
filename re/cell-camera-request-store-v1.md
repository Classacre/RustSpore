# Controlled request store to camera provider

The reviewed original E5CE46 gate/store slice and original E51760/B71D00
provider ran four synthetic cases: zero first triple word, mismatched type,
matching type with null factory return, and matching type with nonnull return.
E4EE10 was a guarded scripted callback. The fixture supplies one stable actor,
component and component pool, and stops before later request methods. The
original run matched, exited zero and closed its non-switched desktop after
reaping. Receipt `target/cell-camera-request-store-result-v1/receipt.json`
SHA-256 `8b6940a90ff69afa1461a7462f252f0cf3e4aee5eceb9ccab70661005d4ba756`.
It does not run allocation, the real factory, later callbacks or establish live
request ownership.

`cell_camera_request_store_v1` separately projects the gate/store and fresh
provider lookup under explicit same-component, nonwrapping pool-address and
lifetime admissions. Actor handle/+FC are post-factory snapshots; B71D00's
low-index-only behavior remains unchecked for generation. The comparator
checks all four retained factory/store/provider/FP rows. The next source
continuation remains pending and no gameplay identity follows.
