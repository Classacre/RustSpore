import sys
if not sys.flags.isolated or not sys.flags.no_site or sys.flags.optimize:
    raise RuntimeError('Require isolated, no-site, nonoptimized Python: python -I -S')
_base = sys.base_prefix.rstrip('\\/').casefold()
sys.path[:] = [p for p in sys.path if p and (p.casefold() == _base or p.casefold().startswith(_base + '\\') or p.casefold().startswith(_base + '/'))]
if not sys.path:
    raise RuntimeError('No trusted interpreter library paths')
for _name in ('argparse', 'hashlib', 'json', 'struct', 'pathlib', 'ast', 'ctypes', 'importlib.abc'):
    if _name in sys.modules:
        raise RuntimeError('Unexpected preloaded admission module: ' + _name)
import argparse, hashlib, json, struct
from pathlib import Path
MANDATORY = {'D:\\AI\\RustSpore\\tools\\anim_cid_child_mode2_native_prepare_v1.py': 'a9d9310b8c42e83001b8075e7b2736635ca9ad69c1ab2c81bd8bcf3c9b6a91ab', 'D:\\AI\\RustSpore\\tools\\anim_cid_child_mode2_native_run_v1.py': 'd2c9b8dc5eff0cc85d3f254c64458d3b70a59d93375e5c99bebbbfdacee463c3', 'D:\\AI\\RustSpore\\tools\\anim_root_pose_sparse_carrier_v3.py': '8384485a7318a2e8b9f7ee0207ffe695bcf850c0054e156acaccf8865a93540a', 'D:\\AI\\RustSpore\\tools\\anim_root_pose_sparse_carrier_audit_v2.py': '137ed05f14cd22ba0180a4a091b0877cabeeac9a74cf8fa906b1d16dee0b8aab', 'D:\\AI\\RustSpore\\tools\\cell_runtime_data_native_run.py': '9673df5b44c40c7e92521ae15d0549d831203de33bdc523775bf048f93530856', 'D:\\AI\\RustSpore\\tools\\cell_schema_probe.py': '05c9e3586e021b9de17668a8cce7ebc8c7ccbee9388dab25e9e4ca9cfc4493fe', 'D:\\AI\\RustSpore\\tools\\cell_native_math_probe.py': 'a09ab9fa0b698892f6a8b116037cb6dffb54712e691e5f90c52f8d74abf05535', 'D:\\AI\\RustSpore\\tools\\anim_cid_child_mode2_fixtures_v1.py': 'dadfc7f4849066c5c76beb366c717379e047c4fc0e78af959169e05b9be828ff', 'D:\\AI\\RustSpore\\target\\animation\\cid-child-mode2-fixtures-v1\\fixtures.json': '8846a9a46f3db9f529c807de65c11f96fcc9008a55cc602669ba15f1d09633d0', 'D:\\AI\\RustSpore\\target\\animation\\cid-child-mode2-fixtures-v1\\manifest.json': '9b59909421cbe0d14daae61ad0fc8a7d40c2eb0be44c44c18be3b9d523c95bdb', 'D:\\AI\\RustSpore\\target\\animation\\cid-child-mode2-source-audit-v1.json': '30ca9e3de9f73701c83d1c3cb7208f5e6980ce6bbe114d4417ac97b1c0be710d', 'D:\\AI\\RustSpore\\target\\animation\\cid-child-mode2-host-contract-v1.json': 'a07854b0cdc841fdb2103440887ae769f44cf9da6cc361b53ca23745e439dc61', 'D:\\AI\\RustSpore\\target\\rea\\20261027-source-runtime-v4\\32-analyze_function.json': 'b97990c18105a35c541da75bd79fa0ffeb73cc61877288ebee934fa704503490', 'D:\\AI\\RustSpore\\target\\rea\\20261028-source-runtime-v6\\99-analyze_function.json': '1af256fb30fa8ee6a50781587b80543da89a95b1489f742ee1860d2b0bd48dbb', 'D:\\AI\\RustSpore\\target\\animation\\cid-node-init-actual-body-inputs-v1\\input.bin': 'da8efb755186a684be861e59a9805325b9f51ab8806ddc8460e5e66ed949f165', 'D:\\AI\\RustSpore\\target\\animation\\cid-node-init-actual-body-inputs-v1\\index.json': '655c11a68e605406c4f818408becdceb0bec26426a255e0a99d7182d938744fb', 'D:\\AI\\RustSpore\\target\\animation\\cid-node-init-actual-body-inputs-v1\\manifest.json': 'd8eda5b8e107029623b6ca0d1e859de620aa5030c257319b84271933b322f025', 'D:\\Games\\SPORE Collection\\SporeBin\\SporeApp.exe': '17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a', 'D:\\AI\\RustSpore\\tools\\anim_root_pose_native_probe.py': '212980f1b37a332b1f668d6f7f7a1a435d1fa07af20a814c5c62eb34dcf78603', 'D:\\AI\\RustSpore\\tools\\anim_root_pose_native_probe_v6.py': '27370aa2b5b5147f72183b8797974b9c4135067fed86e89fb82ed52622d571b9', 'D:\\AI\\RustSpore\\tools\\cell_runtime_data_native_probe.py': '57137bb6d2a1bcb119d85f04642703e57e97e77ccd0a4d5e915f404db8d7c1f8', 'D:\\AI\\RustSpore\\tools\\native_crt_acos_probe.py': '593ba6b27d75faa4d8af6d42310021d8a4bd15c5abe5474c5f93b468d346834b', 'D:\\AI\\RustSpore\\tools\\anim_cid_node_init_inputs_v1.py': '5807a735819acc085023c2a8a256c5dadb8bfc8f61db78c6c668ed8bc147b3ce', 'D:\\AI\\RustSpore\\tools\\anim_body_finalizer_native_probe.py': 'a965c3d96d8a933a9357b323ccb6d08890b8fb4fb9ca723a5cb31d8a7c70531b', 'D:\\AI\\RustSpore\\tools\\anim_cid_child_mode2_host_plan_v1.py': '7756f0e4146565a4565239fb4ef92315ec61681f5e5ed55927b6b086b0cde97b', 'D:\\AI\\RustSpore\\tools\\anim_cid_child_mode2_source_audit_v1.py': '6fd0bb9f9f7cf69674a1f350cbd4ebe0b7972ca7f327b75bd72b324c4b7103dc', 'D:\\AI\\RustSpore\\target\\animation\\cid-node-init-actual-body-inputs-v1\\expected.bin': 'a5013b5e97a27ce83c0f8993fa676baef4f6a73c902cb3cd8aedeab51edbadac', 'D:\\AI\\RustSpore\\target\\rea\\20261028-source-runtime-v6\\100-analyze_function.json': '8544dd0552897d44417a80b74b391a8a2b94ce0b281e27f1c800c83bf0e5b37b', 'D:\\AI\\RustSpore\\target\\rea\\20261028-source-runtime-v6\\101-analyze_function.json': '0e62e149035b564e7936ec3879d0acd062c1591d4354a673fc8bd8d8233735cc', 'D:\\AI\\RustSpore\\target\\animation\\nonroot-selector-Z-inputs-v1-complete\\input.bin': '3e04e0c1a7766ad80f85d1c4920240f9c904f1e5007b53002b7e611cd9312c63', 'D:\\AI\\RustSpore\\target\\animation\\nonroot-selector-Z-inputs-v1-complete\\body-index.json': '9851e0b8f0cfea851923f3b4c17e7611446ac5601d9d80bfaef47c0636ad52ed', 'D:\\AI\\RustSpore\\target\\animation\\nonroot-selector-Z-inputs-v1-complete\\manifest.json': '1c17b0280d86d807c00d63efa004ccf343cab315580d090f3277efe66e88e051'}
MANDATORY[str(Path('D:\\AI\\RustSpore\\tools\\anim_cid_child_mode2_native_prepare_v3.py'))] = 'f8d39bf302216c56ab9623ad3657098c81a13c6ee414ba09b3d126163cf190c5'

