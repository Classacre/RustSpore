"""Extract six hash-bound candidate EFFDIRs into a new ignored target directory.

Input packages are opened read-only. The output location must be below this
checkout's target/source-cell directory; payloads are never written elsewhere.
The manifest preserves package/key/decoded-length/SHA-256 provenance.
"""
import argparse
import hashlib
import json
from pathlib import Path

from dbpf_extract import payload
from gmdl_skin_probe import indexes

TYPE = 0xEA5118B0
GROUP = 0x0001FFFF
EXPECTED = {
    ('PatchData.package', 0x40450202C1195706): (5134522, '3af01e03a287235d7b31ca0c25dd8a07b4997c50bb17d163b13370ceba2c7f8e'),
    ('PatchData.package', 0x40450200C1195706): (5068716, '578cd63c8efc8fe5141d47c0101987d65e2f256e81bb88bc76895c9d303a6036'),
    ('PatchData.package', 0x40450201C1195706): (5122674, '683c0e99913d555acb9a3b1b164d763f09fcfce77f8e4062ccfbd21b6e2d5114'),
    ('Spore_Graphics.package', 0x40450202C1195706): (5134522, 'fc9da7b033b40a789539fe87d327e619326045a0eeb52fb5f7e88eee3ea3fb93'),
    ('Spore_Graphics.package', 0x40450200C1195706): (5068716, '8c277b813c50d27e4041be088a39a252e11f459181b857ba8002e7564709c187'),
    ('Spore_Graphics.package', 0x40450201C1195706): (5122674, 'f740c1f241b6595e0b8738651a4fc11275127e16bb89b950aade41dadcaf9cb4'),
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    allowed = (Path(__file__).resolve().parent.parent / 'target' / 'source-cell').resolve()
    out = args.out.resolve()
    if out == allowed or not out.is_relative_to(allowed):
        raise ValueError('output must be a new directory below target/source-cell')
    if out.exists():
        raise FileExistsError(out)
    # Validate all six identities before writing any bytes.
    selected = []
    for name in ('PatchData.package', 'Spore_Graphics.package'):
        path = args.install / 'Data' / name
        for entry in indexes(path):
            key = (name, entry['instance'])
            if entry['type'] != TYPE or entry['group'] != GROUP or key not in EXPECTED:
                continue
            data = payload(path, entry)
            digest = hashlib.sha256(data).hexdigest()
            if (len(data), digest) != EXPECTED[key] or len(data) != entry['dec']:
                raise ValueError(f'{name} {entry["instance"]:016X}: size or hash mismatch')
            selected.append((key, data, digest))
    if len(selected) != len(EXPECTED) or {x[0] for x in selected} != set(EXPECTED):
        raise ValueError('missing or duplicate candidate entries')
    out.mkdir(parents=True, exist_ok=False)
    records = []
    for (name, instance), data, digest in selected:
        filename = f'{Path(name).stem}-{GROUP:08X}-{TYPE:08X}-{instance:016X}.bin'
        with (out / filename).open('xb') as f:
            f.write(data)
        records.append(dict(package=name, group=f'{GROUP:08X}', type=f'{TYPE:08X}',
                            instance=f'{instance:016X}', decoded_bytes=len(data),
                            payload_sha256=digest, file=filename))
    with (out / 'manifest.json').open('x', encoding='utf-8') as f:
        json.dump(dict(schema='cell-skybox-effdir-payloads-v1', records=records), f, indent=2)
        f.write('\n')
    print(f'{len(records)} hash-verified payloads written to {out}')


if __name__ == '__main__':
    main()
