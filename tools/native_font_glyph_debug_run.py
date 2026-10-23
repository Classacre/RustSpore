"""Prepare or execute the reviewed font fixture under explicit debug startup.

Only the owned no-CRT child is started, hidden with regular-file stdin. Original
source code is never patched. Three owned admission breakpoints are restored
before source release. The only accepted WOW64 EFLAGS serialization difference
is the architectural fixed-one bit proven by the retained owned PUSHFD fixture.
No retail process, desktop interaction or original module loading is used.
"""
import argparse
import stat
import sys
import time
import ctypes as c
from ctypes import wintypes as w
import hashlib
import json
import msvcrt
import os
from pathlib import Path
import struct
import subprocess

root = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--request-file', type=Path, required=True)
parser.add_argument('--out', type=Path, required=True)
parser.add_argument('--peer-review', type=Path)
parser.add_argument('--execute-reviewed', action='store_true')
args = parser.parse_args()
request_path = args.request_file.resolve()
request_path.relative_to((root/'target').resolve())
assert not args.request_file.is_symlink() and stat.S_ISREG(request_path.stat().st_mode)
sys.path.insert(0,str(root/'tools'))
parser_path = root/'tools/native_font_glyph_reply.py'
assert hashlib.sha256(parser_path.read_bytes()).hexdigest()=='7c949b53450632cf64076028ca477af56a7f74dccc63c881524fc8a0aeb7cdde'
from native_font_glyph_reply import parse_reply,parse_request
request = request_path.read_bytes()
expected = parse_request(request)
bundle = root / 'target/source-ui/font-glyph-native-v2'
exe = bundle / 'host-x86.exe'
assert hashlib.sha256(exe.read_bytes()).hexdigest() == '2d6cffaa8b5a0f9a839def8abc06858128d9296995b06d478f7b516cccf3ead5'
out = args.out.resolve()
out.relative_to((root/'target').resolve())
out.mkdir(parents=True,exist_ok=False)
(out/'request.bin').write_bytes(request)
k = c.WinDLL('kernel32', use_last_error=True)
ps = c.WinDLL('psapi', use_last_error=True)

class SI(c.Structure):
    _fields_ = [('cb', w.DWORD), ('reserved', w.LPWSTR), ('desktop', w.LPWSTR),
        ('title', w.LPWSTR), ('x', w.DWORD), ('y', w.DWORD), ('xs', w.DWORD),
        ('ys', w.DWORD), ('xc', w.DWORD), ('yc', w.DWORD), ('fill', w.DWORD),
        ('flags', w.DWORD), ('show', w.WORD), ('cb2', w.WORD), ('res2', c.c_void_p),
        ('inp', w.HANDLE), ('out', w.HANDLE), ('err', w.HANDLE)]

class PI(c.Structure):
    _fields_ = [('process', w.HANDLE), ('thread', w.HANDLE), ('pid', w.DWORD), ('tid', w.DWORD)]

class MBI(c.Structure):
    _fields_ = [('base', c.c_void_p), ('allocation_base', c.c_void_p),
        ('allocation_protect', w.DWORD), ('partition', w.WORD), ('size', c.c_size_t),
        ('state', w.DWORD), ('protect', w.DWORD), ('type', w.DWORD)]

class DE(c.Structure):
    _fields_ = [('code', w.DWORD), ('pid', w.DWORD), ('tid', w.DWORD),
               ('padding', w.DWORD), ('data', c.c_ubyte * 160)]

k.CreateProcessW.argtypes = [w.LPCWSTR, w.LPWSTR, c.c_void_p, c.c_void_p, w.BOOL,
    w.DWORD, c.c_void_p, w.LPCWSTR, c.POINTER(SI), c.POINTER(PI)]
k.CreateProcessW.restype = w.BOOL
k.VirtualQueryEx.argtypes = [w.HANDLE, c.c_void_p, c.POINTER(MBI), c.c_size_t]
k.VirtualQueryEx.restype = c.c_size_t
k.VirtualProtectEx.argtypes = [w.HANDLE, c.c_void_p, c.c_size_t, w.DWORD, c.POINTER(w.DWORD)]
k.VirtualProtectEx.restype = w.BOOL
for name in ['ReadProcessMemory', 'WriteProcessMemory']:
    fn = getattr(k, name)
    fn.argtypes = [w.HANDLE, c.c_void_p, c.c_void_p, c.c_size_t, c.POINTER(c.c_size_t)]
    fn.restype = w.BOOL
