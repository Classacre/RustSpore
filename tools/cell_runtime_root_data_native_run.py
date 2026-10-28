"""Join accepted curve replies, original scalar contributions and data2 reads.

Inputs remain frozen; all new stage requests/replies live in a separate run
directory. This executes only two already reviewed hidden owned fixture hosts.
It does not choose live clips, advance clocks or resolve real game actors.
"""
import argparse
import json
from pathlib import Path
import struct

from cell_runtime_data_native_run import digest, pinned, reviewed_file_run, word
from cell_runtime_modifiers_native_run import request as consumer_request
from cell_schema_probe import SourceImage

ROOT = Path(__file__).resolve().parent.parent
CONTRIBUTION_HOST = ROOT/'target/source-cell/native-runtime-contributions-host-root-reproduction-v1'
CONSUMER_HOST = ROOT/'target/source-cell/native-runtime-modifiers-host-root-reproduction-v1'
CONTRIBUTION_SHA = 'd1f9d69e69882985cc6263a370187796c0f67f582059d85dfde60409e8966f49'
CONSUMER_SHA = '62885b4c906f0890a8c554f7bb289356d45535c85a9f8f9ac820979fbef75db0'


def save(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixture', type=Path, required=True)
    parser.add_argument('--review', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--run', action='store_true')
    args = parser.parse_args()
    args.out.resolve().relative_to((ROOT/'target').resolve())
    manifest = json.loads((args.fixture/'report.json').read_text())
    review = json.loads(args.review.read_text())
    assert manifest['cases'] == 25 and manifest['groups'] == 5
    for path, expected in review['pins'].items():
        assert digest(ROOT/path) == expected, path
    for name, expected in manifest['files'].items():
        assert digest(args.fixture/name) == expected, name
    for path, expected in manifest['source_pins'].items():
        assert digest(Path(path)) == expected, path
    assert digest(ROOT/'tools/cell_runtime_root_data_fixture.py') == manifest['tool_sha256']
    assert digest(CONTRIBUTION_HOST/'host-x86.exe') == CONTRIBUTION_SHA
    assert digest(CONSUMER_HOST/'host-x86.exe') == CONSUMER_SHA
    for host, expected in [(CONTRIBUTION_HOST, CONTRIBUTION_SHA), (CONSUMER_HOST, CONSUMER_SHA)]:
        assert json.loads((host/'linked-audit.json').read_text())['host_exe_sha256'] == expected
    source = Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe')
    SourceImage(source)  # Recheck the same immutable original executable identity.
    raw = (args.fixture/'contribution-input.bin').read_bytes()
    assert len(raw) == 25*1268
    for at in range(0, len(raw), 1268):
        record = raw[at:at+1268]
        assert word(record, 0) == 0x3F800000 and word(record, 4) == 0x027F
        assert word(record, 8) < 1 << 24 and word(record, 20) <= 32
        assert word(record, 24) == word(record, 28) == word(record, 32) == 0
        count = word(record, 20)
        assert record[244+32*count:] == bytes(32*(32-count))
        for index in range(count):
            desc = 244+32*index
            if (word(record, 8) & ~word(record, 12) & ~word(record, 16)) & (1 << index) and word(record, desc) & 15 == 3:
                assert word(record, desc) & 0x40 == 0 and 3 <= word(record, desc+16) <= 8
    args.out.mkdir(parents=True, exist_ok=False)
    save(args.out/'preflight.json', dict(schema='root-data-source-join-preflight-v1',
        cases=25, groups=5, contribution_input_sha256=digest(args.fixture/'contribution-input.bin'),
        reviewed_contribution_host=CONTRIBUTION_SHA, reviewed_consumer_host=CONSUMER_SHA,
        runner_sha256=digest(Path(__file__)), native_execution_requested=args.run))
    if not args.run:
        print(json.dumps(dict(preflight=True, cases=25, native_executed=False)))
        return
    paths = [source, Path(__file__), args.review, args.fixture/'report.json',
        args.out/'preflight.json', *[ROOT/path for path in review['pins']],
        *[args.fixture/name for name in manifest['files']],
        *[Path(path) for path in manifest['source_pins']],
        ROOT/'tools/cell_runtime_data_native_run.py', ROOT/'tools/cell_runtime_modifiers_native_run.py',
        ROOT/'tools/cell_schema_probe.py',
        *[path for host in [CONTRIBUTION_HOST, CONSUMER_HOST] for path in host.iterdir() if path.is_file()]]
    paths = list(dict.fromkeys(path.resolve() for path in paths))
    with pinned(paths, args.out):
        status, contributions = reviewed_file_run((CONTRIBUTION_HOST/'host-x86.exe').resolve(),
            args.fixture/'contribution-input.bin', args.out, 'contributions')
        assert status['exit'] == 0 and status['stderr_bytes'] == 0 and contributions.stat().st_size == 25*184, status
        consumer_input = args.out/'consumer-input.bin'
        with contributions.open('rb') as incoming, consumer_input.open('xb') as output:
            for _ in range(25):
                reply = incoming.read(184)
                assert word(reply, 172) == 1 and word(reply, 0) <= 8
                assert struct.unpack_from('<H', reply, 176)[0] == 0x027F
                assert struct.unpack_from('<H', reply, 178)[0] & 0x3800 == 0
                assert word(reply, 180) & ~0x3F == 0x1F80
                output.write(consumer_request(reply[:164]))
        status, consumers = reviewed_file_run((CONSUMER_HOST/'host-x86.exe').resolve(),
            consumer_input, args.out, 'consumers')
        assert status['exit'] == 0 and status['stderr_bytes'] == 0 and consumers.stat().st_size == 25*28, status
        with consumers.open('rb') as incoming:
            for _ in range(25):
                reply = incoming.read(28)
                assert reply[10:12] == bytes(2) and word(reply, 16) == 1
                assert struct.unpack_from('<H', reply, 20)[0] == 0x027F
                assert struct.unpack_from('<H', reply, 22)[0] & 0x3800 == 0
                assert word(reply, 24) & ~0x3F == 0x1F80
        result = dict(schema='original-source-root-data-kernel-join-v1', native_executed=True,
            cases=25, source_channels=5, source_component_counts=manifest['component_counts'],
            contribution_output=str(contributions.resolve()), contribution_sha256=digest(contributions),
            consumer_output=str(consumers.resolve()), consumer_sha256=digest(consumers),
            consumer_input_sha256=digest(consumer_input),
            scope='Accepted native curve replies -> original ordered scalar contribution -> original data2 consumer; explicit binding, supplied actor route and frame. Cohesive Rust parsing/evaluation remains the separate next gate. No clock or live actor routing claim.')
        save(args.out/'report.json', result)
        print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
