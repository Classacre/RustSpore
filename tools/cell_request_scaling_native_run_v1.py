"""Compare prebuilt original scaling host and bounded model; never build here."""
import argparse, hashlib, json, random, struct, subprocess
from pathlib import Path
ROOT = Path(__file__).resolve().parent.parent
MULT = 0x278DDE6D
SUMS = [0, 1, 2, 0x007fffff, 0x00800000, 0x00800001, 0x3f000000,
        0x3f7fffff, 0x3f800000, 0x3f800001, 0x40000000, 0x7f7fffff]

def corpus():
    inv = pow(MULT, -1, 1 << 32)
    targets = [1, 2, 0x7ffffffe, 0x7fffffff, 0x80000000, 0x80000001,
               0x80000002, 0xfffffffe, 0xffffffff]
    seeds = {1, 2, 3, 0x7fffffff, 0x80000000, 0x80000001, 0xfffffffe, 0xffffffff}
    seeds.update((target * inv) & 0xffffffff for target in targets)
    rng = random.Random(0xe6cc30)
    seeds.update(rng.randrange(1, 1 << 32) for _ in range(64))
    return [(state, total, 0x7f | pc | rc) for pc in (0x200, 0x300)
            for rc in (0, 0x400, 0x800, 0xc00)
            for state in sorted(seeds) for total in SUMS]

def digest(data): return hashlib.sha256(data).hexdigest()
def main():
    p = argparse.ArgumentParser(); p.add_argument('--host', type=Path, required=True)
    p.add_argument('--adapter', type=Path, required=True); p.add_argument('--out', type=Path, required=True)
    args = p.parse_args(); target = (ROOT / 'target').resolve()
    for path in (args.host, args.adapter, args.out): path.resolve().relative_to(target)
    rows = corpus(); request = b''.join(struct.pack('<III', *row) for row in rows)
    args.out.mkdir(parents=True, exist_ok=False); (args.out/'request.bin').write_bytes(request)
    results = []
    for name, exe, width in [('native', args.host, 8), ('model', args.adapter, 12)]:
        proc = subprocess.run([str(exe.resolve())], input=request, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, timeout=120, creationflags=subprocess.CREATE_NO_WINDOW)
        (args.out/(name+'.bin')).write_bytes(proc.stdout)
        (args.out/(name+'.stderr')).write_bytes(proc.stderr)
        if proc.returncode != 0 or proc.stderr or len(proc.stdout) != width * len(rows):
            raise RuntimeError((name, proc.returncode, len(proc.stdout), len(rows), proc.stderr[:1000]))
        results.append(proc.stdout)
    native, model = results; rejected = []; mismatches = []
    for i, (seed, total, cw) in enumerate(rows):
        ns, nb = struct.unpack_from('<II', native, i*8)
        status, ms, mb = struct.unpack_from('<III', model, i*12)
        expected_rejection = ((seed * MULT) & 0xffffffff) == 0x80000000 and (cw & 0xc00) == 0x400
        if expected_rejection:
            if status != 1 or ms != seed or mb != 0 or ns != 0x80000000:
                mismatches.append(dict(index=i, reason='rejection contract', seed=hex(seed), sum=hex(total), cw=hex(cw), native=[hex(ns),hex(nb)], model=[status,hex(ms),hex(mb)]))
            rejected.append(dict(index=i, seed=hex(seed), sum=hex(total), cw=hex(cw), native_state=hex(ns), native_draw_bits=hex(nb)))
        elif status != 0 or (ns, nb) != (ms, mb):
            mismatches.append(dict(index=i, reason='bit or state mismatch', seed=hex(seed), sum=hex(total), cw=hex(cw), native=[hex(ns),hex(nb)], model=[status,hex(ms),hex(mb)]))
    if not rejected: raise RuntimeError('zero-sign rejection case missing')
    report = dict(cases=len(rows), compared=len(rows)-len(rejected), rejected_zero_sign=rejected,
                  mismatch_count=len(mismatches), mismatches=mismatches, request_sha256=digest(request),
                  native_sha256=digest(native), model_sha256=digest(model),
                  host_exe_sha256=digest(args.host.read_bytes()), adapter_exe_sha256=digest(args.adapter.read_bytes()),
                  precision=[53,64], rounding_modes=4, comparison='exact state and raw binary32 bits', retail_control_word_known=False)
    (args.out/'report.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print(json.dumps({k:v for k,v in report.items() if k not in ('rejected_zero_sign','mismatches')}, indent=2))
    if mismatches: raise SystemExit(1)
if __name__ == '__main__': main()