k.FlushInstructionCache.argtypes = [w.HANDLE, c.c_void_p, c.c_size_t]
k.FlushInstructionCache.restype = w.BOOL
k.WaitForDebugEvent.argtypes = [c.POINTER(DE), w.DWORD]
k.WaitForDebugEvent.restype = w.BOOL
k.ContinueDebugEvent.argtypes = [w.DWORD, w.DWORD, w.DWORD]
k.ContinueDebugEvent.restype = w.BOOL
k.Wow64GetThreadContext.argtypes = [w.HANDLE, c.c_void_p]
k.Wow64GetThreadContext.restype = w.BOOL
k.Wow64SetThreadContext.argtypes = [w.HANDLE, c.c_void_p]
k.Wow64SetThreadContext.restype = w.BOOL
k.TerminateProcess.argtypes = [w.HANDLE, w.UINT]
k.CloseHandle.argtypes = [w.HANDLE]
ps.GetMappedFileNameW.argtypes = [w.HANDLE, c.c_void_p, w.LPWSTR, w.DWORD]
ps.GetMappedFileNameW.restype = w.DWORD
pi = PI()

def read(address, size):
    data = c.create_string_buffer(size)
    done = c.c_size_t()
    assert k.ReadProcessMemory(pi.process, address, data, size, c.byref(done)), c.get_last_error()
    assert done.value == size
    return data.raw

def patch(address, data):
    previous = w.DWORD()
    assert k.VirtualProtectEx(pi.process, address, len(data), 0x40, c.byref(previous)), c.get_last_error()
    try:
        done = c.c_size_t()
        assert k.WriteProcessMemory(pi.process, address, c.create_string_buffer(data), len(data), c.byref(done)), c.get_last_error()
        assert done.value == len(data)
        assert k.FlushInstructionCache(pi.process, address, len(data)), c.get_last_error()
    finally:
        scratch = w.DWORD()
        assert k.VirtualProtectEx(pi.process, address, len(data), previous.value, c.byref(scratch)), c.get_last_error()

def region(address):
    info = MBI()
    assert k.VirtualQueryEx(pi.process, address, c.byref(info), c.sizeof(info)) == c.sizeof(info), c.get_last_error()
    name = c.create_unicode_buffer(2048)
    size = ps.GetMappedFileNameW(pi.process, info.allocation_base or address, name, len(name)) if info.type else 0
    return dict(queried=f'{address:08X}', base=f'{info.base or 0:08X}',
        allocation_base=f'{info.allocation_base or 0:08X}', size=info.size,
        state=f'{info.state:08X}', protect=f'{info.protect:08X}', type=f'{info.type:08X}',
        mapped_file=name.value if size else None)


import sys
sys.path.insert(0,str(root/'tools'))
from native_crt_acos_probe import PE
assert hashlib.sha256((root/'tools/native_font_glyph_reply.py').read_bytes()).hexdigest()=='7c949b53450632cf64076028ca477af56a7f74dccc63c881524fc8a0aeb7cdde'
from native_font_glyph_reply import parse_reply,parse_request
points={0x24002041:'reservation_failure',0x2400202f:'commit_failure',0x24002013:'before_first_native_entry'}
image=PE(exe)
original={at:image.read(at,1) for at in points}
lock=json.loads((bundle/'bundle-lock.json').read_text())
pins={str(bundle/row['path']):row['sha256'] for row in lock['files']}
pins[str(bundle/'bundle-lock.json')]=hashlib.sha256((bundle/'bundle-lock.json').read_bytes()).hexdigest()
manifest=json.loads((bundle/'manifest.json').read_text())
for path_key,sha_key in [('source_path','source_sha256'),('crt_path','crt_sha256'),('font_path','font_sha256')]:
    path=Path(manifest[path_key]);pins[str(path)]=manifest[sha_key]