def verify_fixed():
    for name, expected in MANDATORY.items():
        path = Path(name)
        if not (path.is_file() and hashlib.sha256(path.read_bytes()).hexdigest() == expected):
            raise AssertionError(name)
verify_fixed()
early = argparse.ArgumentParser(add_help=False)
early.add_argument('--host-dir', type=Path)
early.add_argument('--carrier', type=Path)
early.add_argument('--linked-peer', type=Path)
early.add_argument('--plan', type=Path)
early.add_argument('--runner-peer', type=Path)
early.add_argument('--run', action='store_true')
a0, _ = early.parse_known_args()
if a0.run:
    if not (a0.plan and a0.runner_peer):
        raise AssertionError('assertion failed')
if a0.plan:
    if not (a0.host_dir and a0.carrier and a0.linked_peer):
        raise AssertionError('assertion failed')
    plan0 = json.loads(a0.plan.read_text())
    if not plan0['schema'] == 'cid-child-mode2-runner-plan-v3':
        raise AssertionError('assertion failed')
    required0 = {str(Path(x).resolve()) for x in MANDATORY}
    required0.update((str(x.resolve()) for x in [Path(__file__), a0.host_dir / 'host-x86.asm', a0.host_dir / 'manifest.json', a0.host_dir / 'host-x86.exe', a0.host_dir / 'host-x86.map', a0.carrier, a0.linked_peer]))
    if not set(plan0['pins']) == required0:
        raise AssertionError('plan pin set omitted or substituted an artifact')
    if not plan0['runner_sha256'] == hashlib.sha256(Path(__file__).read_bytes()).hexdigest():
        raise AssertionError('assertion failed')
    for name, digest in plan0['pins'].items():
        if not hashlib.sha256(Path(name).read_bytes()).hexdigest() == digest:
            raise AssertionError(name)
    for name, digest in MANDATORY.items():
        if not plan0['pins'][str(Path(name).resolve())] == digest:
            raise AssertionError(name)
