"""Prepare, never execute, bounded original Cell/list-presence classification.

Three original ranges execute against owned lists with32IDs and forward parent
indices. Archive lookup and Parent resource loading are replaced by an explicit
checked manager. All proprietary bytes/generated binaries remain under target.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

from cell_capability_cache_probe import TAGS
from cell_native_math_probe import IMPORTS,db
from cell_schema_probe import SOURCE_SHA256,SourceImage

CODE=[(0xe827a0,0xe828e1),(0x6a1f20,0x6a1f79),(0x612410,0x61245d)]
PROVIDER=0x401010
INPUT_BYTES=1120
OUTPUT_BYTES=16


def audit(image):
    sys.path.insert(0,str(Path(__file__).resolve().parent.parent/'target/rea/capstone-runtime'))
    from capstone import Cs,CS_ARCH_X86,CS_MODE_32,CS_GRP_CALL,CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM,X86_OP_MEM,X86_OP_REG
    decoder=Cs(CS_ARCH_X86,CS_MODE_32);decoder.detail=True;records=[]
    for start,end in CODE:
        rows=list(decoder.disasm(image.read(start,end-start),start))
        assert sum(row.size for row in rows)==end-start
        direct=[];indirect=[];memory=[]
        for ins in rows:
            if ins.group(CS_GRP_CALL)or ins.group(CS_GRP_JUMP):
                operand=ins.operands[0]
                if operand.type==X86_OP_IMM:
                    target=operand.imm
                    if ins.group(CS_GRP_CALL):
                        assert (ins.address,target)in{(0xe827a1,PROVIDER),(0x6a1f36,0x612410)}
                        direct.append(dict(at=f'{ins.address:08X}',target=f'{target:08X}'))
                    else:assert start<=target<end
                else:
                    assert operand.type==X86_OP_REG
                    assert start==0xe827a0 and ins.group(CS_GRP_CALL)or ins.address==0x6a1f70
                    indirect.append(dict(at=f'{ins.address:08X}',operation=ins.mnemonic,operand=ins.op_str))
            for operand in ins.operands:
                if operand.type==X86_OP_MEM:
                    assert operand.mem.base!=0
                    memory.append(ins.op_str)
        assert len(indirect)==(12 if start==0xe827a0 else 1 if start==0x6a1f20 else 0)
        records.append(dict(start=f'{start:08X}',end_exclusive=f'{end:08X}',bytes=end-start,
            sha256=hashlib.sha256(image.read(start,end-start)).hexdigest(),direct_calls=direct,
            bounded_indirect_transfers=indirect,memory_operands=sorted(set(memory)),
            disassembly=[f'{ins.address:08X}: {ins.mnemonic} {ins.op_str}'for ins in rows]))
    return records


def source(image):
    lines=['.686p','.xmm','.model flat','option casemap:none',
        'EXTERN _GetStdHandle@4:PROC','EXTERN _ReadFile@20:PROC','EXTERN _WriteFile@20:PROC',
        'EXTERN _VirtualAlloc@16:PROC','EXTERN _GetLastError@0:PROC',
        'EXTERN _FlushInstructionCache@12:PROC','EXTERN _ExitProcess@4:PROC',
        '.const','PUBLIC ManagerVtable,ListVtable,FakeManager',
        'ManagerVtable DWORD 22 DUP(0),OFFSET ManagerLookup',
        'ListVtable DWORD 7 DUP(0),OFFSET CheckedHasProperty',
        'FakeManager DWORD OFFSET ManagerVtable','NodeStride DWORD 56',
        'ClassQueries DWORD 0,11,2,3,7,9,8,6,10,4,5,11,1',
        'Tags DWORD '+','.join(f'0{ident:08X}h'for ident,_ in TAGS),
        '.data','InputBefore DWORD 4 DUP(0A5A5A5A5h)','InputBuffer BYTE 1120 DUP(0)',
        'InputAfter DWORD 4 DUP(0A5A5A5A5h)',
        'NodesBefore DWORD 4 DUP(0A5A5A5A5h)','FakeNodes BYTE 448 DUP(0)',
        'NodesAfter DWORD 4 DUP(0A5A5A5A5h)',
        'EntriesBefore DWORD 4 DUP(0A5A5A5A5h)','Entries BYTE 6144 DUP(0)',
        'EntriesAfter DWORD 4 DUP(0A5A5A5A5h)',
        'PartBefore DWORD 4 DUP(0A5A5A5A5h)','FakePart BYTE 140 DUP(0)',
        'PartAfter DWORD 4 DUP(0A5A5A5A5h)',
        'OutputBefore DWORD 4 DUP(0A5A5A5A5h)','ReplyBuffer DWORD 4 DUP(0)',
        'OutputAfter DWORD 4 DUP(0A5A5A5A5h)',
        'InputHandle DWORD 0','OutputHandle DWORD 0','Transferred DWORD 0',
        'CurrentQuery DWORD 0','ExpectedParent DWORD 0',
        'NativeStack DWORD 0','ExpectedReturnStack DWORD 0','SseControl DWORD 01F80h',
        'ReturnedControl WORD 0','ReturnedStatus WORD 0','ReturnedSse DWORD 0']
    lines+=db('ClassCode',image.read(*[CODE[0][0],CODE[0][1]-CODE[0][0]]))
    lines+=['.code','TextBase LABEL BYTE','DB 010h DUP(0CCh)',
        'PUBLIC ManagerProvider,ManagerLookup,CheckedHasProperty,HostMain,HostFailure,NativePresence,NativeLowerBound',
        'ManagerProvider PROC','mov eax,OFFSET FakeManager','ret','ManagerProvider ENDP',
        'ManagerLookup PROC','cmp ecx,OFFSET FakeManager','jne HostFailure',
        'mov eax,dword ptr [esp+4]','cmp eax,dword ptr [InputBuffer+4]','jne HostFailure',
        'mov eax,dword ptr [esp+8]','cmp eax,dword ptr [InputBuffer+8]','jne HostFailure',
        'cmp dword ptr [ReplyBuffer+4],0','jne HostFailure','inc dword ptr [ReplyBuffer+4]',
        'xor eax,eax','cmp dword ptr [InputBuffer],0','je MissingRoot','mov eax,OFFSET FakeNodes',
        'MissingRoot:','ret 8','ManagerLookup ENDP',
        'CheckedHasProperty PROC','cmp ecx,OFFSET FakeNodes','jb HostFailure',
        'mov eax,ecx','sub eax,OFFSET FakeNodes','xor edx,edx','div NodeStride',
        'test edx,edx','jnz HostFailure','cmp eax,dword ptr [InputBuffer]','jae HostFailure',
        'test eax,eax','jz RootQuery','cmp ecx,ExpectedParent','jne HostFailure',
        'mov eax,dword ptr [esp+4]','cmp eax,CurrentQuery','jne HostFailure','jmp QueryReady',
        'RootQuery:','mov eax,dword ptr [ReplyBuffer+8]','cmp eax,11','jae HostFailure',
        'mov edx,dword ptr [esp+4]','cmp edx,dword ptr [Tags+eax*4]','jne HostFailure',
        'mov CurrentQuery,edx','inc dword ptr [ReplyBuffer+8]','QueryReady:',
        'mov eax,dword ptr [ecx+48]','mov ExpectedParent,eax',
        'inc dword ptr [ReplyBuffer+12]','cmp dword ptr [ReplyBuffer+12],88','ja HostFailure',
        'mov eax,006A1F20h','jmp eax','CheckedHasProperty ENDP',
        'ReadRecord PROC','push ebx','push esi','mov ebx,1120','mov esi,OFFSET InputBuffer',
        'ReadLoop:','mov Transferred,0','push 0','push OFFSET Transferred','push ebx',
        'push esi','push InputHandle','call _ReadFile@20','test eax,eax','jnz ReadTransferred',
        'call _GetLastError@0','cmp eax,109','jne HostFailure','cmp ebx,1120','jne HostFailure',
        'cmp Transferred,0','jne HostFailure','xor eax,eax','jmp ReadDone','ReadTransferred:',
        'mov eax,Transferred','test eax,eax','jnz ReadSome','cmp ebx,1120','jne HostFailure',
        'xor eax,eax','jmp ReadDone','ReadSome:','add esi,eax','sub ebx,eax','jnz ReadLoop',
        'mov eax,1','ReadDone:','pop esi','pop ebx','ret','ReadRecord ENDP',
        'HostMain PROC','push 040h','push 03000h','push 010000h','push 00E80000h',
        'call _VirtualAlloc@16','cmp eax,00E80000h','jne AllocationFailure',
        'mov edi,00E80000h','mov ecx,010000h','mov al,0CCh','cld','rep stosb',
        'mov esi,OFFSET ClassCode','mov edi,00E827A0h','mov ecx,321','rep movsb',
        'push 0','push 0','push -1','call _FlushInstructionCache@12',
        'test eax,eax','jz CacheFlushFailure',
        'push -10','call _GetStdHandle@4','mov InputHandle,eax',
        'push -11','call _GetStdHandle@4','mov OutputHandle,eax',
        'NextRecord:','call ReadRecord','test eax,eax','jz HostSuccess',
        'cmp dword ptr [InputBuffer],8','ja HostFailure',
        'movzx eax,word ptr [InputBuffer+12]','mov edx,eax','and edx,0F0FFh',
        'cmp edx,07Fh','jne HostFailure','and eax,0300h','cmp eax,0100h','je HostFailure',
        'mov edi,OFFSET ReplyBuffer','xor eax,eax','mov ecx,4','rep stosd',
        'mov edi,OFFSET FakeNodes','mov eax,0A5A5A5A5h','mov ecx,112','rep stosd',
        'mov edi,OFFSET Entries','mov ecx,1536','rep stosd',
        'mov edi,OFFSET FakePart','mov ecx,35','rep stosd',
        'mov eax,dword ptr [InputBuffer+4]','mov dword ptr [FakePart+136],eax',
        'mov eax,dword ptr [InputBuffer+8]','mov dword ptr [FakePart+132],eax',
        'mov esi,OFFSET InputBuffer+32','mov edi,OFFSET FakeNodes','mov edx,OFFSET Entries',
        'xor ebx,ebx','cmp dword ptr [InputBuffer],0','je NodesReady','BuildNode:',
        'mov eax,dword ptr [esi]','cmp eax,32','ja HostFailure',
        'mov dword ptr [edi],OFFSET ListVtable','mov dword ptr [edi+24],edx',
        'imul eax,eax,24','add eax,edx','mov dword ptr [edi+28],eax',
        'mov dword ptr [edi+44],0','mov eax,dword ptr [esi+4]','cmp eax,0FFFFFFFFh','je NoParent',
        'cmp eax,ebx','jbe HostFailure','cmp eax,dword ptr [InputBuffer]','jae HostFailure',
        'imul eax,eax,56','add eax,OFFSET FakeNodes','jmp ParentReady',
        'NoParent:','xor eax,eax','ParentReady:','mov dword ptr [edi+48],eax',
        'mov ecx,32','CopyIdentifiers:','mov eax,dword ptr [esi+8]','mov dword ptr [edx],eax',
        'add esi,4','add edx,24','dec ecx','jnz CopyIdentifiers',
        'add esi,8','add edi,56','inc ebx','cmp ebx,dword ptr [InputBuffer]','jb BuildNode','NodesReady:',
        'fninit','fldcw word ptr [InputBuffer+12]','ldmxcsr SseControl',
        'mov NativeStack,esp','and esp,0FFFFFFF0h','push OFFSET FakePart',
        'mov ExpectedReturnStack,esp','mov eax,00E827A0h','call eax',
        'cmp esp,ExpectedReturnStack','jne HostFailure','mov dword ptr [ReplyBuffer],eax',
        'fnstcw ReturnedControl','mov ax,ReturnedControl','cmp ax,word ptr [InputBuffer+12]','jne HostFailure',
        'fnstsw ReturnedStatus','movzx eax,ReturnedStatus','and eax,03800h','jnz HostFailure',
        'stmxcsr ReturnedSse','mov eax,ReturnedSse','and eax,0FFFFFFC0h','cmp eax,01F80h','jne HostFailure',
        'mov esp,NativeStack','cmp dword ptr [ReplyBuffer+4],1','jne HostFailure',
        'mov eax,dword ptr [ReplyBuffer]','dec eax','cmp eax,11','ja HostFailure','inc eax',
        'cmp dword ptr [InputBuffer],0','jne CheckClassQueries',
        'cmp eax,1','jne HostFailure','cmp dword ptr [ReplyBuffer+8],0','jne HostFailure',
        'cmp dword ptr [ReplyBuffer+12],0','jne HostFailure','jmp CheckGuards',
        'CheckClassQueries:','mov eax,dword ptr [ClassQueries+eax*4]',
        'cmp dword ptr [ReplyBuffer+8],eax','jne HostFailure',
        'cmp dword ptr [ReplyBuffer+12],eax','jb HostFailure',
        'imul eax,dword ptr [InputBuffer]','cmp dword ptr [ReplyBuffer+12],eax','ja HostFailure','CheckGuards:']
    for label in ['InputBefore','InputAfter','NodesBefore','NodesAfter','EntriesBefore','EntriesAfter',
                  'PartBefore','PartAfter','OutputBefore','OutputAfter']:
        for offset in range(0,16,4):
            lines+=[f'cmp dword ptr [{label}+{offset}],0A5A5A5A5h','jne HostFailure']
    lines+=['push 0','push OFFSET Transferred','push 16','push OFFSET ReplyBuffer',
        'push OutputHandle','call _WriteFile@20','test eax,eax','jz HostFailure',
        'cmp Transferred,16','jne HostFailure','jmp NextRecord',
        'HostSuccess:','push 0','call _ExitProcess@4',
        'AllocationFailure:','push 10','call _ExitProcess@4',
        'CacheFlushFailure:','push 20','call _ExitProcess@4',
        'HostFailure::','push 7','call _ExitProcess@4','HostMain ENDP',
        'DB 0211410h-($-TextBase) DUP(0CCh)','NativeLowerBound LABEL BYTE']
    lines+=db('LowerBoundBytes',image.read(0x612410,77))
    lines+=['DB 02A0F20h-($-TextBase) DUP(0CCh)','NativePresence LABEL BYTE']
    lines+=db('PresenceBytes',image.read(0x6a1f20,89))
    lines+=['END HostMain']
    return '\n'.join(lines)+'\n'


def linked_audit(image,exe,mapping):
    wanted={'ManagerProvider','ManagerLookup','CheckedHasProperty','NativeLowerBound','NativePresence',
        'ManagerVtable','ListVtable','FakeManager','HostFailure'};symbols={}
    for line in mapping.read_text().splitlines():
        match=re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\w+)\s+([0-9A-Fa-f]{8})\b',line)
        if match and match[1]in wanted:symbols[match[1]]=int(match[2],16)
    assert set(symbols)==wanted,symbols
    raw=exe.read_bytes();pe=struct.unpack_from('<I',raw,60)[0]
    base=struct.unpack_from('<I',raw,pe+52)[0];count=struct.unpack_from('<H',raw,pe+6)[0]
    optional=struct.unpack_from('<H',raw,pe+20)[0];sections=[]
    for index in range(count):
        at=pe+24+optional+40*index;_,rva,size,offset=struct.unpack_from('<4I',raw,at+8)
        flags=struct.unpack_from('<I',raw,at+36)[0];sections.append((base+rva,size,offset,flags))
    def read(address,length):
        for start,size,offset,flags in sections:
            if start<=address and address+length<=start+size:
                at=offset+address-start;return raw[at:at+length],flags
        raise ValueError(f'unmapped_host_address{address:08X}')
    assert base==0x400000 and symbols['ManagerProvider']==0x401010
    for label,start,length in [('NativeLowerBound',0x612410,77),('NativePresence',0x6a1f20,89)]:
        assert symbols[label]==start
        actual,flags=read(start,length)
        assert actual==image.read(start,length)and flags&0x60000000==0x60000000 and not flags&0x80000000
    provider=b'\xb8'+struct.pack('<I',symbols['FakeManager'])+b'\xc3';actual,flags=read(PROVIDER,6)
    assert actual==provider and flags&0x60000000==0x60000000 and not flags&0x80000000
    for label in ['FakeManager','ManagerVtable','ListVtable']:
        _,flags=read(symbols[label],4);assert flags&0x40000000 and not flags&0x80000000
    assert struct.unpack('<I',read(symbols['FakeManager'],4)[0])[0]==symbols['ManagerVtable']
    assert struct.unpack('<I',read(symbols['ManagerVtable']+0x58,4)[0])[0]==symbols['ManagerLookup']
    assert struct.unpack('<I',read(symbols['ListVtable']+0x1c,4)[0])[0]==symbols['CheckedHasProperty']
    return dict(symbols={label:f'{address:08X}'for label,address in symbols.items()},
        provider_stub_hex=provider.hex(),provider_compiled_RX_not_writable=True,
        original_helpers_linked_exact_RX_addresses=True,owned_padding='INT3',immutable_vtables_and_manager=True,
        proof='ManagerLookup requires exact FakeManager andinputkeylow/scope,onceonly. ItreturnsNULLorFakeNodes0. Everylistcallbackvalidatesthisalignment/index,queryorderandforwardparentpointer,thenabsolutetailjumps6A1F20. Alloriginalparentvirtualjumpsreturnthroughsamecheckedcallback. Nodeparentsarestrictlyforwardandbounded,arrays0..32entriesof24bytes,nomorethan8nodes/88visits. Noinputpointerorcallbackaddressaccepted.')


def main():
    ap=argparse.ArgumentParser();ap.add_argument('exe',type=Path)
    ap.add_argument('--sdk-bin-x86',type=Path,required=True);ap.add_argument('--kernel32-lib',type=Path,required=True)
    ap.add_argument('--out',type=Path,required=True);args=ap.parse_args()
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    image=SourceImage(args.exe);native=audit(image);assembly=source(image)
    args.out.mkdir(parents=True,exist_ok=False);out=args.out.resolve()
    asm=out/'host-x86.asm';obj=out/'host-x86.obj';exe=out/'host-x86.exe';mapping=out/'host-x86.map'
    asm.write_text(assembly)
    for command in [
        [str(args.sdk_bin_x86/'ml.exe'),'/nologo','/c','/coff',f'/Fo{obj}',str(asm)],
        [str(args.sdk_bin_x86/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE',
         '/ENTRY:HostMain','/BASE:0x00400000','/DYNAMICBASE:NO','/SAFESEH:NO',
         f'/OUT:{exe}',f'/MAP:{mapping}',str(obj),str(args.kernel32_lib)],
    ]:
        result=subprocess.run(command,capture_output=True,text=True);assert result.returncode==0,result.stdout+'\n'+result.stderr
    manifest=dict(source_sha256=SOURCE_SHA256,host_executed=False,host_image_base='00400000',
        code_allowlist=native,data_allowlist=[],replacement_provider=linked_audit(image,exe,mapping),
        imported_host_functions=IMPORTS,pages=[dict(address='00E80000',bytes=65536,protection='RWX')],
        protocol=dict(input_bytes=INPUT_BYTES,output_bytes=OUTPUT_BYTES,
            input_fields=['nodecount:u32@0=0..8','partkeylow:u32@4','partkeyscope:u32@8','CW:u16@12',
                'reserved@14..31','nodes:8*136bytes@32(count:u32,parent:u32,32IDs:u32)',
                'parent=FFFFFFFFnoneorstrictlyforwardindex<nodecount;count0..32'],
            output_fields=['class:u32@0','manager_count:u32@4','root_query_count:u32@8','predicate_visit_count:u32@12']),
        post_call_guards=['exactESPbeforeouterstackrestore','unchangedCW','x87TOP0','MXCSRcontrol1F80(low6stickymayvary)',
            'ten16bytebufferguards','exactmanagercount1','class-specificrootquerycount','boundedpredicatevisits'],
        host_source_sha256=hashlib.sha256(asm.read_bytes()).hexdigest(),host_exe_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(),
        scope='OriginalE827A0plusordinarylist6A1F20andunsignedlowerbound612410;checkedownedmanager/listcallbacks. Nativearchive/Parentresource-loading notexecuted; resolvednodesexplicit. Noretailprocess,initializer,window,GUIornetwork.')
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2))
    print(json.dumps({key:manifest[key]for key in ['source_sha256','host_executed','host_source_sha256','host_exe_sha256']},indent=2))


if __name__=='__main__':main()
