"""Compare Rust selection to an independently expressed 987A00 branch model.

Explicit state and null/non-null slot masks only. No native execution or
claim about a runtime drawable state or rendering.
"""
import argparse
from pathlib import Path
import subprocess


def reference(state, mask):
    def slot(i):
        return i if mask & (1 << i) else None
    mode = state & 7
    if mode > 3:
        return slot(0)
    if mode == 0:
        return (slot(4) if state & 8 else None) or slot(0)
    if state & 8:
        special = slot(mode + 4)
        if special is not None:
            return special
        if mode in (2, 3) and slot(4) is not None:
            return 4
    ordinary = slot(mode)
    return ordinary if ordinary is not None else slot(0)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('driver', type=Path)
    args = parser.parse_args()
    output = subprocess.run([str(args.driver.resolve())], check=True, text=True, capture_output=True).stdout.splitlines()
    expected = [(state, mask) for state in list(range(16)) + [x | 0xfffffff0 for x in range(16)] for mask in range(256)]
    if len(output) != len(expected):
        raise ValueError('wrong row count')
    for line, (state, mask) in zip(output, expected):
        selected = reference(state, mask)
        target = f'{state:08X}\t{mask:02X}\t' + ('-' if selected is None else str(selected))
        if line != target:
            raise ValueError(f'expected {target}, got {line}')
    print(f'{len(output)} explicit state/mask rows match; no native execution')


if __name__ == '__main__':
    main()