import ast, sys, importlib.abc, importlib.util, importlib.machinery
sys.meta_path[:] = [importlib.machinery.BuiltinImporter, importlib.machinery.FrozenImporter, importlib.machinery.PathFinder]

class _Checks(ast.NodeTransformer):

    def visit_Assert(self, node):
        self.generic_visit(node)
        msg = node.msg if node.msg is not None else ast.Constant('assertion failed')
        return ast.copy_location(ast.If(test=ast.UnaryOp(op=ast.Not(), operand=node.test), body=[ast.Raise(exc=ast.Call(func=ast.Name(id='AssertionError', ctx=ast.Load()), args=[msg], keywords=[]), cause=None)], orelse=[]), node)

class _PinnedLoader(importlib.abc.Loader):

    def __init__(self, path, digest):
        self.path, self.digest = (path, digest)

    def create_module(self, spec):
        return None

    def exec_module(self, module):
        data = self.path.read_bytes()
        if hashlib.sha256(data).hexdigest() != self.digest:
            raise RuntimeError('Imported source changed: ' + str(self.path))
        tree = _Checks().visit(ast.parse(data, filename=str(self.path)))
        ast.fix_missing_locations(tree)
        exec(compile(tree, str(self.path), 'exec', optimize=0), module.__dict__)

class _PinnedFinder(importlib.abc.MetaPathFinder):

    def find_spec(self, fullname, path=None, target=None):
        if '.' in fullname:
            return None
        candidate = Path(__file__).resolve().parent / (fullname + '.py')
        if not candidate.is_file():
            return None
        key = str(candidate.resolve())
        if key not in MANDATORY:
            raise ImportError('Unlisted project import: ' + key)
        spec = importlib.util.spec_from_loader(fullname, _PinnedLoader(candidate, MANDATORY[key]), origin=key)
        spec.has_location = True
        return spec
for _path in MANDATORY:
    _p = Path(_path)
    if _p.parent == Path(__file__).resolve().parent and _p.suffix == '.py' and (_p.stem in sys.modules):
        raise RuntimeError('Preloaded project module: ' + _p.stem)
sys.meta_path.insert(0, _PinnedFinder())
import anim_cid_child_mode2_native_prepare_v3 as prep
from anim_root_pose_sparse_carrier_audit_v2 import Image, syms
from cell_schema_probe import SourceImage
from cell_runtime_data_native_run import pinned, saved_file_run
ROOT = prep.ROOT

def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()

def save(p, v):
    with p.open('x') as f:
        json.dump(v, f, indent=2)
        f.write('\n')

def verify_peer(p, required, schema):
    v = json.loads(p.read_text())
    if not (v['no_blocking_static_issue_found'] and (not v['native_executed']) and (v['review_method'] == 'independent') and (v['schema'] == schema)):
        raise AssertionError('assertion failed')
    for item in required:
        if not v['pins'].get(str(item.resolve())) == sha(item):
            raise AssertionError(str(item))
    for name, value in v['pins'].items():
        if not sha(Path(name)) == value:
            raise AssertionError(name)
    return v

