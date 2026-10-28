"""Pin frozen v3; add positive-blend admission and safe owned failure exit.

No compile/link/run. Original code/data, native entries/exits, buffers and
ordinary computation are unchanged. The fixed linked base stays29000000.
"""
import argparse
import hashlib
import json
import re
from pathlib import Path

import anim_root_pose_native_probe as v3
from cell_schema_probe import SourceImage
from native_crt_acos_probe import PE,decoder

ROOT = Path(__file__).resolve().parent.parent
FROZEN = ROOT/'target/animation/root-pose-conversion-host-v3'
TOOL = '212980f1b37a332b1f668d6f7f7a1a435d1fa07af20a814c5c62eb34dcf78603'
ASM = 'b60a73629d2984031816292580023976eb64491ca5ab5477e75eb950e0d3d331'
CODE,BUFFERS,IMPORTS,BASE = v3.CODE,v3.BUFFERS,v3.IMPORTS,v3.BASE


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def host_source(image,audit):
    assert sha(v3.__file__)==TOOL and sha(FROZEN/'host-x86.asm')==ASM
    original = v3.host_source(image,audit)
    assert original==(FROZEN/'host-x86.asm').read_text()
    lines = original.splitlines()
    lines.insert(lines.index('.code'),'PUBLIC SavedStack,FailureStackReady')
    index = lines.index('HostFailure PROC')+1
    failure = ['cmp SavedStack,0','je FailureStackReady','mov esp,SavedStack','FailureStackReady:']
    lines[index:index]=failure
    index = lines.index('ValidateRecord ENDP')
    assert lines[index-1]=='ret'
    positive = ['mov eax,dword ptr [InputBuffer+16]','test eax,080000000h','jnz HostFailure',
                'test eax,07FFFFFFFh','jz HostFailure']
    lines[index-1:index-1]=positive
    # Inverse edit proves unchanged source bytes, buffer declarations and
    # ordinary owned instructions without rewriting any frozen artifact.
    inverse=list(lines)
    start=inverse.index('cmp SavedStack,0');assert inverse[start:start+len(failure)]==failure
    del inverse[start:start+len(failure)]
    start=inverse.index('ValidateRecord ENDP')-1-len(positive)
    assert inverse[start:start+len(positive)]==positive
    del inverse[start:start+len(positive)]
    inverse.remove('PUBLIC SavedStack,FailureStackReady')
    assert inverse==original.splitlines()
    return '\n'.join(lines)+'\n'


