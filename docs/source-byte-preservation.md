# Source bytes used by reconstruction evidence

Reconstruction manifests bind source files by SHA256. Git checkout filters must
preserve those bytes, including existing LF, CRLF and mixed endings. The rules
in `.gitattributes` preserve each measured representation. Do not run a blanket
line-ending or whitespace formatter over frozen tools or models.

Git's whitespace policy recognizes CR as an end-of-line character. Other
trailing spaces and blank final lines remain checked, with the narrow exceptions
below for retained evidence sources. No implementation bytes were changed
to make those checks pass.

| Frozen file under `tools/` | Preserved bytes | SHA256 |
|---|---|---|
| `anim_selector_z_bounds_native_probe_v2.py` | Terminal blank line | `d4a316a1250f97f2bbb7a8e554deb9d34566354fe72169a672d31ce90e12dbb8` |
| `anim_ungrouped_binding_builder_audit_v1.py` | Terminal blank line | `f1eba517a054966c30c3184441227dcf94dd0341f5b91f15b645291c1287f6be` |
| `anim_ungrouped_binding_builder_native_run_v1.py` | Terminal blank line | `3d9d57c4ad329a26e795bb7cc3ea6a0d1ba0a90de873dbb86dcd03df0c6df38e` |
| `anim_ungrouped_binding_builder_native_run_v2.py` | Terminal blank line | `5e7f758de3eec797e30fd7bd2008a8f14e66182c594b14a8c0b5d194ab407500` |
| `anim_ungrouped_binding_builder_prepare_v1.py` | Terminal blank line | `7cd00a63f17a8e99da310c1012e28b4d1298508ed3085b9eb5f9a264a821d07f` |
| `cell_runtime_source_clock_root_data_fixture.py` | One whitespace-only line | `bbc2f9d3d92032307eed221c7f95bd2987eea7f527b1e49c94e40b39add8cc67` |

The 2026-10-28 continuation adds these exact whitespace exceptions, preserving
the source representations used or retained during the recorded comparisons:

| File | Preserved bytes | SHA256 |
|---|---|---|
| `re/cell-ui-registry-audit-v1.md` | Terminal blank line | `12cb6a5cdf1d3152155270e0788c0a49a70727bf3f4e55a7e3283a612f6317c7` |
| `tools/cell_runtime_fresh_sampled_channel_endpoint_runner_v3.py` | Trailing space on one line | `1ffed1086cbf210d14dc7baa582eba89727ec1702707049c55be0245561c0106` |
| `tools/creature_health_lookup_semantics_v1.py` | Terminal blank line | `c1cb567348a4506f8216d3680ddf2d02d2f46eb8d618d6f31cdffcd1a575eaa4` |

`tools/source_checkout_hash_audit_v2.py` checks every publishable file without
staging or modifying sources. `tools/staged_source_checkout_audit.py` checks
the complete Git index and its checkout representation, while excluding local
untracked drafts from the publication assertion. Both write new reports only
under ignored `target/`.