def static(host, carrier):
    image = SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    fx = prep.fixture_bytes()
    audit = prep.proof(image)
    asm = host / 'host-x86.asm'
    exe = host / 'host-x86.exe'
    mapping = host / 'host-x86.map'
    if not asm.read_text() == prep.host(image, fx):
        raise AssertionError('assertion failed')
    m = json.loads((host / 'manifest.json').read_text())
    if not (m['asm_sha256'] == sha(asm) and m['schema'] == 'cid-child-mode2-host-v3' and (m['tool_sha256'] == sha(prep.__file__)) and (m['mandatory_pins'] == prep.MANDATORY)):
        raise AssertionError('assertion failed')
    rebuilt, _ = prep.build(exe, mapping, image)
    if not rebuilt == carrier.read_bytes():
        raise AssertionError('assertion failed')
    old = Image(exe)
    new = Image(carrier)
    symbols = syms(mapping)
    if not (old.base == 687865856 and new.base == prep.BASE):
        raise AssertionError('assertion failed')
    ni, _ = new.imports()
    oi, _ = old.imports()
    if not (ni == oi and set(ni.values()) == {'GetStdHandle', 'ReadFile', 'WriteFile', 'ExitProcess', 'GetModuleHandleA', 'GetLastError'}):
        raise AssertionError('assertion failed')
    if not all((not (s['flags'] & 536870912 and s['flags'] & 2147483648) for s in new.sections)):
        raise AssertionError('assertion failed')
    for address, data, flags in prep.pages(image, audit, symbols['OriginalDone']):
        skip = 4096 if address == prep.BASE else 0
        if not (new.read(address + skip, len(data) - skip) == data[skip:] and new.perm(address + skip, len(data) - skip) == flags & 3758096384):
            raise AssertionError('assertion failed')
    if not (new.read(symbols['Fixtures'], len(fx)) == fx and new.perm(symbols['Fixtures'], len(fx)) == 1073741824):
        raise AssertionError('assertion failed')
    for name, size in prep.SIZES.items():
        at = symbols[name + 'Buffer']
        if not (symbols[name + 'Before'] == at - 16 and symbols[name + 'After'] == at + size and (new.perm(at - 16, size + 32) == 3221225472) and (new.perm(symbols[name + 'Shadow'], size) == 3221225472)):
            raise AssertionError('assertion failed')
    first, last = (symbols['OwnedStart'], symbols['OwnedEnd'])
    if not new.read(first, last - first) == old.read(first, last - first):
        raise AssertionError('assertion failed')
    return dict(static_pass=True, actual_linked=True, native_executed=False, carrier_sha256=sha(carrier), owned_sha256=sha(exe), map_sha256=sha(mapping), source_pages_exact=True, imports_closed=True, no_WX=True, limits='Reproducibility and basic PE checks; independent control flow, ABI, byte guards, stack, runner review required.')