def _base_linked_audit(path,mapping,image,audit):
    from capstone import CS_GRP_CALL,CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM,X86_OP_REG,X86_REG_EAX
    pe = PE(path);symbols = {}
    for line in mapping.read_text().splitlines():
        match = re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]+\s+(\S+)\s+([0-9A-Fa-f]{8})\b',line)
        if match:symbols[match[1]] = int(match[2],16)
    assert pe.base == BASE
    first,last = symbols['OwnedStart'],symbols['OwnedEnd']
    rows = list(decoder().disasm(pe.read(first,last-first),first))
    assert sum(row.size for row in rows)==last-first
    boundaries = {row.address for row in rows};imports = pe.imports()
    assert len(imports)==8
    assert {name for dll,name in imports.values() if dll.lower()=='kernel32.dll'}==IMPORTS
    def imported(row):
        if row.mnemonic!='call' or row.operands[0].type!=X86_OP_IMM:return None
        target = row.operands[0].imm
        if target in boundaries:return None
        thunk = pe.read(target,6);assert thunk[:2]==bytes.fromhex('ff25')
        return imports[struct.unpack_from('<I',thunk,2)[0]][1]
    entries = []
    for index,row in enumerate(rows):
        if not(row.group(CS_GRP_CALL) or row.group(CS_GRP_JUMP)):continue
        op = row.operands[0]
        if op.type==X86_OP_REG:
            assert op.reg==X86_REG_EAX and index>0
            previous = rows[index-1]
            assert previous.mnemonic=='mov' and previous.operands[1].type==X86_OP_IMM
            target = previous.operands[1].imm
            assert (row.mnemonic,target) in (('call',0x9b0ca0),('jmp',0x99d74f))
            entries.append(dict(at=f'{row.address:08X}',operation=row.mnemonic,target=f'{target:08X}'))
        else:
            assert op.type==X86_OP_IMM
            assert op.imm in boundaries or imported(row) in IMPORTS
    assert len(entries)==2
    at = struct.unpack_from('<I',pe.data,0x3c)[0];optional=at+24
    assert pe.base+struct.unpack_from('<I',pe.data,optional+16)[0]==symbols['HostMain']
    assert not struct.unpack_from('<H',pe.data,optional+70)[0]&0x40
    count = struct.unpack_from('<H',pe.data,at+6)[0];size = struct.unpack_from('<H',pe.data,at+20)[0]
    sections=[]
    for i in range(count):
        header=optional+size+40*i;extent,rva=struct.unpack_from('<II',pe.data,header+8)
        sections.append((pe.base+rva,pe.base+rva+extent,struct.unpack_from('<I',pe.data,header+36)[0]))
    def permission(address,n):
        return next(flags for lo,hi,flags in sections if lo<=address and address+n<=hi)&0xe0000000
    assert permission(first,last-first)==0x60000000
    for a,b in CODE:
        linked = symbols[f'Original_{a:08X}']
        assert pe.read(linked,b-a)==image.read(a,b-a) and permission(linked,b-a)==0x40000000
    for row in audit['data']:
        a,n = int(row['address'],16),row['bytes'];linked=symbols[f'Data_{a:08X}']
        assert pe.read(linked,n)==image.read(a,n) and permission(linked,n)==0x40000000
    extents=[]
    def add_extent(name,low,n):
        assert permission(low,n)==0xc0000000
        assert all(low+n<=lo or hi<=low for _,lo,hi in extents)
        extents.append((name,low,low+n))
    for name,n in BUFFERS:
        address=symbols[name+'Buffer']
        assert address-symbols[name+'Before']==16 and symbols[name+'After']-address==n
        assert pe.read(address-16,16)==bytes.fromhex('a5'*16)
        assert pe.read(address+n,16)==bytes.fromhex('a5'*16)
        add_extent(name,address-16,n+32)
        if name not in ('Converted','Reply'):add_extent(name+'Shadow',symbols[name+'Shadow'],n)
    first_read = next(row.address for row in rows if imported(row)=='ReadFile')
    incoming=[]
    for index,row in enumerate(rows):
        if not(row.group(CS_GRP_CALL) or row.group(CS_GRP_JUMP)):continue
        if row.operands[0].type!=X86_OP_IMM or row.operands[0].imm!=symbols['MappingFailure']:continue
        assert symbols['HostMain']<row.address<first_read and index>=2
        name=imported(rows[index-2]);assert name in ('VirtualAlloc','VirtualProtect')
        assert (name=='VirtualAlloc' and rows[index-1].mnemonic=='cmp' and row.mnemonic in ('jne','jnz')) or \
               (name=='VirtualProtect' and rows[index-1].bytes==bytes.fromhex('85c0') and row.mnemonic in ('je','jz'))
        incoming.append(dict(at=f'{row.address:08X}',api=name))
    assert incoming and sum(x['api']=='VirtualAlloc' for x in incoming)==sum(x['api']=='VirtualProtect' for x in incoming)
    for label,code in [('MappingFailure',10)]:
        index=next(i for i,row in enumerate(rows) if row.address==symbols[label])
        assert rows[index].mnemonic=='push' and rows[index].operands[0].imm==code
        assert imported(rows[index+1])=='ExitProcess'
    return dict(schema='root-POS-ROT-conversion-accumulation-linked-audit-v1',native_executed=False,
        host_exe_sha256=sha(path),host_map_sha256=sha(mapping),
        owned_code_sha256=hashlib.sha256(pe.read(first,last-first)).hexdigest(),
        source_and_data_byte_exact=True,sections_checked=True,guarded_buffers_shadows_nonoverlapping=True,
        partial_native_entries=entries,mapping_failure_pre_input=incoming,
        imports=[dict(iat=f'{a:08X}',dll=dll,name=name) for a,(dll,name) in imports.items()],
        symbols={k:f'{v:08X}' for k,v in symbols.items() if not k.startswith('_')},
        disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in rows],
        limits='Partial source entry/exit with checkedESP; no partialGPRABI promise or staged/livepose claim.')



