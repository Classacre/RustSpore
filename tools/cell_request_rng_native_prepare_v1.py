"""Prepare a fixed-address x86 RNG host; never execute or compile it here."""
import argparse, hashlib, json, struct, sys
from pathlib import Path
from cell_schema_probe import SourceImage, SOURCE_SHA256

ROOT = Path(__file__).resolve().parent.parent
START, END = 0x935D40, 0x935D76
CONSTANTS = {0x143F52C: 0x2F800000, 0x1471B2C: 0x3F000000}

def main():
    p = argparse.ArgumentParser()
    p.add_argument('exe', type=Path)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    out = a.out.resolve()
    out.relative_to((ROOT / 'target').resolve())
    image = SourceImage(a.exe)
    sys.path.insert(0, str(ROOT / 'target/rea/capstone-runtime'))
    from capstone import Cs, CS_ARCH_X86, CS_MODE_32, CS_GRP_JUMP, CS_GRP_CALL
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    d = Cs(CS_ARCH_X86, CS_MODE_32); d.detail = True
    code = image.read(START, END - START)
    ins = list(d.disasm(code, START))
    assert sum(i.size for i in ins) == len(code) and ins[-1].address == 0x935D75 and ins[-1].mnemonic == 'ret'
    branches, absolute = [], []
    for i in ins:
        assert not i.group(CS_GRP_CALL)
        if i.group(CS_GRP_JUMP):
            assert i.operands[0].type == X86_OP_IMM
            target = i.operands[0].imm
            assert START <= target < END and target in [x.address for x in ins]
            branches.append((i.address, target))
        for op in i.operands:
            if op.type == X86_OP_MEM and op.mem.base == 0 and op.mem.index == 0:
                absolute.append(op.mem.disp & 0xffffffff)
    assert branches == [(0x935D44, 0x935D4A), (0x935D6E, 0x935D74)]
    assert absolute == list(CONSTANTS)
    for addr, bits in CONSTANTS.items():
        assert struct.unpack('<I', image.read(addr, 4))[0] == bits
    assert [i.mnemonic for i in ins].count('rdtsc') == 1
    out.mkdir(parents=True, exist_ok=False)
    def array(b): return ','.join('0x%02x' % v for v in b)
    c = r'''#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <fcntl.h>
#include <io.h>
static const unsigned char code[] = {CODE};
static const unsigned char c1[] = {CONST1};
static const unsigned char c2[] = {CONST2};
static void map(uintptr_t page, DWORD flags) {
 if (VirtualAlloc((void*)page, 0x10000, MEM_RESERVE|MEM_COMMIT, flags)!=(void*)page) ExitProcess(10);
}
int main(void) {
 uint32_t row[2], state; unsigned short oldcw, cw; double value;
 _setmode(_fileno(stdin), _O_BINARY); _setmode(_fileno(stdout), _O_BINARY);
 map(0x930000, PAGE_EXECUTE_READWRITE); map(0x1430000, PAGE_READWRITE); map(0x1470000, PAGE_READWRITE);
 memcpy((void*)0x935d40, code, sizeof code); memcpy((void*)0x143f52c,c1,4); memcpy((void*)0x1471b2c,c2,4);
 FlushInstructionCache(GetCurrentProcess(), (void*)0x935d40, sizeof code);
 for (;;) {
  size_t n=fread(row,1,8,stdin); if (!n) return ferror(stdin)?11:0; if(n!=8) return 12;
  state=row[0]; cw=(unsigned short)row[1];
  if(!state || row[1]!=cw || (cw & ~0x0f00)!=0x7f || ((cw & 0x300)!=0x200 && (cw & 0x300)!=0x300)) return 13;
  __asm { fnstcw oldcw }
  __asm { fldcw cw }
  __asm { lea ecx, state }
  __asm { mov eax, 0x935d40 }
  __asm { call eax }
  __asm { fstp qword ptr value }
  __asm { fldcw oldcw }
  if(fwrite(&state,1,4,stdout)!=4 || fwrite(&value,1,8,stdout)!=8) return 14;
 }
}'''.replace('CODE',array(code)).replace('CONST1',array(image.read(0x143F52C,4))).replace('CONST2',array(image.read(0x1471B2C,4)))
    c = c.replace('#include <stdio.h>', '#include <stdio.h>\n#include <string.h>')
    (out/'host.c').write_bytes(c.encode('utf-8'))
    manifest = dict(source_sha256=SOURCE_SHA256, code_start=hex(START), code_end_exclusive=hex(END), code_sha256=hashlib.sha256(code).hexdigest(), branches=[[hex(x),hex(y)] for x,y in branches], absolute_operands=[hex(x) for x in absolute], disassembly=[f'{i.address:08X} {i.mnemonic} {i.op_str}' for i in ins], host_c_sha256=hashlib.sha256((out/'host.c').read_bytes()).hexdigest(), executed=False)
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({k:v for k,v in manifest.items() if k != 'disassembly'}))
if __name__ == '__main__': main()