def inspect(raw, n):
    if not len(raw) == n * 60:
        raise AssertionError('assertion failed')
    for i in range(n):
        at = i * 60
        if not struct.unpack_from('<3I', raw, at) == (1, 1, 1):
            raise AssertionError('assertion failed')
        if not raw[at + 40:at + 48] == bytes.fromhex('0400ffff0400ffff'):
            raise AssertionError('assertion failed')
        if not struct.unpack_from('<I', raw, at + 48)[0] == 3212836864:
            raise AssertionError('assertion failed')
        cw, sw, csr = struct.unpack_from('<HHI', raw, at + 52)
        if not (cw == 639 and sw & 14336 == 0 and (csr & ~63 == 8064)):
            raise AssertionError('assertion failed')
    return raw

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--host-dir', type=Path, required=True)
    p.add_argument('--carrier', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    p.add_argument('--linked-peer', type=Path, required=True)
    p.add_argument('--plan', type=Path)
    p.add_argument('--runner-peer', type=Path)
    p.add_argument('--run', action='store_true')
    a = p.parse_args()
    a.out.resolve().relative_to((ROOT / 'target').resolve())
    checks = static(a.host_dir, a.carrier)
    owned = a.host_dir / 'host-x86.exe'
    mapping = a.host_dir / 'host-x86.map'
    files = [Path(x) for x in MANDATORY] + [Path(__file__), a.host_dir / 'host-x86.asm', a.host_dir / 'manifest.json', owned, mapping, a.carrier, a.linked_peer]
    verify_peer(a.linked_peer, [owned, mapping, a.carrier, Path(prep.__file__), a.host_dir / 'host-x86.asm', a.host_dir / 'manifest.json'], 'cid-child-mode2-linked-review-v3')
    if a.plan:
        plan = json.loads(a.plan.read_text())
        if not plan['runner_sha256'] == sha(__file__):
            raise AssertionError('assertion failed')
        for name, value in plan['pins'].items():
            if not sha(Path(name)) == value:
                raise AssertionError(name)
        files.append(a.plan)
    if a.runner_peer:
        if not a.plan:
            raise AssertionError('assertion failed')
        verify_peer(a.runner_peer, [Path(__file__), a.plan, owned, mapping, a.carrier], 'cid-child-mode2-runner-review-v3')
        files.append(a.runner_peer)
        files.extend((Path(name) for name in json.loads(a.runner_peer.read_text())['pins']))
    files.extend((Path(name) for name in json.loads(a.linked_peer.read_text())['pins']))
    if a.run:
        if not (a.plan and a.runner_peer):
            raise AssertionError('Frozen plan and independent linked and exact-runner reviews required')
    if a.run:
        verify_fixed()
        verify_peer(a.linked_peer, [owned, mapping, a.carrier, Path(prep.__file__), a.host_dir / 'host-x86.asm', a.host_dir / 'manifest.json'], 'cid-child-mode2-linked-review-v3')
        verify_peer(a.runner_peer, [Path(__file__), a.plan, owned, mapping, a.carrier], 'cid-child-mode2-runner-review-v3')
        for name, value in json.loads(a.plan.read_text())['pins'].items():
            if not sha(Path(name)) == value:
                raise AssertionError(name)
    a.out.mkdir(parents=True, exist_ok=False)
    if not a.plan:
        save(a.out / 'plan.json', dict(schema='cid-child-mode2-runner-plan-v3', runner_sha256=sha(__file__), native_executed=False, pins={str(f.resolve()): sha(f) for f in dict.fromkeys([Path(x) for x in MANDATORY] + [Path(__file__), a.host_dir / 'host-x86.asm', a.host_dir / 'manifest.json', owned, mapping, a.carrier, a.linked_peer])}))
    save(a.out / 'preflight.json', dict(**checks, execution_requested=a.run, synthetic_stage=True, fixture_count=64, independent_linked_peer=str(a.linked_peer)))
    if not a.run:
        print('Static preflight PASS; native not executed')
        return
    try:
        with pinned(list(dict.fromkeys((f.resolve() for f in files))), a.out):
            protocol = a.out / 'protocol'
            protocol.mkdir()

            def execute(label, data, code, expected=None):
                request = protocol / (label + '-input.bin')
                request.write_bytes(data)
                pins = protocol / (label + '-pins')
                pins.mkdir()
                with pinned([request], pins):
                    status, out = saved_file_run(a.carrier, request, protocol, label)
                if not (status['exit'] == code and (not status['timeout']) and (status['stderr_bytes'] == 0)):
                    raise AssertionError((label, status))
                got = out.read_bytes()
                if expected is not None:
                    if not got == expected:
                        raise AssertionError(label)
                return got
            one = execute('single', struct.pack('<I', 0), 0)
            inspect(one, 1)
            execute('empty', b'', 0, b'')
            execute('repeat', bytes(8), 0, one + one)
            execute('partial', bytes(3), 71, b'')
            execute('out_of_range', struct.pack('<I', 64), 71, b'')
            execute('complete_then_partial', bytes(7), 71, one)
            data = b''.join((struct.pack('<I', i) for i in range(64)))
            raw = execute('corpus', data, 0)
            inspect(raw, 64)
            if not raw[:60] == one:
                raise AssertionError('assertion failed')
            output = a.out / 'native-output.bin'
            output.write_bytes(raw)
            save(a.out / 'report.json', dict(native_executed=True, fixtures=64, synthetic_stage=True, native_sha256=sha(output), rust_comparison=False, actual_actor_association=False, scope='mode-2 branch only; no live animation or full traversal'))
    except BaseException as e:
        save(a.out / 'failure.json', dict(error=repr(e), retained=True, retries=0))
        raise
if __name__ == '__main__':
    main()
