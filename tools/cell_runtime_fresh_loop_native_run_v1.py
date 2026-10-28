"""Frozen fullactivation hybrid runner: preflight default; --run one attempt only."""
import argparse,json,struct
from pathlib import Path
from types import SimpleNamespace
import cell_runtime_fresh_loop_hybrid_carrier_audit_v2 as linked
import cell_runtime_fresh_loop_inputs as fixture
from cell_runtime_data_native_run import digest,pinned,saved_file_run
ROOT=Path(__file__).resolve().parent.parent
HOST=ROOT/'target/source-cell/native-runtime-fresh-loop-hybrid-host-root-v2'
INPUT=ROOT/'target/source-cell/runtime-fresh-loop-inputs-v1'
CARRIER='400252d4bc1c85f6a6d334e6bfd8e2bed4d29f047581bf955bb84649e705f421'
OWNED='5a3e2504b23c80c39dc1bff39d9b772c7e2a6161a571f6d6eea32e8f2fa331c0'
HARD_PINS={
  "D:\\AI\\RustSpore\\crates\\spore-core\\examples\\cell_runtime_fresh_loop_verify.rs.pending": "aa1cdb29d6623d19632591f7c4d876076f7c56acaba0a3c4fec96ad6f8abc27d",
  "D:\\AI\\RustSpore\\crates\\spore-core\\src\\cell_runtime_fresh_loop.rs": "3609ec4699a6f76eaceb86c37b34d251cdfd71749e280657244fb85457f2c392",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-root-v1\\compile.log": "8e78a236646e15f946a74eac452026b11278efa7e4c1aa15825be3fce901902b",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-root-v1\\host-x86.asm": "feae82186ed9c308dbf5ecd534983aeae453573b23f01f655f4f42bd7d1d9cf2",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-root-v1\\manifest.json": "6b55c45ef71b07ae1f8ac4f81a904ba1f5d1850edd10683bb3c8e707ace78b4d",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-root-v2\\carrier-build-review.json": "1029c38a59fad5b4b741a35fc49d5c5d79b15fc35d32ed99776f10466f09125b",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-root-v2\\carrier-x86.exe": "400252d4bc1c85f6a6d334e6bfd8e2bed4d29f047581bf955bb84649e705f421",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-root-v2\\compile.log": "12df85f12a91afe4a8e6b6e953fe68740b8989444f2ea8398e4ff113fc1f0421",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-root-v2\\host-x86.asm": "cba277dff766d875a9d7e80bbc276ce034f1060ba3ddc2e09a162a40e75e7ee4",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-root-v2\\host-x86.exe": "5a3e2504b23c80c39dc1bff39d9b772c7e2a6161a571f6d6eea32e8f2fa331c0",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-root-v2\\host-x86.map": "39189d375935f1ebd694f3e36920557134043df7a00314e83ceb41ed2a9ee8f2",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-root-v2\\host-x86.obj": "5b58f6902796c0cb0f9227a1aefdb28822378942f8360fc00f600803185000c4",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-root-v2\\manifest.json": "e37b97ef3b56d5a0fdcf5b0556dd4a9595ef083f81bbe271ade3c42a79982e2e",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-root-v2\\root-linked-audit-v2.json": "d32b8c616990495d120de1e731d7af70943a5dee4af119e9584de48fd02d59a9",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-v3\\host-x86.asm": "feae82186ed9c308dbf5ecd534983aeae453573b23f01f655f4f42bd7d1d9cf2",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-v3\\manifest.json": "6b55c45ef71b07ae1f8ac4f81a904ba1f5d1850edd10683bb3c8e707ace78b4d",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-v4\\format-review.json": "f9e4c2aa52a56104a948ffdd4c93c0184797db8b44ca9b41143101c3e42bbbe7",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-v4\\host-x86.asm": "cba277dff766d875a9d7e80bbc276ce034f1060ba3ddc2e09a162a40e75e7ee4",
  "D:\\AI\\RustSpore\\target\\source-cell\\native-runtime-fresh-loop-hybrid-host-v4\\manifest.json": "e37b97ef3b56d5a0fdcf5b0556dd4a9595ef083f81bbe271ade3c42a79982e2e",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\capacity.bin": "5e2c75668fec68e8b99694e8c86467e77fd34a28f8f389adad0f8bd514eb0763",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\cases.json": "e799c5f99e3a2d1a989273566bf7177b1ce26309bc00451399a006a963fb8171",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\cases.tsv": "ff1986df4628da6b9033f512b12a5df61fa0220f995c6111af1f2690325d2b6b",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\channel_over_bound.bin": "72af5d35be3e90cb6f9907c33d8ba331b5bcfb7675b7d87db61dd3da0e9e12f9",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\channel_zero.bin": "f35e783405cea8c46db928e50cc919d6ffc96ee857f482c3dbbc921df9ae7dc6",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\clips\\342460de7e88f25854b15fb2b9dbb30bb111f2785d0223294a4ec25fbfa8ec97.anim": "342460de7e88f25854b15fb2b9dbb30bb111f2785d0223294a4ec25fbfa8ec97",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\clips\\3d079a6eac053f65f7f9bff26503bc1443dc3d1c963562da8caf9fd25005ee36.anim": "3d079a6eac053f65f7f9bff26503bc1443dc3d1c963562da8caf9fd25005ee36",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\clips\\8d26f08b5ee9780ee8ec232364900944162744abf7eb6167fd28895e585e9bca.anim": "8d26f08b5ee9780ee8ec232364900944162744abf7eb6167fd28895e585e9bca",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\clips\\ae00a4811bf31ec2075ed1480add77de58c0e397f9ada2848ee47a6758234b7c.anim": "ae00a4811bf31ec2075ed1480add77de58c0e397f9ada2848ee47a6758234b7c",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\clips\\bbfadcc31bc9cb8672f5f0e3efeba89c30a44f4967a4a17f6e0fbd841f1e5fd8.anim": "bbfadcc31bc9cb8672f5f0e3efeba89c30a44f4967a4a17f6e0fbd841f1e5fd8",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\counter_indefinite_domain.bin": "f65034269455031ab96e43945c77ece2d2993847ba9f86b75acb0ee06f59f71c",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\crt_policy.bin": "a2f01b35e685dfed79429e7bebb08fc06d8b14baef532f646a15c7d276aeab93",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\cw_reserved.bin": "bf8025506f7b231001abbc12a3a4c0b79403f3619d07a8264d64434224074efd",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\cw_unmasked.bin": "4b484d08c13e1e39ec315a97bbb32174017e64c72fb80565ac71f1563a31392c",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\frames_one.bin": "269d7a0053487f8cc94ead1aa6ce52afa800a6851b1ca62513e46a8d0c25682a",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\fresh_mode.bin": "d255d4b7e6653cd9cbb8d4c03a316ad23ce8ac24e08d2b8ac5356c3a247ac69a",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\header_tail.bin": "ceed5e0174af43879bb9166fe17e168ef247325ecabefa9e3144daf02c662521",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\initial_event_count.bin": "85ca777f206f942cd2e3931b0ae80ee4919f593d2f783fb7191fd07dfab61531",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\input.bin": "9935aeab89036bdf486bde4d1dd5f6ecf51f7e335d316f4f3999dcd63d1d4b62",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\interval_zero.bin": "283e346817835d068407e174b3cb43ab3e374f58f4118797cb7e293af033baa6",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\negative_absolute.bin": "a1c9d6e7919840d72c6928496cfb92fd0279211fd8af859bed21e4abfdda7847",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\negative_first_delta.bin": "cb191b04a07455d69d31bfe634cf780b0bde80d64d334c1549cfadde1a505efd",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\nonfinite_absolute.bin": "7104a3bbc62527e02eae013763538af9543de322fb325a2a806de74a11a925bc",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\owner_boundary.bin": "829ad3207726f512eb2abd4b72aefff86622c627c2e4cddbc0651a0c52a01b55",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\pending_count.bin": "a312651b92f39db76366f16084d380dc403fb07f569c7d201cbe235f06d93aac",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\property_mode.bin": "0ecda977879d11b450c973f85d03658b007b1e1970c9345ae9536c83d7b83d7d",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\report.json": "1c30ef806a2b2378c53ee39224ad98b7b6c246e2713595e5a1bc053c4ec64c86",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\reserved_header.bin": "d4bf6dbf47fe50f1b23e93ac2c38a06158195d292a24fdc747c4ee20137b0f5b",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\sampling_mask.bin": "35de5fd3d6824c97bd0fc567e5234c52cb5f2a801b69f2bb25b03184abdd74f8",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\step_count.bin": "918754ed82d3f69792a518bd86ed23c6810690cad9fec3ac54020e67f2a0b7ae",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\step_mode.bin": "502d62b41b37fc3650104589e227eb7360068cfd5fe26638891abaebd48dae5e",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-inputs-v1\\step_reserved.bin": "1f7287afc75c7e7962b8b39ee43004f92999089e704f5a59d66b6857a5e53327",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\007D0230-disassembly.txt": "e59256d86628f56527c84b9155e44d8d1d547b97d6bf9067f4a872f95b38faba",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\007D0230-original.bin": "6609b5d1f3be3db0887d7e2fa0647b8f9bb8b54f15ba7f96dce9f02997b58660",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\0099BCB0-disassembly.txt": "ee81970172e10a672fcfafee7864bf805f346db48df41e8200ccb9f90f4b8d0d",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\0099BCB0-original.bin": "225fc41fff2297f5e855d7a84ef50a68236bc8c858971b96376f0e4731a9bd17",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A1590-disassembly.txt": "f989089444240d3c0d408b7c75bc54ea9bad5d740058054c6f18779ac3999e36",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A1590-original.bin": "c8b4b84e836d0aa00198539f77fcb8aa0bb2a00b2d494d33f629cb51a0202f12",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A1690-disassembly.txt": "445f09cf45bdef6a16888b88ca16fdc17fa71c7c059e481cba42ba6d692200cc",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A1690-original.bin": "8eee99389c4e749212787e3fc42187ad838e345c6b289613c4f95674195565c2",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A1A20-disassembly.txt": "910975e24fa572822091400c8ab05217a12909dc191946a8dd4e2163202e2394",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A1A20-original.bin": "dc50079f3efe0b0462757d8cb3a83a423d24167107d8dc5c6edf865fe2c9339f",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A1A40-disassembly.txt": "764c36f20cfbb9aeba12a9bfba0c20823e7d4fb52408f01c622efe5ca0a78bf6",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A1A40-original.bin": "a26c7324bb3929b2ecf36e6fc9ab3f386f66179fb78b173f80335848daf68555",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A1B40-disassembly.txt": "90a9fc426f05d337a83ae4ac9a2b9ddf9a6a43bbb32cd262f0d2661646b2d574",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A1B40-original.bin": "ec5e7c3b6aac9920502159b72ff0808bf7d898d788dfc4c19d905630f5aa658a",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A1CB0-disassembly.txt": "fbd060ad5f8d3903df315a25835cca2a089b26c4ce9dc3539c80fb2273c92e83",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A1CB0-original.bin": "97dd0f0fa055b77f1f84b3620bf5ab2e369e702ab5fdbd1707d5d07ef57bef04",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A2BB0-disassembly.txt": "1dc52b93cc211d4fdbe5e28c21ff400785ca223a5f8612ab5484a05f289a746a",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A2BB0-original.bin": "d895d48fa9905d7fc5f797cd1df25b83fda2593bc9e5bac1713e8c2490c16df2",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A2BE0-disassembly.txt": "45bb0f831c02fae61edcbc5dbc083130094726b29f0c10d73087c760f7c7ec60",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A2BE0-original.bin": "4d1372d2e462775691bf962b3d3427265cc5dd3c7ac3035e26021cb059acb8ac",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A2FA0-disassembly.txt": "45bfc2d7a4317e75564c3922f273b3d1d7c5fef130b58fe0bba7623a09700620",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A2FA0-original.bin": "9bffbeabf61f7351dcefb53940736a195ab0957720bce71a57efaf80ddaf4fdc",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A3198-disassembly.txt": "55d3363e5b7b6207bcac9b0bb72c5bb96cf172b95dba9dfe1d0b2fbdd86ce8cd",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A3198-original.bin": "6d79f345323dfccb50c03344765ed294aa10e0aba3954dee3ac57a5f307dc593",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A32D4-disassembly.txt": "b2538ce404071139e2690fdd89e5989c6f7e0e3ade428d7eb1f5295252e80257",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A32D4-original.bin": "efff3157a3ca5b35323e3e8d6627375f92302dc2836d980c11052ebe680891a9",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A33C0-disassembly.txt": "e24d344fd555c107cd9b7b7b1b761820fc324e8ed7dc00c2a6fc67c81b4a790e",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A33C0-original.bin": "54aeea0d5cddbcb7d43a4a84f47305d0c7d1eae288d85e90df9ee6279e1b0924",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A7F30-disassembly.txt": "24e5d3051c6fcc0940d46b062472429a223a2949051624c2f34d0f7290f4b38c",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A7F30-original.bin": "317565586c39a2670944c9c142fa8f371a2446e907dbc4ebb8dcd7cdb31c6444",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A7FA0-disassembly.txt": "80621219ec9280b7598628ea42c58660a1e5b9cd9ab8245f517875e47f8c3617",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009A7FA0-original.bin": "05b88f8465309071e58d78c0de6b2fb6efa093d33ca790067da5b246a8932f16",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009FFE30-disassembly.txt": "f0b5da91adb95b2c475be262ee4992a21da8f44ac2aa322351c6330d2b2d537a",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\009FFE30-original.bin": "6a8f5448df0abb4d956095cdab423e5e69495b9b06671bd4841dd955cff5f8ad",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\00A00310-disassembly.txt": "5dbff5bcf44ac0fab191a695b1f5978df7535816c7f610073ff101d30ca3f6e0",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\00A00310-original.bin": "8e043a37aae421b60d00f7c523b837b1edd5fcb945728ad09c9b99911ca9b9c3",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\011E04D0-disassembly.txt": "e8899ded1d6d0bf96357d29dcfb39dc9d943180d9bdc6fdc3807a8ae528da3a8",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\011E04D0-original.bin": "04614067463d9a7345e7de2955a0c9fb5906c602499fb5df7f220ae231db613b",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\785A3350-disassembly.txt": "c18ecc771e11db4bc110feae5778a4d81c24170baed365390c915df056c8c684",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\785A3350-original.bin": "bfc57c26da922ceeef1faf0b70af4faabc2d4a85a8c878c2d8367e2558352c69",
  "D:\\AI\\RustSpore\\target\\source-cell\\runtime-fresh-loop-source-plan-v1\\plan.json": "4bfafdd8886e0126adb0b4b4788d3f7fb761aeab567296b329e7caa2d8c86a95",
  "D:\\AI\\RustSpore\\tools\\anim_root_pose_native_probe.py": "212980f1b37a332b1f668d6f7f7a1a435d1fa07af20a814c5c62eb34dcf78603",
  "D:\\AI\\RustSpore\\tools\\anim_root_pose_native_probe_v6.py": "27370aa2b5b5147f72183b8797974b9c4135067fed86e89fb82ed52622d571b9",
  "D:\\AI\\RustSpore\\tools\\anim_root_pose_sparse_carrier_audit_v2.py": "137ed05f14cd22ba0180a4a091b0877cabeeac9a74cf8fa906b1d16dee0b8aab",
  "D:\\AI\\RustSpore\\tools\\cell_native_math_probe.py": "a09ab9fa0b698892f6a8b116037cb6dffb54712e691e5f90c52f8d74abf05535",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_clip_metadata_native_probe.py": "a23df0abeec013ab41591f53379945c68ab923f23b4d703cb5cbf8b30218f9d8",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_clock_native_probe.py": "393af5d597c3587652c34e123c4c274bf676e66b99bd4fe0833be767c877e74b",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_clock_native_probe_v2.py": "3b75609ce2428bd34f8ef7b7691d416bd4d11657f9fd3da86feb3d1a8af782a0",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_controller_loop_native_probe.py": "d0ce1797cc27959e46c09557768326bfe1072923f49958a3368d58b47348a9f5",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_curves_fixture.py": "dd7d0f795fb9d3855572686853b507db722f6bf1f3f73803493780144986fa8c",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_data_native_probe.py": "57137bb6d2a1bcb119d85f04642703e57e97e77ccd0a4d5e915f404db8d7c1f8",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_data_native_run.py": "9673df5b44c40c7e92521ae15d0549d831203de33bdc523775bf048f93530856",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_fresh_aid_native_probe.py": "c25839f813107ae3210bb1504e732a8062f43f1681dd98c8e79c63602b9bcc59",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_fresh_loop_hybrid_carrier_audit_v1.py": "f10cd18c3d62436cb46136af4f26cb6a8b770db21752dba6cd9166f53efa12ef",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_fresh_loop_hybrid_carrier_audit_v2.py": "32196b2eb3b300a188f30d8edb22b9f76ed2151a82dfd1597b258445fd0883a1",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_fresh_loop_inputs.py": "e9d4b8967494a12c7b4711acda345267cf7dc9231dd5d31ac32a550821f5e08d",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_fresh_loop_native_probe.py": "8ecef145eff94aeba577fb8c85dbbc9d1e49a23ce12539692fdc41d260cc30ef",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_fresh_loop_native_probe_v2.py": "a42ceb8ab65aa89e70f68917a0492c2767f8f5ac5bdf76ef43c60917d18f9b98",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_fresh_loop_sparse_carrier.py": "20dad0c2846995ffc62ee102fccf26698be4641f11903f1aa2049917470d974e",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_fresh_loop_sparse_carrier_v2.py": "2a1c0f4d825e1e0f074f081cf3dc450331135befa16325f01f234dcb58fc13a4",
  "D:\\AI\\RustSpore\\tools\\cell_runtime_loop_clock_native_probe.py": "4ee2023865639a32b519a7616f965c7d5937a2f78d93309ab422520a8aa97179",
  "D:\\AI\\RustSpore\\tools\\cell_schema_probe.py": "05c9e3586e021b9de17668a8cce7ebc8c7ccbee9388dab25e9e4ca9cfc4493fe",
  "D:\\AI\\RustSpore\\tools\\native_archive_index_probe.py": "2de35986dd9cc94a83537d61bf54d6e101b99fe156a9ad5513e34d55f527bd6a",
  "D:\\AI\\RustSpore\\tools\\native_crt_acos_probe.py": "593ba6b27d75faa4d8af6d42310021d8a4bd15c5abe5474c5f93b468d346834b",
  "D:\\Games\\SPORE Collection\\SporeBin\\msvcr90.dll": "ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185",
  "D:\\Games\\SPORE Collection\\SporeBin\\SporeApp.exe": "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
  "D:\\AI\\RustSpore\\crates\\spore-core\\src\\anim.rs": "4fcfa708e5a36a3121ebfd0c3406fa5ab7ba6ddf085856748269c6be15de4fe4",
  "D:\\AI\\RustSpore\\crates\\spore-core\\src\\cell_runtime_fresh_aid.rs": "adbd247af869dceae3a4dc7be4af447b25c8f3205c846a361d03d2f757852a14",
  "D:\\AI\\RustSpore\\crates\\spore-core\\src\\cell_runtime_controller_loop.rs": "b77add96808ea2390c970f7352faeadfcef70664dc0e1c7b4674f1868ad71813",
  "D:\\AI\\RustSpore\\crates\\spore-core\\src\\cell_runtime_clip_metadata.rs": "c0e7563099332792f22a3ebd30358df6ee2a4610ce35f6a8d6efb01b7f8bbc15",
  "D:\\AI\\RustSpore\\crates\\spore-core\\src\\cell_runtime_loop_clock.rs": "becc2c63a357b72676ebd85c62c8599b0d03cb3b791a0b667e36f114f1670420",
  "D:\\AI\\RustSpore\\crates\\spore-core\\src\\cell_runtime_source_clock_root_data.rs": "810d6ff1e849ea0619ce869ff54bb9eed8f548bb4a68493fc2406bfdd8c4c6a1",
  "D:\\AI\\RustSpore\\crates\\spore-core\\src\\native_sse.rs": "cb97082bb99cc4cbcd3be42f64ff9afca34f7b921206d79891dfda0961e1d511",
  "D:\\AI\\RustSpore\\crates\\spore-core\\src\\native_x87.rs": "8d42e8be9d70be7e189ec2a61de2f05721a93840d384efea625aa9d59b50dc6f"
}
def save(p,v):
 with p.open('x')as f:json.dump(v,f,indent=2);f.write('\n')