for row in manifest['analysis_reports'].values():pins[str(root/row['path'])]=row['sha256']
for path in [Path(__file__),parser_path,request_path,out/'request.bin']:
    pins[str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
if args.execute_reviewed:
    assert args.peer_review is not None,'Execution requires the independent runner review'
    review_path=args.peer_review.resolve();review_path.relative_to((root/'target').resolve())
    review=json.loads(review_path.read_text())
    assert review['runner_sha256']==hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    pins[str(review_path)]=hashlib.sha256(review_path.read_bytes()).hexdigest()
for path,sha in pins.items():assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==sha
report=dict(schema='reviewed-font-debug-startup-run-v1',host_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(),
    hidden=True,regular_file_stdin=True,debug_only_this_process=True,source_code_patched=False,
    source_data_EIF_patch='Reviewed frozen v2 four-byte pointer substitution to owned one-byte RET.',
    native_font_entry_started=False,request=expected,executed=args.execute_reviewed,events=[],owned_breakpoint_original_bytes={f'{at:08X}':data.hex() for at,data in original.items()})
proof_path=root/'target/source-ui/font-map-debug-run-v7-attempt2/report.json'
assert hashlib.sha256(proof_path.read_bytes()).hexdigest()=='2f57b69efc47c1fcc7d1a1626340ce9938bdee10d7a09303857a77a9e15ea7f9'
proof=json.loads(proof_path.read_text())
assert proof['diagnostic_completed'] and proof['owned_PUSHFD_flags']==proof['context_before']['EFLAGS']=='00000202'
assert not proof['source_call_executed'] and proof['frozen_inputs_unchanged']
report['reserved_bit1_preflight']=dict(path=str(proof_path),sha256=hashlib.sha256(proof_path.read_bytes()).hexdigest(),actual_PUSHFD_flags=proof['owned_PUSHFD_flags'])
pins[str(proof_path)]=hashlib.sha256(proof_path.read_bytes()).hexdigest()
(out/'pins-v1.json').write_text(json.dumps(pins,indent=2)+'\n')
if not args.execute_reviewed:
    (out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(prepared=True,executed=False,request=expected)))
    raise SystemExit(0)
