"""Prepare, never execute, the original E51EC0 skip-flag reduction host.

Only the geometry/count/index prefix and shared return epilogue are copied.
The caller supplies literal skipflag1, making every manager/curve call absent
and unreachable. One checked owned memset callback clears13counterwords.
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

CODE=[(0xe51ec0,0xe524a4),(0xe5271f,0xe52726)]
CLEAR=0x11e052e
CONSTANTS={0x13f05f4:0x461c4000,0x13ec5ac:0x447a0000,0x1471b2c:0x3f000000}
INPUT_BYTES=4784
OUTPUT_BYTES=144


def audit(image):
    sys.path.insert(0,str(Path(__file__).resolve().parent.parent/'target/rea/capstone-runtime'))
    from capstone import Cs,CS_ARCH_X86,CS_MODE_32,CS_GRP_CALL,CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM,X86_OP_MEM
    decoder=Cs(CS_ARCH_X86,CS_MODE_32);decoder.detail=True
    records=[];calls=[];absolutes=set(CONSTANTS)|{0x16b4c08,0x16b4c0c,0x16b4c10}
    for start,end in CODE:
        rows=list(decoder.disasm(image.read(start,end-start),start))
        assert sum(row.size for row in rows)==end-start
        memory=[]
        for ins in rows:
            if ins.group(CS_GRP_CALL)or ins.group(CS_GRP_JUMP):
                operand=ins.operands[0];assert operand.type==X86_OP_IMM
                target=operand.imm
                if ins.group(CS_GRP_CALL):
                    assert ins.address==0xe51ed3 and target==CLEAR
                    calls.append(dict(at=f'{ins.address:08X}',target=f'{target:08X}'))
                else:assert any(a<=target<b for a,b in CODE)
            for operand in ins.operands:
                if operand.type!=X86_OP_MEM:continue
                if operand.mem.base==0:
                    assert operand.mem.index==0 and operand.mem.disp in absolutes
                memory.append(ins.op_str)
        records.append(dict(start=f'{start:08X}',end_exclusive=f'{end:08X}',bytes=end-start,
            sha256=hashlib.sha256(image.read(start,end-start)).hexdigest(),
            memory_operands=sorted(set(memory)),
            disassembly=[f'{ins.address:08X}: {ins.mnemonic} {ins.op_str}'for ins in rows]))
    assert calls==[dict(at='00E51ED3',target='011E052E')]
    for address,value in CONSTANTS.items():assert image.read(address,4)==struct.pack('<I',value)
    return records,calls


def source(image):
    lines=['.686p','.xmm','.model flat','option casemap:none',
        'EXTERN _GetStdHandle@4:PROC','EXTERN _ReadFile@20:PROC','EXTERN _WriteFile@20:PROC',
        'EXTERN _VirtualAlloc@16:PROC','EXTERN _GetLastError@0:PROC',
        'EXTERN _FlushInstructionCache@12:PROC','EXTERN _ExitProcess@4:PROC',
        '.data','InputBefore DWORD 4 DUP(0A5A5A5A5h)','InputBuffer BYTE 4784 DUP(0)',
        'InputAfter DWORD 4 DUP(0A5A5A5A5h)',
        'CacheBefore DWORD 4 DUP(0A5A5A5A5h)','NativeCache BYTE 4752 DUP(0)',
        'CacheAfter DWORD 4 DUP(0A5A5A5A5h)',
        'OutputBefore DWORD 4 DUP(0A5A5A5A5h)','ReplyBuffer BYTE 144 DUP(0)',
        'OutputAfter DWORD 4 DUP(0A5A5A5A5h)',
        'InputHandle DWORD 0','OutputHandle DWORD 0','Transferred DWORD 0',
        'NativeStack DWORD 0','ExpectedReturnStack DWORD 0','SseControl DWORD 01F80h',
        'ReturnedControl WORD 0','ReturnedStatus WORD 0','ReturnedSse DWORD 0']
    for start,end in CODE:lines+=db(f'Code_{start:08X}',image.read(start,end-start))
    for address in CONSTANTS:lines+=db(f'Constant_{address:08X}',image.read(address,4))
    lines+=['.code','PUBLIC HostMain,ClearCounters,HostFailure',
        'ClearCounters PROC','cmp dword ptr [esp],00E51ED8h','jne HostFailure',
        'cmp dword ptr [esp+4],OFFSET NativeCache+01204h','jne HostFailure',
        'cmp dword ptr [esp+8],0','jne HostFailure','cmp dword ptr [esp+12],52','jne HostFailure',
        'cmp dword ptr [ReplyBuffer],0','jne HostFailure','inc dword ptr [ReplyBuffer]',
        'push edi','mov edi,OFFSET NativeCache+01204h','xor eax,eax','mov ecx,13',
        'cld','rep stosd','pop edi','mov eax,OFFSET NativeCache+01204h','ret','ClearCounters ENDP',
        'ReadRecord PROC','push ebx','push esi','mov ebx,4784','mov esi,OFFSET InputBuffer',
        'ReadLoop:','mov Transferred,0','push 0','push OFFSET Transferred','push ebx',
        'push esi','push InputHandle','call _ReadFile@20','test eax,eax','jnz ReadTransferred',
        'call _GetLastError@0','cmp eax,109','jne HostFailure','cmp ebx,4784','jne HostFailure',
        'cmp Transferred,0','jne HostFailure','xor eax,eax','jmp ReadDone','ReadTransferred:',
        'mov eax,Transferred','test eax,eax','jnz ReadSome','cmp ebx,4784','jne HostFailure',
        'xor eax,eax','jmp ReadDone','ReadSome:','add esi,eax','sub ebx,eax','jnz ReadLoop',
        'mov eax,1','ReadDone:','pop esi','pop ebx','ret','ReadRecord ENDP',
        'HostMain PROC']
    pages={0xe50000,0x11e0000,0x13e0000,0x13f0000,0x1470000,0x16b0000}
    code_pages={0xe50000,0x11e0000}
    for ordinal,page in enumerate(sorted(pages),10):
        lines+=[f'push {"040h"if page in code_pages else"04h"}','push 03000h',
            'push 010000h',f'push 0{page:08X}h','call _VirtualAlloc@16',
            f'cmp eax,0{page:08X}h',f'jne AllocationFailure{ordinal}']
        if page in code_pages:
            lines+=[f'mov edi,0{page:08X}h','mov ecx,010000h','mov al,0CCh','cld','rep stosb']
    for start,end in CODE:
        lines+=[f'mov esi,OFFSET Code_{start:08X}',f'mov edi,0{start:08X}h',f'mov ecx,{end-start}','rep movsb']
    lines+=['mov byte ptr [011E052Eh],0B8h','mov dword ptr [011E052Fh],OFFSET ClearCounters',
        'mov word ptr [011E0533h],0E0FFh']
    for address in CONSTANTS:
        lines+=[f'mov esi,OFFSET Constant_{address:08X}',f'mov edi,0{address:08X}h','mov ecx,4','rep movsb']
    lines+=['push 0','push 0','push -1','call _FlushInstructionCache@12',
        'test eax,eax','jz CacheFlushFailure',
        'push -10','call _GetStdHandle@4','mov InputHandle,eax',
        'push -11','call _GetStdHandle@4','mov OutputHandle,eax',
        'NextRecord:','call ReadRecord','test eax,eax','jz HostSuccess',
        'mov ecx,dword ptr [InputBuffer]','cmp ecx,64','ja HostFailure',
        'mov edx,OFFSET InputBuffer+32','test ecx,ecx','jz ClassesReady','CheckClass:',
        'cmp dword ptr [edx+4],12','ja HostFailure','add edx,72','dec ecx','jnz CheckClass','ClassesReady:',
        'movzx eax,word ptr [InputBuffer+4]','mov edx,eax','and edx,0F0FFh',
        'cmp edx,07Fh','jne HostFailure','and eax,0300h','cmp eax,0100h','je HostFailure',
        'mov edi,OFFSET ReplyBuffer','xor eax,eax','mov ecx,36','rep stosd',
        'mov edi,OFFSET NativeCache','mov eax,0A5A5A5A5h','mov ecx,1188','rep stosd',
        'mov esi,OFFSET InputBuffer+4640','mov edi,OFFSET NativeCache+01204h','mov ecx,35','rep movsd',
        'mov esi,OFFSET InputBuffer+8','mov edi,016B4C08h','mov ecx,3','rep movsd',
        'fninit','fldcw word ptr [InputBuffer+4]','ldmxcsr SseControl',
        'mov NativeStack,esp','and esp,0FFFFFFF0h',
        # This immutable literal is the only caller skipflag. The native prefix
        # neither writes its caller argument nor reaches uncopied E524A4 code.
        'push 1','push OFFSET InputBuffer+32','mov eax,dword ptr [InputBuffer]',
        'mov esi,OFFSET NativeCache','mov edx,00E51EC0h','mov ExpectedReturnStack,esp','call edx',
        'cmp esp,ExpectedReturnStack','jne HostFailure',
        'fnstcw ReturnedControl','mov ax,ReturnedControl','cmp ax,word ptr [InputBuffer+4]','jne HostFailure',
        'fnstsw ReturnedStatus','movzx eax,ReturnedStatus','and eax,03800h','jnz HostFailure',
        'stmxcsr ReturnedSse','mov eax,ReturnedSse','and eax,0FFFFFFC0h',
        'cmp eax,01F80h','jne HostFailure','mov esp,NativeStack',
        'cmp dword ptr [ReplyBuffer],1','jne HostFailure',
        'mov edi,OFFSET NativeCache','mov ecx,01204h','mov al,0A5h','repe scasb','jne HostFailure',
        'mov esi,OFFSET NativeCache+01204h','mov edi,OFFSET ReplyBuffer+4','mov ecx,35','rep movsd']
    for label in ['InputBefore','InputAfter','CacheBefore','CacheAfter','OutputBefore','OutputAfter']:
        for offset in range(0,16,4):
            lines+=[f'cmp dword ptr [{label}+{offset}],0A5A5A5A5h','jne HostFailure']
    lines+=['push 0','push OFFSET Transferred','push 144','push OFFSET ReplyBuffer',
        'push OutputHandle','call _WriteFile@20','test eax,eax','jz HostFailure',
        'cmp Transferred,144','jne HostFailure','jmp NextRecord',
        'HostSuccess:','push 0','call _ExitProcess@4']
    for ordinal in range(10,16):
        lines+=[f'AllocationFailure{ordinal}:',f'push {ordinal}','call _ExitProcess@4']
    lines+=['CacheFlushFailure:','push 20','call _ExitProcess@4',
        'HostFailure::','push 7','call _ExitProcess@4','HostMain ENDP','END HostMain']
    return '\n'.join(lines)+'\n',pages,code_pages


def symbols(mapping):
    result={}
    for line in mapping.read_text().splitlines():
        match=re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\w+)\s+([0-9A-Fa-f]{8})\b',line)
        if match and match[1]in{'HostMain','ClearCounters','HostFailure'}:result[match[1]]=int(match[2],16)
    assert len(result)==3,result
    return result


def main():
    ap=argparse.ArgumentParser();ap.add_argument('exe',type=Path)
    ap.add_argument('--sdk-bin-x86',type=Path,required=True)
    ap.add_argument('--kernel32-lib',type=Path,required=True)
    ap.add_argument('--out',type=Path,required=True);args=ap.parse_args()
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    image=SourceImage(args.exe);native,calls=audit(image);assembly,pages,code_pages=source(image)
    args.out.mkdir(parents=True,exist_ok=False);out=args.out.resolve()
    asm=out/'host-x86.asm';obj=out/'host-x86.obj';exe=out/'host-x86.exe';mapping=out/'host-x86.map'
    asm.write_text(assembly)
    for command in [
        [str(args.sdk_bin_x86/'ml.exe'),'/nologo','/c','/coff',f'/Fo{obj}',str(asm)],
        [str(args.sdk_bin_x86/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE',
         '/ENTRY:HostMain','/BASE:0x00680000','/DYNAMICBASE:NO','/SAFESEH:NO',
         f'/OUT:{exe}',f'/MAP:{mapping}',str(obj),str(args.kernel32_lib)],
    ]:
        result=subprocess.run(command,capture_output=True,text=True)
        assert result.returncode==0,result.stdout+'\n'+result.stderr
    addresses=symbols(mapping)
    manifest=dict(source_sha256=SOURCE_SHA256,host_executed=False,host_image_base='00680000',
        code_allowlist=native,direct_calls=calls,indirect_calls=0,
        data_allowlist=[dict(address=f'{address:08X}',bytes=4,word=f'{word:08X}',
            sha256=hashlib.sha256(image.read(address,4)).hexdigest())for address,word in CONSTANTS.items()]+
            [dict(address='016B4C08',bytes=12,source='explicit_input_empty_center_bits')],
        replacement_clear=dict(entry=f'{CLEAR:08X}',
            stub_hex=(b'\xb8'+struct.pack('<I',addresses['ClearCounters'])+b'\xff\xe0').hex(),
            target=f'{addresses["ClearCounters"]:08X}',outbound_relative_transfers_in_fixed_stub=0,
            proof='Only callsiteE51ED3 ->fixedabsoluteMOV/JMP. Checked callback requires returnE51ED8,destNativeCache+1204,value0,bytes52,andoncecounter. It preserves EDI/ESI/EBX/EBP; only13counterwords andownoncecounterwritten; normal cdeclreturn.'),
        omitted_path_proof='Host pushes literal skipflag1 as callerarg2. Native CMPbyte[ESP+44],0 atE52493 sees that immutable argument; JNZatE5249E reaches copiedE5271F epilogue. FalsefallthroughE524A4 andallmanager/curvebodies remainINT3. Nativeprefix writes onlyboundedlocals andcachefields, nevercallerarg2. No gamefunction iscalledexceptownedcheckedmemset.',
        post_call_guards=dict(stack='SaveESPimmediatelybeforeCALL; compareESPbeforeanyrestoration,requireunchangedcdeclarguments.',
            x87_control='FNSTCWmustmatchsuppliedCWexactly.',x87_top='FNSTSW&3800mustbezero(TOP0).',
            mxcsr='STMXCSR&FFFFFFC0mustbe1F80; low6stickyexceptionflagsmayvary.'),
        bounds_proof='count0..64,allactiveclasswords0..12validatedbeforeentry. Allrowreads stay in64*72inputbytes,allcounter/indexwrites stay in NativeCache1204..128F. NativeCacheprefix0..1203ischeckedunchangedA5. Input/cache/reply areseparateguardedhostbuffers. Inputhasnopointers/callbacks; allabsoluteoriginalreadsarelistedconstantsor12explicitglobalbytes.',
        imported_host_functions=IMPORTS,
        pages=[dict(address=f'{page:08X}',bytes=65536,protection='RWX'if page in code_pages else'RW')for page in sorted(pages)],
        protocol=dict(input_bytes=INPUT_BYTES,output_bytes=OUTPUT_BYTES,
            input_fields=['count:u32@0=0..64','CW:u16@4','reserved@6..7',
                'empty_center:3*u32@8','reserved@20..31','rows:64*72bytes@32',
                'initial_cache_tail:140bytes@4640(native1204..128F)','reserved@4780..4783'],
            output_fields=['clear_count:u32@0=1','complete_cache_tail:140bytes@4']),
        setup_failure_exit_codes={str(n):f'VirtualAlloc{page:08X}'for n,page in enumerate(sorted(pages),10)}|{'20':'FlushInstructionCache','7':'Protocol/clear/bufferguard'},
        mxcsr_control='1F80',host_source_sha256=hashlib.sha256(asm.read_bytes()).hexdigest(),
        host_exe_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(),
        scope='OriginalE51EC0skipflagprefixonly,checkedownedmemset; explicitresolvedclasses/geometry/emptycenter/x87policy. No managers,curves,actor,initializer,retailprocess,GUIornetwork.')
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2))
    print(json.dumps({key:manifest[key]for key in ['source_sha256','host_executed','host_source_sha256','host_exe_sha256']},indent=2))


if __name__=='__main__':main()
