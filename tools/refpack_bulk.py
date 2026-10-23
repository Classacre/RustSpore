"""Bulk back-reference copy for local corpus diagnostics, same RefPack grammar.

The production Rust decoder and existing Python command parser are unchanged.
Only Python's bytewise overlap-copy inner loop is replaced after equivalence
checks, so independent format probes need not spend minutes copying zero runs.
"""
import refpack


def copy(out, dist, count, limit):
    if dist <= 0 or dist > len(out):
        raise refpack.RefPackError(f'invalid copy distance {dist} at output offset {len(out)}')
    count = min(count, max(0, limit-len(out)))
    if count <= 0:
        return
    if dist >= count:
        start = len(out)-dist
        out.extend(out[start:start+count])
    else:
        pattern = out[-dist:]
        out.extend((pattern*((count+dist-1)//dist))[:count])


def install():
    original = refpack._copy
    if original is copy:
        return
    # Exercise overlap, exact distance, limits before and inside output, and
    # varying bytes rather than testing zero-filled fixtures alone.
    for dist in range(1, 257):
        source = bytearray((i*37+dist*11) & 255 for i in range(300))
        for length in (0, 1, 2, 3, 4, 7, 16, 63, 256, 1028):
            for remaining in (0, 1, 5, 30, 1028):
                a, b = source.copy(), source.copy()
                original(a, dist, length, len(a)+remaining)
                copy(b, dist, length, len(b)+remaining)
                if a != b:
                    raise AssertionError((dist, length, remaining))
    refpack._copy = copy


if __name__ == '__main__':
    install()
    print('12800 bulk/bytewise overlap and limit cases agree')
