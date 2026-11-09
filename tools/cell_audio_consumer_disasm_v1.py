"""Dump bounded native audio-property consumer instructions from a PE image."""
import argparse, hashlib
from pathlib import Path
import pefile
import capstone

ap = argparse.ArgumentParser()
ap.add_argument('exe', type=Path)
ap.add_argument('output', type=Path)
a = ap.parse_args()
data = a.exe.read_bytes()
digest = hashlib.sha256(data).hexdigest()
if digest != '17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a':
    raise SystemExit(f'unexpected executable SHA-256: {digest}')
pe = pefile.PE(data=data, fast_load=True)
cs = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
lines = [f'path={a.exe}', f'sha256={digest}', f'image_base={pe.OPTIONAL_HEADER.ImageBase:08x}', f'capstone={capstone.__version__}', f'pefile={pefile.__version__}']
for start, end in ((0xA2D2C8, 0xA2D3DA), (0xA2038E, 0xA20407)):
    lines.append(f'\nrange={start:08x}..{end:08x} exclusive')
    code = pe.get_data(start-pe.OPTIONAL_HEADER.ImageBase, end-start)
    instructions = list(cs.disasm(code, start))
    consumed = sum(i.size for i in instructions)
    if len(code) != end - start or consumed != end - start:
        raise SystemExit(f'incomplete decode at {start:08x}: {consumed}/{end-start} bytes')
    for ins in instructions:
        lines.append(f'{ins.address:08x} {ins.bytes.hex():20} {ins.mnemonic} {ins.op_str}')
    lines.append(f'consumed={consumed}/{len(code)}')
with a.output.open('x', encoding='utf-8', newline='\n') as output:
    output.write('\n'.join(lines)+'\n')