def linked_audit(exe,mapping,image,audit):
    result=_base_linked_audit(exe,mapping,image,audit)
    pe=PE(exe);symbols={k:int(v,16) for k,v in result['symbols'].items()}
    saved=symbols['SavedStack'];failure=symbols['HostFailure']
    assert pe.read(saved,4)==bytes(4)
    rows=list(decoder().disasm(pe.read(failure,40),failure))
    assert rows[0].mnemonic=='cmp' and rows[0].operands[0].mem.disp==saved
    assert rows[0].operands[1].imm==0
    assert rows[1].mnemonic in ('je','jz') and rows[1].operands[0].imm==symbols['FailureStackReady']
    assert rows[2].mnemonic=='mov' and rows[2].op_str==f'esp, dword ptr [0x{saved:x}]'
    assert rows[3].address==symbols['FailureStackReady'] and rows[3].mnemonic=='push' and rows[3].operands[0].imm==71
    owned_start=symbols['OwnedStart'];owned_end=symbols['OwnedEnd']
    allrows=list(decoder().disasm(pe.read(owned_start,owned_end-owned_start),owned_start))
    epilogue=next(i for i,r in enumerate(allrows) if r.address==symbols['RunOnce'])
    prefix=allrows[:epilogue]
    assert prefix[-1].mnemonic=='ret'
    assert [r.mnemonic for r in prefix[-6:-1]]==['mov','test','jne','test','je']
    result.update(schema='root-POS-ROT-conversion-accumulation-linked-audit-v4',
        owned_failure_restores_saved_stack=True,positive_finite_blend_only=True,
        no_original_code_data_or_buffer_change=True)
    return result


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    p.add_argument('--out',type=Path,required=True)
    p.add_argument('--audit-exe',type=Path);p.add_argument('--audit-map',type=Path)
    a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve())
    image=SourceImage(a.source_exe);audit=v3.source_audit(image);text=host_source(image,audit)
    if a.audit_exe is not None:
        assert a.audit_map is not None
        manifest=json.loads((a.out/'manifest.json').read_text())
        assert sha(__file__)==manifest['preparation_tool_sha256']
        assert (a.out/'host-x86.asm').read_text()==text and sha(a.out/'host-x86.asm')==manifest['host_source_sha256']
        report=linked_audit(a.audit_exe,a.audit_map,image,audit)
        with (a.out/'linked-audit.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
        print(json.dumps({k:report[k] for k in('schema','host_exe_sha256','native_executed')},indent=2));return
    a.out.mkdir(parents=True,exist_ok=False);(a.out/'host-x86.asm').write_text(text)
    manifest=json.loads((FROZEN/'manifest.json').read_text())
    manifest.update(schema='root-POS-ROT-conversion-accumulation-host-preparation-v4',
        host_source_sha256=sha(a.out/'host-x86.asm'),preparation_tool_sha256=sha(__file__),
        frozen_v3_pins={str(p.relative_to(ROOT)):sha(p) for p in(Path(v3.__file__),FROZEN/'host-x86.asm',FROZEN/'manifest.json')},
        owned_instruction_changes=['Conditional restoration ofSavedStack beforeHostFailure exit71.',
            'Reject finite nonpositive blend atownedValidateRecord boundary beforeoriginalpartialentry.'],
        domain=manifest['domain']+' V4requirespositivefiniteblend;negative/zero inputsaretypedprotocolrejections,notwholecallerexamples.',
        linked_audit_command='python tools/anim_root_pose_native_probe_v4.py --out HOST_DIR --audit-exe HOST_DIR/host-x86.exe --audit-map HOST_DIR/host-x86.map')
    (a.out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({k:manifest[k] for k in('schema','host_source_sha256','compiled','linked','native_executed')},indent=2))


if __name__=='__main__':main()