started=time.monotonic()
stage=0
startup32_seen=False
exit_code=None
try:
    with (out/'request.bin').open('rb') as inp,(out/'reply.bin').open('wb') as output,(out/'stderr.bin').open('wb') as err:
        si=SI();si.cb=c.sizeof(si);si.flags=0x100
        si.inp,si.out,si.err=[msvcrt.get_osfhandle(f.fileno()) for f in [inp,output,err]]
        for handle in [si.inp,si.out,si.err]:os.set_handle_inheritable(handle,True)
        cmd=c.create_unicode_buffer(subprocess.list2cmdline([str(exe)]))
        assert k.CreateProcessW(str(exe),cmd,None,None,True,0x08000002,None,str(bundle),c.byref(si),c.byref(pi)),c.get_last_error()
        for _ in range(128):
            assert time.monotonic()-started<30,'Whole-run debugger timeout'
            event=DE();assert k.WaitForDebugEvent(c.byref(event),min(5000,max(1,int((30-(time.monotonic()-started))*1000)))),c.get_last_error()
            assert event.pid==pi.pid, 'Unexpected debug process'
            data=bytes(event.data);code=int.from_bytes(data[:4],'little') if event.code==1 else None
            address=int.from_bytes(data[16:24],'little') if event.code==1 else None
            report['events'].append(dict(event=event.code,exception=code,address=None if address is None else f'{address:08X}'))
            disposition=0x10002
            if event.code in (3,6):
                handle=int.from_bytes(data[:8],'little')
                if handle:k.CloseHandle(handle)
            if event.code==5:
                exit_code=int.from_bytes(data[:4],'little')
                break
            if event.code==1:
                if stage==0:
                    assert code==0x80000003 and address>0x70000000
                    assert region(address)['mapped_file'].lower().endswith('ntdll.dll')
                    for at in points:
                        assert read(at,1)==original[at]
                        patch(at,b'\xcc')
                    stage=1
                elif stage==1 and address in points and code in (0x80000003,0x4000001f):
                    assert event.tid==pi.tid, 'Unexpected breakpoint thread'
                    context=(c.c_ubyte*736)();ptr=(c.addressof(context)+15)&~15
                    c.c_uint32.from_address(ptr).value=0x10007
                    assert k.Wow64GetThreadContext(pi.thread,ptr),c.get_last_error()
                    before=list(struct.unpack('<179I',c.string_at(ptr,716)))
                    eax,esi,esp=before[44],before[40],before[49]
                    if points[address]!='before_first_native_entry':
                        report['mapping_failure']=dict(stage=points[address],actual_return=f'{eax:08X}',ESI=esi)
                        raise RuntimeError('Debug-created child also hit mapping failure; no source call continued')
                    assert before[46]==address+1 and eax==0x8a6e60
                    for at in points:
                        assert read(at,1)==b'\xcc'
                        patch(at,original[at])
                        assert read(at,1)==original[at] and region(at)['protect']=='00000020'
                    c.c_uint32.from_address(ptr+184).value=address
                    assert k.Wow64SetThreadContext(pi.thread,ptr),c.get_last_error()
                    check=(c.c_ubyte*736)();check_ptr=(c.addressof(check)+15)&~15
                    c.c_uint32.from_address(check_ptr).value=0x10007
                    assert k.Wow64GetThreadContext(pi.thread,check_ptr),c.get_last_error()
                    after=list(struct.unpack('<179I',c.string_at(check_ptr,716)))
                    names=['SegGs','SegFs','SegEs','SegDs','EDI','ESI','EBX','EDX','ECX','EAX','EBP','EIP','SegCs','EFLAGS','ESP','SegSs']
                    report['context_before']={name:f'{before[i]:08X}' for i,name in enumerate(names,35)}
                    report['context_after']={name:f'{after[i]:08X}' for i,name in enumerate(names,35)}
                    report['context_changes']=[dict(register=name,before=f'{before[i]:08X}',after=f'{after[i]:08X}') for i,name in enumerate(names,35) if before[i]!=after[i]]
                    assert after[46]==address
                    assert all(after[i]==before[i] for i in range(35,51) if i not in (46,48))
                    # V7 actual PUSHFD proves the fixed-one reserved bit remains1.
                    # Every architectural flag still compares exactly.
                    assert before[48]&2 and (after[48]|2)==before[48]
                    report['WOW64_reserved_bit1_only_serialization_delta']=after[48]^before[48]
                    report['continuation']=dict(all_owned_bytes_restored=True,all_owned_pages_RX=True,
                        all_GPR_segments_architectural_EFLAGS_ESP_preserved=True,EIP=f'{address:08X}',EAX=f'{eax:08X}')
                    report['native_font_entry_started']=True
                    report['native_entry_marker_scope']='Authorized to continue; actual source completion requires reply/setjmp evidence.'
                    stage=2
                elif stage==1 and not startup32_seen and code==0x4000001f and address>0x70000000:
                    assert region(address)['mapped_file'].lower().endswith('ntdll.dll')
                    startup32_seen=True
                else:
                    report.setdefault('unexpected_exceptions',[]).append(dict(code=f'{code:08X}',address=f'{address:08X}'))
                    # Frozen application VEH terminates faults; do not swallow
                    # source INT3 admission traps or any post-startup exception.
                    disposition=0x80010001
                    if len(report['unexpected_exceptions'])>1:raise RuntimeError('Repeated unexpected own-child exception')
            assert k.ContinueDebugEvent(event.pid,event.tid,disposition),c.get_last_error()
        assert exit_code is not None,'No child termination within bounded debugger event sequence'
    report['exit']=exit_code
    reply=(out/'reply.bin').read_bytes()
    exports=[]
    parsed=parse_reply(reply,expected,active_sink=lambda item,pixels:exports.append((item,pixels)))
    active=out/'active';active.mkdir()
    for item,pixels in exports:
        (active/f"glyph-{item['ordinal']:02d}-{item['glyph_index']:03d}.alpha").write_bytes(pixels)
    (out/'parsed.json').write_text(json.dumps(parsed,indent=2)+'\n')
    report['glyph_success']=parsed['success'];report['native_error']=parsed.get('native_error')
    report['reply_bytes']=len(reply)
    assert exit_code==0 and not (out/'stderr.bin').read_bytes() and parsed['success'],'Native fixture did not complete successfully'
except BaseException as error:
    report['wrapper_error']=repr(error)
    raise
finally:
    report['completed_stage']=stage
    report['elapsed_seconds']=time.monotonic()-started
    changed=[path for path,sha in pins.items() if hashlib.sha256(Path(path).read_bytes()).hexdigest()!=sha]
    report['frozen_inputs_unchanged']=not changed;report['changed_files']=changed
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    if pi.process:
        k.TerminateProcess(pi.process,0);k.CloseHandle(pi.thread);k.CloseHandle(pi.process)
    assert not changed
print(json.dumps(report,indent=2))
