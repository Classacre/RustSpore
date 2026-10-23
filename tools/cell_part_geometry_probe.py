"""Prepare, never execute, the bounded original E83E60 geometry fixture host.

Creature rows and the classification result are explicit protocol inputs. The
loader and classifier are checked owned stubs; no actual game manager, parent
resolution, initializer, retail process, window or network is invoked.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

from cell_native_math_probe import IMPORTS,db
from cell_schema_probe import SOURCE_SHA256,SourceImage

CODE=(0xe83e60,0xe8417a)
LOADER=0x4bc760
CLASSIFIER=0xe827a0
CONSTANTS={0x1471b2c:0x3f000000,0x14861d0:0x3f800000,0x1486bc4:0x40200000}
INPUT_BYTES=224
OUTPUT_BYTES=160


def audit(image):
    sys.path.insert(0,str(Path(__file__).resolve().parent.parent/'target/rea/capstone-runtime'))
    from capstone import Cs,CS_ARCH_X86,CS_MODE_32,CS_GRP_CALL,CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM,X86_OP_MEM,X86_OP_REG
    decoder=Cs(CS_ARCH_X86,CS_MODE_32);decoder.detail=True
    start,end=CODE;rows=list(decoder.disasm(image.read(start,end-start),start))
    assert sum(row.size for row in rows)==end-start
    calls=[];indirect=[];memory=[]
    absolutes=set(CONSTANTS)|{0x16b54e4,0x16b54e8,0x16b54ec}
    for ins in rows:
        if ins.group(CS_GRP_CALL)or ins.group(CS_GRP_JUMP):
            operand=ins.operands[0]
            if operand.type==X86_OP_IMM:
                target=operand.imm
                if ins.group(CS_GRP_CALL):
                    assert target in {LOADER,CLASSIFIER}
                    calls.append(dict(at=f'{ins.address:08X}',target=f'{target:08X}'))
                else:assert start<=target<end
            else:
                assert ins.address==0xe84172 and ins.group(CS_GRP_CALL)and operand.type==X86_OP_REG
                indirect.append(dict(at=f'{ins.address:08X}',operand=ins.op_str))
        for operand in ins.operands:
            if operand.type!=X86_OP_MEM:continue
            if operand.mem.base==0:
                assert operand.mem.index==0 and operand.mem.disp in absolutes
            memory.append(ins.op_str)
    assert calls==[dict(at='00E83EEF',target='004BC760'),dict(at='00E83F5D',target='00E827A0')]
    assert len(indirect)==1
    for address,value in CONSTANTS.items():assert image.read(address,4)==struct.pack('<I',value)
    assert any(ins.address==0xe83eb5 and ins.op_str=='esi, 0x16b5534'for ins in rows)
    return dict(start=f'{start:08X}',end_exclusive=f'{end:08X}',bytes=end-start,
        sha256=hashlib.sha256(image.read(start,end-start)).hexdigest(),direct_calls=calls,
        checked_vtable_calls=indirect,absolute_memory_operands=sorted(f'{a:08X}'for a in absolutes),
        explicit_special_matrix_pointer='016B5534,9dwords',memory_operands=sorted(set(memory)),
        disassembly=[f'{ins.address:08X}: {ins.mnemonic} {ins.op_str}'for ins in rows])


def source(image):
    start,end=CODE
    lines=['.686p','.xmm','.model flat','option casemap:none',
        'EXTERN _GetStdHandle@4:PROC','EXTERN _ReadFile@20:PROC','EXTERN _WriteFile@20:PROC',
        'EXTERN _VirtualAlloc@16:PROC','EXTERN _GetLastError@0:PROC',
        'EXTERN _FlushInstructionCache@12:PROC','EXTERN _ExitProcess@4:PROC',
        '.const','PUBLIC CreatureVtable','CreatureVtable DWORD 0,OFFSET CreatureRelease',
        '.data','InputBefore DWORD 4 DUP(0A5A5A5A5h)','InputBuffer BYTE 224 DUP(0)',
        'InputAfter DWORD 4 DUP(0A5A5A5A5h)',
        'CreatureBefore DWORD 4 DUP(0A5A5A5A5h)','FakeCreature BYTE 160 DUP(0)',
        'CreatureAfter DWORD 4 DUP(0A5A5A5A5h)',
        'RowsBefore DWORD 4 DUP(0A5A5A5A5h)','SourceRows BYTE 35840 DUP(0)',
        'RowsAfter DWORD 4 DUP(0A5A5A5A5h)',
        'PartsBefore DWORD 4 DUP(0A5A5A5A5h)','NativeParts BYTE 4608 DUP(0)',
        'PartsAfter DWORD 4 DUP(0A5A5A5A5h)',
        'OutputBefore DWORD 4 DUP(0A5A5A5A5h)','ReplyBuffer BYTE 160 DUP(0)',
        'OutputAfter DWORD 4 DUP(0A5A5A5A5h)',
        'InputHandle DWORD 0','OutputHandle DWORD 0','Transferred DWORD 0',
        'NativeStack DWORD 0','ExpectedRef DWORD 0','ExpectedCount DWORD 0',
        'NativeKey DWORD 3 DUP(0)','SseControl DWORD 01F80h']
    lines+=db('NativeCode',image.read(start,end-start))
    for address in CONSTANTS:lines+=db(f'Constant_{address:08X}',image.read(address,4))
    lines+=['.code','PUBLIC HostMain,LoadSource,ClassProvider,CreatureRelease,HostFailure,LoaderProvider',
        'LoadSource PROC','cmp dword ptr [ReplyBuffer+4],0','jne HostFailure',
        'inc dword ptr [ReplyBuffer+4]','cmp dword ptr [esp+4],OFFSET NativeKey','jne HostFailure',
        'mov eax,dword ptr [esp+8]','cmp eax,ExpectedRef','jne HostFailure',
        'cmp dword ptr [eax],0','jne HostFailure','mov dword ptr [eax],OFFSET FakeCreature',
        'mov eax,OFFSET FakeCreature','ret','LoadSource ENDP',
        'ClassProvider PROC','mov eax,dword ptr [ReplyBuffer+8]',
        'cmp eax,ExpectedCount','jae HostFailure','imul eax,eax,140','add eax,OFFSET SourceRows',
        'cmp eax,dword ptr [esp+4]','jne HostFailure','inc dword ptr [ReplyBuffer+8]',
        'mov eax,dword ptr [InputBuffer+4]','ret','ClassProvider ENDP',
        'CreatureRelease PROC','cmp ecx,OFFSET FakeCreature','jne HostFailure',
        'cmp dword ptr [ReplyBuffer+12],0','jne HostFailure',
        'inc dword ptr [ReplyBuffer+12]','xor eax,eax','ret','CreatureRelease ENDP',
        'ReadRecord PROC','push ebx','push esi','mov ebx,224','mov esi,OFFSET InputBuffer',
        'ReadLoop:','mov Transferred,0','push 0','push OFFSET Transferred','push ebx',
        'push esi','push InputHandle','call _ReadFile@20','test eax,eax','jnz ReadTransferred',
        'call _GetLastError@0','cmp eax,109','jne HostFailure','cmp ebx,224','jne HostFailure',
        'cmp Transferred,0','jne HostFailure','xor eax,eax','jmp ReadDone','ReadTransferred:',
        'mov eax,Transferred','test eax,eax','jnz ReadSome','cmp ebx,224','jne HostFailure',
        'xor eax,eax','jmp ReadDone','ReadSome:','add esi,eax','sub ebx,eax','jnz ReadLoop',
        'mov eax,1','ReadDone:','pop esi','pop ebx','ret','ReadRecord ENDP',
        'HostMain PROC']
    pages={0xe80000,0x1470000,0x1480000,0x16b0000}
    for ordinal,page in enumerate(sorted(pages),10):
        lines+=[f'push {"040h" if page==0xe80000 else "04h"}','push 03000h',
            'push 010000h',f'push 0{page:08X}h','call _VirtualAlloc@16',
            f'cmp eax,0{page:08X}h',f'jne AllocationFailure{ordinal}']
    lines+=['mov edi,00E80000h','mov ecx,010000h','mov al,0CCh','cld','rep stosb',
        'mov esi,OFFSET NativeCode',f'mov edi,0{start:08X}h',f'mov ecx,{end-start}','rep movsb',
        # Copied classifier trampoline has only absolute transfers, no relocated
        # outbound relative branch to the linked host image.
        'mov byte ptr [00E827A0h],0B8h','mov dword ptr [00E827A1h],OFFSET ClassProvider',
        'mov word ptr [00E827A5h],0E0FFh']
    for address in CONSTANTS:
        lines+=[f'mov esi,OFFSET Constant_{address:08X}',f'mov edi,0{address:08X}h','mov ecx,4','rep movsb']
    lines+=['push 0','push 0','push -1','call _FlushInstructionCache@12',
        'test eax,eax','jz CacheFlushFailure',
        'push -10','call _GetStdHandle@4','mov InputHandle,eax',
        'push -11','call _GetStdHandle@4','mov OutputHandle,eax',
        'NextRecord:','call ReadRecord','test eax,eax','jz HostSuccess',
        'cmp dword ptr [InputBuffer],256','ja HostFailure',
        'movzx eax,word ptr [InputBuffer+20]','mov edx,eax','and edx,0F0FFh',
        'cmp edx,07Fh','jne HostFailure','and eax,0300h','cmp eax,0100h','je HostFailure',
        'cmp dword ptr [InputBuffer+12],0476A98C7h','je ClassReady',
        'cmp dword ptr [InputBuffer],0','je ClassReady',
        'mov eax,dword ptr [InputBuffer+4]','dec eax','cmp eax,11','ja HostFailure','ClassReady:',
        'mov edi,OFFSET ReplyBuffer','xor eax,eax','mov ecx,40','rep stosd',
        'mov edi,OFFSET NativeParts','mov eax,0A5A5A5A5h','mov ecx,1152','rep stosd',
        'mov edi,OFFSET SourceRows','mov ecx,8960','rep stosd',
        'mov edi,OFFSET SourceRows','mov ebx,dword ptr [InputBuffer]',
        'test ebx,ebx','jz RowsReady','CopyRow:','mov esi,OFFSET InputBuffer+32',
        'mov ecx,35','rep movsd','dec ebx','jnz CopyRow','RowsReady:',
        'mov dword ptr [FakeCreature],OFFSET CreatureVtable',
        'mov dword ptr [FakeCreature+152],OFFSET SourceRows',
        'imul eax,dword ptr [InputBuffer],140','add eax,OFFSET SourceRows',
        'mov dword ptr [FakeCreature+156],eax',
        'mov esi,OFFSET InputBuffer+8','mov edi,OFFSET NativeKey','mov ecx,3','rep movsd',
        'mov esi,OFFSET InputBuffer+172','mov edi,016B54E4h','mov ecx,3','rep movsd',
        'mov esi,OFFSET InputBuffer+184','mov edi,016B5534h','mov ecx,9','rep movsd',
        'mov eax,dword ptr [InputBuffer]','cmp eax,64','jbe CountReady','mov eax,64',
        'CountReady:','mov ExpectedCount,eax',
        'fninit','fldcw word ptr [InputBuffer+20]','ldmxcsr SseControl',
        'mov NativeStack,esp','and esp,0FFFFFFF0h','sub esp,4',
        'push OFFSET NativeKey','push OFFSET NativeParts','push OFFSET ReplyBuffer',
        'mov eax,esp','sub eax,48','mov ExpectedRef,eax',
        'mov eax,00E83E60h','call eax','mov esp,NativeStack',
        'mov eax,ExpectedCount','mov edx,1',
        'cmp dword ptr [InputBuffer+12],0476A98C7h','jne OrdinaryCounts',
        'cmp dword ptr [ReplyBuffer],1','jne HostFailure','xor edx,edx','xor eax,eax',
        'jmp CheckCalls','OrdinaryCounts:','cmp dword ptr [ReplyBuffer],eax','jne HostFailure',
        'CheckCalls:','cmp dword ptr [ReplyBuffer+4],edx','jne HostFailure',
        'cmp dword ptr [ReplyBuffer+8],eax','jne HostFailure',
        'cmp dword ptr [ReplyBuffer+12],edx','jne HostFailure',
        # All defined source output rows preserve their three padding bytes.
        'mov ecx,dword ptr [ReplyBuffer]','mov esi,OFFSET NativeParts',
        'test ecx,ecx','jz CheckUnwritten','CheckPadding:',
        'cmp byte ptr [esi+1],0A5h','jne HostFailure','cmp word ptr [esi+2],0A5A5h','jne HostFailure',
        'add esi,72','dec ecx','jnz CheckPadding','CheckUnwritten:',
        'mov edx,OFFSET NativeParts+4608','CheckUnwrittenLoop:',
        'cmp esi,edx','jae CopyReply','cmp byte ptr [esi],0A5h','jne HostFailure',
        'inc esi','jmp CheckUnwrittenLoop','CopyReply:',
        'mov esi,OFFSET NativeParts','mov edi,OFFSET ReplyBuffer+16','mov ecx,18','rep movsd',
        'mov eax,dword ptr [ReplyBuffer]','test eax,eax','jz LastReady','dec eax',
        'LastReady:','imul eax,eax,72','lea esi,[NativeParts+eax]',
        'mov edi,OFFSET ReplyBuffer+88','mov ecx,18','rep movsd']
    for label in ['InputBefore','InputAfter','CreatureBefore','CreatureAfter','RowsBefore','RowsAfter',
                  'PartsBefore','PartsAfter','OutputBefore','OutputAfter']:
        for offset in range(0,16,4):
            lines+=[f'cmp dword ptr [{label}+{offset}],0A5A5A5A5h','jne HostFailure']
    lines+=['push 0','push OFFSET Transferred','push 160','push OFFSET ReplyBuffer',
        'push OutputHandle','call _WriteFile@20','test eax,eax','jz HostFailure',
        'cmp Transferred,160','jne HostFailure','jmp NextRecord',
        'HostSuccess:','push 0','call _ExitProcess@4']
    for ordinal in range(10,14):
        lines+=[f'AllocationFailure{ordinal}:',f'push {ordinal}','call _ExitProcess@4']
    lines+=['CacheFlushFailure:','push 20','call _ExitProcess@4',
        'HostFailure::','push 7','call _ExitProcess@4','HostMain ENDP',
        # .text RVA1000 plus B760 reaches fixed loader entry004BC760. The
        # image is reserved by Windows before heap/stack; no native relocation.
        'ORG 0B760h','LoaderProvider PROC','mov eax,OFFSET LoadSource','jmp eax',
        'LoaderProvider ENDP','END HostMain']
    return '\n'.join(lines)+'\n',pages


def linked_audit(exe,mapping):
    wanted={'CreatureVtable','ClassProvider','CreatureRelease','LoadSource','LoaderProvider','HostFailure'}
    symbols={}
    for line in mapping.read_text().splitlines():
        match=re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\w+)\s+([0-9A-Fa-f]{8})\b',line)
        if match and match[1]in wanted:symbols[match[1]]=int(match[2],16)
    assert set(symbols)==wanted,symbols
    raw=exe.read_bytes();pe=struct.unpack_from('<I',raw,60)[0]
    base=struct.unpack_from('<I',raw,pe+52)[0]
    count=struct.unpack_from('<H',raw,pe+6)[0];optional=struct.unpack_from('<H',raw,pe+20)[0]
    sections=[]
    for index in range(count):
        at=pe+24+optional+40*index
        _,rva,size,offset=struct.unpack_from('<4I',raw,at+8)
        flags=struct.unpack_from('<I',raw,at+36)[0]
        sections.append((base+rva,size,offset,flags))
    def read(address,length):
        for start,size,offset,flags in sections:
            if start<=address and address+length<=start+size:
                at=offset+address-start
                return raw[at:at+length],flags
        raise ValueError(f'unmapped_host_address{address:08X}')
    assert base==0x4b0000 and symbols['LoaderProvider']==LOADER
    stub=b'\xb8'+struct.pack('<I',symbols['LoadSource'])+b'\xff\xe0'
    actual,flags=read(LOADER,7)
    assert actual==stub and flags&0x60000000==0x60000000 and not flags&0x80000000
    vtable,flags=read(symbols['CreatureVtable'],8)
    assert flags&0x40000000 and not flags&0x80000000
    assert struct.unpack_from('<I',vtable,4)[0]==symbols['CreatureRelease']
    return dict(symbols={name:f'{address:08X}'for name,address in symbols.items()},
        loader_entry=f'{LOADER:08X}',loader_stub_hex=stub.hex(),
        loader_compiled_into_reserved_host_RX_image=True,
        classifier_entry=f'{CLASSIFIER:08X}',
        classifier_stub_hex=(b'\xb8'+struct.pack('<I',symbols['ClassProvider'])+b'\xff\xe0').hex(),
        outbound_relative_transfers_in_fixed_stubs=0,immutable_vtable=True,
        vtable_call_proof='Only native indirect callE84172 uses ECX returned from checked loader. That loader writes only FakeCreature to the exact native local reference slot; vtable+4 points to checked CreatureRelease.',
        bounds_proof='Input provides no pointers. Loader validates exact NativeKey pointer and zero-initialized reference-stack-slot address. FakeCreature has SourceRows begin/end for0..256copies of one140-byte input row. ClassProvider validates each monotonically ordered row pointer and count<min(sourcecount,64). Native output is64*72guarded bytes; host verifies3padding bytes per row and every unwritten suffix byte. All source/output/object buffers have16-byte guards. Special globals are explicit12+36input bytes at original data addresses.')


def main():
    ap=argparse.ArgumentParser();ap.add_argument('exe',type=Path)
    ap.add_argument('--sdk-bin-x86',type=Path,required=True)
    ap.add_argument('--kernel32-lib',type=Path,required=True)
    ap.add_argument('--out',type=Path,required=True);args=ap.parse_args()
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    image=SourceImage(args.exe);native=audit(image);assembly,pages=source(image)
    args.out.mkdir(parents=True,exist_ok=False);out=args.out.resolve()
    asm=out/'host-x86.asm';obj=out/'host-x86.obj';exe=out/'host-x86.exe';mapping=out/'host-x86.map'
    asm.write_text(assembly)
    for command in [
        [str(args.sdk_bin_x86/'ml.exe'),'/nologo','/c','/coff',f'/Fo{obj}',str(asm)],
        [str(args.sdk_bin_x86/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE',
         '/ENTRY:HostMain','/BASE:0x004B0000','/DYNAMICBASE:NO','/SAFESEH:NO',
         f'/OUT:{exe}',f'/MAP:{mapping}',str(obj),str(args.kernel32_lib)],
    ]:
        result=subprocess.run(command,capture_output=True,text=True)
        assert result.returncode==0,result.stdout+'\n'+result.stderr
    manifest=dict(source_sha256=SOURCE_SHA256,host_executed=False,host_image_base='004B0000',
        code_allowlist=[native],
        data_allowlist=[dict(address=f'{address:08X}',bytes=4,word=f'{word:08X}',
            sha256=hashlib.sha256(image.read(address,4)).hexdigest())for address,word in CONSTANTS.items()]+
            [dict(address='016B54E4',bytes=12,source='explicit_input_origin_bits'),
             dict(address='016B5534',bytes=36,source='explicit_input_matrix_bits')],
        replacement_provider=linked_audit(exe,mapping),imported_host_functions=IMPORTS,
        pages=[dict(address=f'{page:08X}',bytes=65536,protection='RWX'if page==0xe80000 else'RW')for page in sorted(pages)],
        protocol=dict(input_bytes=INPUT_BYTES,output_bytes=OUTPUT_BYTES,
            input_fields=['source_count:u32@0=0..256','explicit_class:u32@4=1..12whenconsumed',
                'source_resource_key:3*u32@8(low,type,high)','CW:u16@20','reserved@22..31',
                'source_row:140bytes@32','special_origin:3*u32@172',
                'special_matrix:9*u32@184','reserved@220..223'],
            output_fields=['output_count:u32@0','loader_count:u32@4','class_count:u32@8',
                'release_count:u32@12','first_part:72bytes@16','last_part:72bytes@88'],
            no_rows='first/last remainA5; specialresource alwaysonepart'),
        setup_failure_exit_codes={'10':'VirtualAllocE80000','11':'VirtualAlloc1470000',
            '12':'VirtualAlloc1480000','13':'VirtualAlloc16B0000','20':'FlushInstructionCache','7':'Protocol/provider/guard'},
        mxcsr_control='1F80',host_source_sha256=hashlib.sha256(asm.read_bytes()).hexdigest(),
        host_exe_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(),
        scope='Only original E83E60 geometry and first64limit; owned checked loader/classifier stubs. Classes and special globals explicit, no original property-manager/inheritance execution, no actor/initializer/retailprocess/GUI/network.')
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2))
    print(json.dumps({key:manifest[key]for key in ['source_sha256','host_executed','host_source_sha256','host_exe_sha256']},indent=2))


if __name__=='__main__':main()
