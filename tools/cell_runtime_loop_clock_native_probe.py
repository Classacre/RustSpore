"""Prepare/audit an arithmetic-only original looping clock and CRT modf host.

No compilation/execution. Counter/event work is excluded at declared partial
entries. The original helper forwards to the actual finite SSE2 CRT kernel;
the owned import provider validates stack arguments and captures exact80 data.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import sys
import cell_runtime_clock_native_probe as base
import cell_runtime_clock_native_probe_v2 as fixed
from native_crt_acos_probe import PE,decoder,SOURCE_SHA256 as CRT_SHA
from cell_schema_probe import SourceImage,SOURCE_SHA256
from cell_native_math_probe import db

ROOT=Path(__file__).resolve().parent.parent
APP_CODE=[(0x9A1CDB,0x9A1CF1),(0x9A1D4E,0x9A1D69),(0x9A1D77,0x9A1DB2),(0x99BCB0,0x99BCE1)]
CRT_CODE=[(0x785A3350,0x785A3429)]
CODE=APP_CODE+CRT_CODE
DATA=[(0x1485E24,4),(0x785347F0,16),(0x78534800,16)]
EXITS={0x9A1CF1:'PrefixDone',0x9A1D69:'HostFailure',0x9A1DB2:'FrameDone',0x785A3429:'HostFailure',0x785A940A:'HostFailure'}
BUFFERS=[('Input',64),('Clip',0xC8),('Resource',0x130),('Context',16),('Frame',0x1000),('State',64),('Reply',64)]
BASE_SHA='393af5d597c3587652c34e123c4c274bf676e66b99bd4fe0833be767c877e74b'
FIXED_SHA='3b75609ce2428bd34f8ef7b7691d416bd4d11657f9fd3da86feb3d1a8af782a0'
def sha(raw):return hashlib.sha256(raw).hexdigest()
class Images:
    def __init__(self,app,crt):
        self.app_path=app;self.app=SourceImage(app);self.crt=PE(crt);assert sha(self.crt.data)==CRT_SHA
    def read(self,address,size):return self.crt.read(address,size)if address>=0x78000000 else self.app.read(address,size)
def import_export_proof(images):
    pe=PE(images.app_path)
    found=None;rva,size=pe.directories[1]
    for i in range(size//20):
        oft,_,_,name,ft=struct.unpack('<5I',pe.read(pe.base+rva+20*i,20))
        if not name:break
        dll=pe.text(pe.base+name)
        for j in range(1024):
            entry=pe.u32(pe.base+(oft or ft)+4*j)
            if not entry:break
            if pe.base+ft+4*j==0x13CC4C0:
                assert not entry&0x80000000;found=(dll,pe.text(pe.base+entry+2))
    assert found==('MSVCR90.dll','modf')
    crt=images.crt;rva,size=crt.directories[0];fields=struct.unpack('<IIHH7I',crt.read(crt.base+rva,40))
    for i in range(fields[7]):
        if crt.text(crt.base+crt.u32(crt.base+fields[9]+4*i))=='modf':
            ordinal=struct.unpack('<H',crt.read(crt.base+fields[10]+2*i,2))[0]
            assert crt.base+crt.u32(crt.base+fields[8]+4*ordinal)==0x785A3350
            return dict(game_iat='013CC4C0',dll=found[0],name=found[1],crt_export='785A3350',ordinal=fields[5]+ordinal)
    raise AssertionError('missing modf export')
def audit_source(images):
    sys.path.insert(0,str(ROOT/'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL,CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM,X86_OP_MEM
    decoded=[list(decoder().disasm(images.read(a,b-a),a))for a,b in CODE];bounds={i.address for rows in decoded for i in rows}
    records=[]
    for(a,b),rows in zip(CODE,decoded):
        assert sum(i.size for i in rows)==b-a;transfers=[]
        for i in rows:
            if i.group(CS_GRP_CALL)or i.group(CS_GRP_JUMP):
                op=i.operands[0]
                if op.type==X86_OP_IMM:
                    target=op.imm;assert target in bounds|set(EXITS)
                    if i.group(CS_GRP_CALL):assert(i.address,target)==(0x9A1D5B,0x99BCB0)
                else:assert i.address==0x99BCC1 and op.type==X86_OP_MEM and op.mem.disp==0x13CC4C0;target=0x13CC4C0
                transfers.append(dict(at=f'{i.address:08X}',kind=i.mnemonic,target=f'{target:08X}'))
            for op in i.operands:
                if op.type==X86_OP_MEM and not op.mem.base and not op.mem.index:
                    assert(op.mem.disp,op.size)in DATA+[base.ONE,(0x13CC4C0,4),(0x785BD7B0,4)]
        records.append(dict(start=f'{a:08X}',end_exclusive=f'{b:08X}',bytes=b-a,sha256=sha(images.read(a,b-a)),transfers=transfers,
            disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in rows]))
    assert images.read(0x1485E24,4)==bytes(4)
    return records

def host_source(images):
    old=(base.CODE,base.EXITS,base.BUFFERS)
    try:
        base.CODE,base.EXITS,base.BUFFERS=CODE,EXITS,BUFFERS
        text=fixed.host_source(images)
    finally:base.CODE,base.EXITS,base.BUFFERS=old
    constants=[]
    for a,n in DATA:constants += [f'PUBLIC Original_{a:08X}']+db(f'Original_{a:08X}',images.read(a,n))
    text=text.replace('.data\n','\n'.join(constants)+'\n.data\n',1)
    text=text.replace('NativeStack DWORD 0','NativeStack DWORD 0\nProviderCalls DWORD 0',1)
    text=text.replace('Expected BYTE 20 DUP(0)','Expected BYTE 56 DUP(0)',1)
    text=text.replace('HostFailure,DeltaDone,FrameDone','HostFailure,PrefixDone,FrameDone,ModfProvider,ModfReturn',1)
    start=text.index('ValidateRecord PROC\n');end=text.index('ValidateRecord ENDP\n',start)+len('ValidateRecord ENDP\n')
    validation=['ValidateRecord PROC','cmp dword ptr [InputBuffer],0','jne HostFailure','movzx eax,word ptr [InputBuffer+4]',
        'mov edx,eax','and eax,0F0FFh','cmp eax,07Fh','jne HostFailure','and edx,0300h','cmp edx,0100h','je HostFailure',
        'cmp word ptr [InputBuffer+6],0','jne HostFailure']
    for at in [32,36,40]:validation += [f'cmp dword ptr [InputBuffer+{at}],1','jne HostFailure']
    for at in range(44,64,4):validation += [f'cmp dword ptr [InputBuffer+{at}],0','jne HostFailure']
    validation += ['mov eax,dword ptr [InputBuffer+8]','mov edx,eax','and edx,07FFFFFFFh','cmp edx,07F800000h','jae HostFailure',
        'test eax,080000000h','jz LoopTimeValid','test edx,edx','jnz HostFailure','LoopTimeValid:',
        'mov eax,dword ptr [InputBuffer+24]','test eax,080000000h','jnz HostFailure','mov edx,eax','and edx,07FF00000h',
        'cmp edx,07FF00000h','je HostFailure','or eax,dword ptr [InputBuffer+20]','jz HostFailure',
        'mov eax,dword ptr [InputBuffer+28]','cmp eax,03F800000h','jb HostFailure','cmp eax,07F800000h','jae HostFailure','ret','ValidateRecord ENDP']
    text=text[:start]+'\n'.join(validation)+'\n'+text[end:]
    text=text.replace('mov byte ptr [ClipBuffer+0ADh],0','mov byte ptr [ClipBuffer+0ADh],1',1)
    text=text.replace('fninit\nfldcw','mov ProviderCalls,0\nfninit\nfldcw',1)
    start=text.index('cmp dword ptr [InputBuffer],0\nje PrepareClock\n');end=text.index('PrepareClock:\n',start)
    text=text[:start]+text[end:]
    prefix=['PrefixDone::','cmp esp,OFFSET FrameBuffer+0800h','jne HostFailure','mov esp,NativeStack',
        'fstp tbyte ptr [StateBuffer]','fstp st(0)','mov eax,dword ptr [FrameBuffer+0804h]','mov dword ptr [StateBuffer+12],eax',
        'and eax,07F800000h','cmp eax,07F800000h','je HostFailure','fld tbyte ptr [StateBuffer]',
        'mov esi,OFFSET ClipBuffer','mov esp,OFFSET FrameBuffer+0800h','push 009A1D4Eh','ret']
    text=text.replace('FrameDone::\n','\n'.join(prefix)+'\nFrameDone::\n',1)
    provider=['ModfProvider::','cmp esp,OFFSET FrameBuffer+07E0h','jne HostFailure',
        'cmp dword ptr [esp],0099BCC7h','jne HostFailure','cmp dword ptr [esp+12],OFFSET FrameBuffer+07F0h','jne HostFailure',
        'inc ProviderCalls','cmp ProviderCalls,1','jne HostFailure','mov eax,dword ptr [FrameBuffer+07FCh]',
        'mov dword ptr [StateBuffer+16],eax','mov eax,dword ptr [esp+4]','mov edx,dword ptr [esp+8]',
        'push dword ptr [esp+12]','push edx','push eax','push OFFSET ModfReturn','push 0785A3350h','ret',
        'ModfReturn::','cmp esp,OFFSET FrameBuffer+07D4h','jne HostFailure','fstp tbyte ptr [StateBuffer+28]',
        'fld tbyte ptr [StateBuffer+28]','mov eax,dword ptr [FrameBuffer+07F0h]','mov dword ptr [StateBuffer+20],eax',
        'mov eax,dword ptr [FrameBuffer+07F4h]','mov dword ptr [StateBuffer+24],eax','add esp,12','ret']
    text=text.replace('RunOnce PROC\n','\n'.join(provider)+'\nRunOnce PROC\n',1)
    state_compare='mov esi,OFFSET StateBuffer\nmov edi,OFFSET StateShadow\nmov ecx,16\nrepe cmpsd\njne HostFailure\n'
    assert text.count(state_compare)==1;text=text.replace(state_compare,'',1)
    text=text.replace('CheckFrame:\ncmp ecx,07E0h','CheckFrame:\ncmp ecx,07C0h',1)
    start=text.index('cmp dword ptr [ReplyBuffer+0],0A5A5A5A5h\n');end=text.index('RunOnce ENDP\n',start)
    tail=['cmp ProviderCalls,1','jne HostFailure','cmp word ptr [StateBuffer+10],0A5A5h','jne HostFailure','cmp word ptr [StateBuffer+38],0A5A5h','jne HostFailure']
    for at in range(48,64,4):tail += [f'cmp dword ptr [StateBuffer+{at}],0A5A5A5A5h','jne HostFailure']
    for at in range(0,64,4):tail += [f'cmp dword ptr [ReplyBuffer+{at}],0A5A5A5A5h','jne HostFailure']
    tail += ['fnstcw word ptr [ReplyBuffer+56]','fnstsw word ptr [ReplyBuffer+58]','stmxcsr dword ptr [ReplyBuffer+60]',
        'mov ax,word ptr [ReplyBuffer+56]','cmp ax,word ptr [InputBuffer+4]','jne HostFailure','test word ptr [ReplyBuffer+58],03800h','jnz HostFailure',
        'mov eax,dword ptr [ReplyBuffer+60]','and eax,0FFFFFFC0h','cmp eax,01F80h','jne HostFailure',
        'mov eax,dword ptr [FrameBuffer+0818h]','mov dword ptr [StateBuffer+40],eax','mov eax,dword ptr [FrameBuffer+07E4h]','mov dword ptr [StateBuffer+44],eax',
        'mov esi,OFFSET ClipBuffer+0B0h','mov edi,OFFSET ReplyBuffer','mov ecx,8','rep movsb',
        'mov esi,OFFSET StateBuffer','mov edi,OFFSET ReplyBuffer+8','mov ecx,10','rep movsb','mov word ptr [ReplyBuffer+18],0',
        'mov esi,OFFSET StateBuffer+12','mov edi,OFFSET ReplyBuffer+20','mov ecx,26','rep movsb','mov word ptr [ReplyBuffer+46],0',
        'mov esi,OFFSET StateBuffer+40','mov edi,OFFSET ReplyBuffer+48','mov ecx,8','rep movsb','ret']
    text=text[:start]+'\n'.join(tail)+'\n'+text[end:]
    start=text.index('HostMain PROC\n');end=text.index('MainLoop:\n',start)
    init=['HostMain PROC','cld','push -10','call _GetStdHandle@4','mov InputHandle,eax','push -11','call _GetStdHandle@4','mov OutputHandle,eax']
    pages={0x990000:0x20,0x9A0000:0x20,0x13C0000:2,0x1480000:2,0x78530000:2,0x785A0000:0x20,0x785B0000:2}
    for page in pages:init += ['push 4','push 03000h','push 010000h',f'push 0{page:X}h','call _VirtualAlloc@16',f'cmp eax,0{page:X}h','jne AllocationFailure','mov edi,eax','mov ecx,04000h','mov eax,0CCCCCCCCh','rep stosd']
    for a,b in CODE:init += [f'mov esi,OFFSET Original_{a:08X}',f'mov edi,0{a:X}h',f'mov ecx,{b-a}','rep movsb']
    for a,n in DATA:init += [f'mov esi,OFFSET Original_{a:08X}',f'mov edi,0{a:X}h',f'mov ecx,{n}','rep movsb']
    init += ['mov esi,OFFSET OriginalOne','mov edi,014861D0h','mov ecx,4','rep movsb','mov dword ptr [0785BD7B0h],1',
        'mov eax,OFFSET ModfProvider','mov dword ptr [013CC4C0h],eax']
    for at,label in EXITS.items():init += [f'mov byte ptr [0{at:X}h],0B8h',f'mov eax,OFFSET {label}',f'mov dword ptr [0{at+1:X}h],eax',f'mov word ptr [0{at+5:X}h],0E0FFh']
    for page,flags in pages.items():
        init += ['push OFFSET OldProtect',f'push 0{flags:X}h','push 010000h',f'push 0{page:X}h','call _VirtualProtect@16','test eax,eax','jz HostFailure']
        if flags==0x20:init += ['push 010000h',f'push 0{page:X}h','push -1','call _FlushInstructionCache@12','test eax,eax','jz HostFailure']
    text=text[:start]+'\n'.join(init)+'\n'+text[end:]
    start=text.index('MainLoop:\n');text=text[:start]+text[start:].replace('mov ecx,20\n','mov ecx,56\n').replace('mov ebx,28\n','mov ebx,64\n')
    return text

def linked_audit(path,mapping,images):
    old=(base.CODE,base.BUFFERS)
    try:
        base.CODE=CODE+[(a,a+n)for a,n in DATA];base.BUFFERS=BUFFERS
        return base.linked_audit(path,mapping,images)
    finally:base.CODE,base.BUFFERS=old
def main():
    assert sha(Path(base.__file__).read_bytes())==BASE_SHA and sha(Path(fixed.__file__).read_bytes())==FIXED_SHA
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True)
    p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    p.add_argument('--source-crt',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll'))
    p.add_argument('--audit-exe',type=Path);p.add_argument('--audit-map',type=Path);args=p.parse_args();args.out.resolve().relative_to((ROOT/'target').resolve())
    images=Images(args.source_exe,args.source_crt);proof=import_export_proof(images);source=audit_source(images);assembly=host_source(images)
    if args.audit_exe:
        m=json.loads((args.out/'manifest.json').read_text());assert m['preparation_tool_sha256']==sha(Path(__file__).read_bytes())and(args.out/'host-x86.asm').read_text()==assembly
        assert m['host_source_sha256']==sha((args.out/'host-x86.asm').read_bytes());result=linked_audit(args.audit_exe,args.audit_map,images)
        with(args.out/'linked-audit.json').open('x')as output:json.dump(result,output,indent=2)
    else:
        args.out.mkdir(parents=True,exist_ok=False);(args.out/'host-x86.asm').write_text(assembly)
        result=dict(schema='arithmetic-only-original-loop-clock-host-v1',source_sha256=SOURCE_SHA256,source_crt_sha256=CRT_SHA,
            import_export=proof,code_allowlist=source,source_bytes=sum(b-a for a,b in CODE),game_source_bytes=sum(b-a for a,b in APP_CODE),crt_source_bytes=sum(b-a for a,b in CRT_CODE),
            data_allowlist=[dict(address=f'{a:08X}',bytes=n,sha256=sha(images.read(a,n)))for a,n in DATA+[base.ONE]],
            host_source_sha256=sha((args.out/'host-x86.asm').read_bytes()),preparation_tool_sha256=sha(Path(__file__).read_bytes()),base_generator_sha256=BASE_SHA,fixed_failure_wrapper_sha256=FIXED_SHA,
            protocol=dict(input_bytes=64,projection_bytes=56,output_bytes=64,
                input='mode0@0,CW:u16@4,reserved0:u16@6,time32@8,unused64@12,duration64@20,frames32@28,loop1@32,arithmeticBoundary1@36,explicitSSE2policy1@40,reserved0@44..60',
                output='storedTime64@0,retainedQuotient80@8,zero16@18,firstCommit32@20,helperArg32@24,modfInteger64@28,modfFraction80@36,zero16@46,normalized32@48,frame32@52,CW:u16@56,SW:u16@58,MXCSR:u32@60'),
            domain='Finite nonnegative time32 including signedzero; positive finite duration64; finite committed quotient32; finite frames32>=1. Explicit loop1,arithmetic-only boundary1,SSE2dispatch1. Twelve masked x87 policies,MXCSR1F80. Negative/nonfinite helper branch and CRTlegacy/NaN dispatch excluded.',
            arithmetic_boundary='9A1CDB..CF1 computes retained quotient and first32 commit. Owned prefix captures both and discards the retained time operand, then reloads exact80 quotient and enters9A1D4E with loopAD1. Original comparison/counter/global-generation/event work is not executed or inferred. Caller must supply that state boundary separately. Final exit9A1DB2 before sampling callback.',
            crt_policy='Own readonly785BD7B0=1 explicitly selects original finite SSE2modf undermaskedCW/MXCSR1F80; the on-disk value0 is not a retail startup claim. Original helper IAT013CC4C0 forwards through a guarded owned provider to unchangedCRT785A3350, with exact source double/outpointer args and once-only return99BCC7. No replacement numeric modf implementation.',
            provider='Provider validates original ESPFrame7E0/return99BCC7/outptrFrame7F0, forwards exact threewords, enters CRT with returnModfReturn atFrame7D0, captures returned80 and integer64, reloads80 unchanged and returns to original helper. No Windows calls on fake stack; failures restore NativeStack before ExitProcess.',
            memory='Seven16byte-guarded buffers. Input/Resource/Context shadows,Clip whole shadow exceptB0/B4,Frame whole shadow except7C0..800/804/818. State exact unusedpadding checks; allReplypoison checked before output. ESP/DF/CW/TOP/MXCSRcontrols checked; zero/allonesXMMprojection agreement. Source payloads linkedR; executionpagesRX,dataR,gapsINT3.',
            exits={f'{a:08X}':n for a,n in EXITS.items()},compiled=False,host_executed=False,root_review_required=True,
            compile_commands=[['ml.exe','/nologo','/c','/coff','/Fohost-x86.obj','host-x86.asm'],['link.exe','/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/BASE:0x27000000','/DYNAMICBASE:NO','/SAFESEH:NO','/OUT:host-x86.exe','/MAP:host-x86.map','host-x86.obj','kernel32.lib']])
        (args.out/'manifest.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(dict(host_executed=False,source_bytes=sum(b-a for a,b in CODE),host_source_sha256=sha((args.out/'host-x86.asm').read_bytes())),indent=2))
if __name__=='__main__':main()