def resolve(p):
 p=Path(p);return p if p.is_absolute()else ROOT/p

def inspect(raw,reply):
 assert len(raw)%256==0 and len(reply)==len(raw)//256*864
 projection=bytearray()
 for i in range(len(raw)//256):
  req=raw[i*256:(i+1)*256];cw=struct.unpack_from('<H',req,4)[0]
  for j in range(9):
   r=reply[i*864+j*96:i*864+(j+1)*96]
   assert r[10:12]==bytes(2)and struct.unpack_from('<H',r,88)[0]==cw
   assert struct.unpack_from('<H',r,90)[0]&0x3800==0 and fixture.word(r,92)&~0x3f==0x1f80
   assert fixture.word(r,56)==0 and fixture.word(r,60)==fixture.word(req,16)
   assert fixture.word(r,64)==1 and fixture.word(r,68)==1 and fixture.word(r,72)==0 and fixture.word(r,76)==fixture.word(req,32)
   projection.extend(r[:88])
 return bytes(projection)

def review(path,runner_required=False):
 r=json.loads(path.read_text());assert r['no_blocking_static_issue_found']is True and r['native_executed']is False
 pins=r['pins'];assert CARRIER in pins.values()and OWNED in pins.values()
 if runner_required:assert digest(Path(__file__))in pins.values(),'Supplement must pin this exact runner'
 files=[path]
 for name,value in pins.items():q=resolve(name);assert digest(q)==value,name;files.append(q)
 return files

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--peer-review',type=Path);p.add_argument('--runner-review',type=Path);p.add_argument('--run',action='store_true');a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve())
 files=[Path(__file__)]
 for name,value in HARD_PINS.items():q=resolve(name);assert digest(q)==value,name;files.append(q)
 assert digest(HOST/'carrier-x86.exe')==CARRIER and digest(HOST/'host-x86.exe')==OWNED
 actual=linked.base.audit(SimpleNamespace(carrier=HOST/'carrier-x86.exe',owned_exe=HOST/'host-x86.exe',owned_map=HOST/'host-x86.map',source_exe=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),source_crt=Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll')))
 assert actual==json.loads((HOST/'root-linked-audit-v2.json').read_text())
 report=json.loads((INPUT/'report.json').read_text());assert report['records']==300 and report['total_clock_calls']==2700 and len(report['protocols'])==23
 for name,value in report['pins'].items():q=resolve(name);assert digest(q)==value,name;files.append(q)
 raw=(INPUT/'input.bin').read_bytes();assert len(raw)==300*256 and digest(INPUT/'input.bin')==report['input_sha256'];cases=json.loads((INPUT/'cases.json').read_text());assert len(cases)==300
 for i,row in enumerate(cases):
  req=raw[i*256:(i+1)*256];payload=(INPUT/row['clip']).read_bytes();assert digest(INPUT/row['clip'])==row['source_sha256']
  assert fixture.word(payload,8)==25 and fixture.word(payload,4)==len(payload)
  assert [fixture.word(req,x)for x in[24,28,32,36]]==[fixture.word(payload,x)for x in[280,284,288,324]]
  assert fixture.word(req,36)in[7,9,10,11]
  fixture.domain(req,struct.unpack('<d',struct.pack('<Q',int(row['duration_bits'],16)))[0])
 for name,value in report['protocols'].items():assert digest(INPUT/(name+'.bin'))==value
 # Check actual assembly. Poison is assigned but never loaded into XMM: only repeated-run determinism is claimed.
 asm=(HOST/'host-x86.asm').read_text();assert 'mov Poison,0\ncall RunOnce' in asm and 'mov Poison,0FFFFFFFFh\ncall RunOnce'in asm
 assert not any('xmm'in line.lower()for line in asm.splitlines()if not line.startswith('.xmm'))
 for path in[INPUT,HOST]:files.extend(q for q in path.rglob('*')if q.is_file())
 if a.peer_review:files.extend(review(a.peer_review))
 if a.runner_review:files.extend(review(a.runner_review,True))
 if a.run:assert a.peer_review and a.runner_review,'Actual linked peer and runner supplemental peer required'
 a.out.mkdir(parents=True,exist_ok=False);save(a.out/'preflight.json',dict(schema='fresh-loop-hybrid-preflight-v1',native_executed=False,execution_requested=a.run,actual_linked_audit=True,records=300,snapshots=2700,protocol_rejections=23,single_attempt=True,retries=0,distinct_XMM_poison_loaded=False,repeated_RunOnce_full_output_comparison=True,peer_review=bool(a.peer_review),runner_review=bool(a.runner_review),carrier_sha256=CARRIER,owned_sha256=OWNED))
 if not a.run:print('Preflight PASS; no native execution');return
 try:
  with pinned(list(dict.fromkeys(q.resolve()for q in files)),a.out):
   proto=a.out/'protocol';proto.mkdir()
   def execute(label,data,code,expected=None):
    req=proto/(label+'-input.bin')
    with req.open('xb')as f:f.write(data)
    with pinned([req],create_pin_dir(proto,label)):
     status,out=saved_file_run(HOST/'carrier-x86.exe',req,proto,label)
     assert status['exit']==code and status['stderr_bytes']==0,(label,status)
     got=out.read_bytes()
     if expected is not None:assert got==expected,label
     return got
   ref=execute('reference',raw[:256],0);inspect(raw[:256],ref)
   execute('empty',b'',0,b'');execute('repeat_two',raw[:256]*2,0,ref*2)
   sequence=execute('valid_sequence',raw[:512],0);inspect(raw[:512],sequence)
   execute('partial_first',raw[:128],7,b'');execute('complete_then_partial',raw[:256]+raw[:128],7,ref)
   for name in report['protocols']:execute(name,(INPUT/(name+'.bin')).read_bytes(),7,b'')
   with pinned([INPUT/'input.bin'],create_pin_dir(a.out,'corpus')):
    status,out=saved_file_run(HOST/'carrier-x86.exe',INPUT/'input.bin',a.out,'corpus')
    assert status['exit']==0 and status['stderr_bytes']==0,status
   native=out.read_bytes();projection=inspect(raw,native);assert native[:864]==ref and native[:1728]==sequence
   pp=a.out/'native-projection792.bin'
   with pp.open('xb')as f:f.write(projection)
   save(a.out/'report.json',dict(schema='full-fresh-activation-private-loop-native-run-v1',native_executed=True,records=300,snapshots=2700,projection_bytes_per_record=792,protocol_rejections=23,framing_protocols=5,reference_requests=1,retries=0,carrier_sha256=CARRIER,owned_sha256=OWNED,raw_output_sha256=digest(out),native_projection_sha256=digest(pp),rust_accepted=False,distinct_XMM_poison_loaded=False,repeated_RunOnce_full_output_comparison=True,rust_next='Promote pending verifier only after review; INPUT-DIR corpus-output.bin NEW-RUST792.bin',typed_boundary='Explicit borrowed v25 resource, fresh owner0, direct Repeat, actual resource channel count/fourzero masks, isolated serial/generation seeds; no live actor/eventdispatch/sampling/registry claim.'))
 except BaseException as e:save(a.out/'failure.json',dict(error=repr(e),retained=True,retries=0,single_attempt=True));raise

def create_pin_dir(parent,label):
 p=parent/(label+'-pins');p.mkdir(exist_ok=False);return p
if __name__=='__main__':main()
