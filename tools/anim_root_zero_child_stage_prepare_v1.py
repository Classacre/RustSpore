"""Prepare an unlinked full-entry zero-child root-stage host core; never execute.
This packet deliberately has no transport/PE admission until carrier integration.
"""
import argparse,json,hashlib,shutil
from pathlib import Path
from cell_native_math_probe import db
ROOT=Path(__file__).resolve().parent.parent
SOURCE=ROOT/'target/animation/root-zero-child-staging-source-v2-reviewed'
CLOSURE_SHA='4927bfdc7f7d524583a9d886aae0080c71d3b363a015aa787fc9e74b072a4a82'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def prepare(out):
 assert sha(SOURCE/'source-closure.json')==CLOSURE_SHA
 proof=json.loads((SOURCE/'source-closure.json').read_text());out.mkdir(parents=True,exist_ok=False)
 lines=['.686p','.xmm','.model flat','option casemap:none','.const']
 for r in proof['source_records']:
  for span in r['ranges']:
   p=SOURCE/span['blob'];assert sha(p)==span['sha256'];shutil.copyfile(p,out/p.name)
   lines+=db('Original_'+span['start'],p.read_bytes())
 for r in proof['data']:
  p=SOURCE/('data-'+r['address']+'.bin');assert sha(p)==r['sha256'];shutil.copyfile(p,out/p.name);lines+=db('Data_'+r['address'],p.read_bytes())
 sizes={'Input':1256,'Root':700,'Definition':524,'Context':128,'Body':748,'Orientation':16,'DefaultPosition':12,'Reply':736}
 lines+=['.data','NativeStack DWORD 0','RequestedMxcsr DWORD 01F80h','Poison DWORD 0']
 for name,n in sizes.items():
  lines += [f'PUBLIC {name}Before,{name}Buffer,{name}After',f'{name}Before DWORD 4 DUP(0A5A5A5A5h)',f'{name}Buffer BYTE {n} DUP(0)',f'{name}After DWORD 4 DUP(0A5A5A5A5h)',f'{name}Shadow BYTE {n} DUP(0)']
 lines+=['.code','PUBLIC StageOwnedStart,StageOwnedEnd,ValidateStage,RunStage,StageFailure','StageOwnedStart LABEL BYTE','ValidateStage PROC']
 lines+=['mov eax,dword ptr [InputBuffer]','and eax,0FFFFF0FFh','cmp eax,07Fh','jne StageFailure','mov eax,dword ptr [InputBuffer]','and eax,0300h','cmp eax,0100h','je StageFailure',
 'cmp dword ptr [InputBuffer+4],0','jne StageFailure','cmp dword ptr [InputBuffer+8],1','jne StageFailure','cmp dword ptr [InputBuffer+20],1','ja StageFailure','cmp dword ptr [InputBuffer+28],0','jne StageFailure',
 'cmp dword ptr [InputBuffer+1252],0','jne StageFailure']
 for at in range(32,48,4):lines += [f'cmp dword ptr [InputBuffer+{at}],0','jne StageFailure']
 finite=[12,16,24]+list(range(32+0x10,32+0x38,4))+list(range(32+0x3c,32+0xa0,4))+list(range(732+0x10c,732+0x128,4))
 for at in finite:lines += [f'mov eax,dword ptr [InputBuffer+{at}]','and eax,07F800000h','cmp eax,07F800000h','je StageFailure']
 lines+=['test dword ptr [InputBuffer+12],080000000h','jnz StageFailure','ret','ValidateStage ENDP','RunStage PROC','mov NativeStack,esp','cld','call ValidateStage']
 for name,n in sizes.items():
  if name=='Input':continue
  lines += [f'mov edi,OFFSET {name}Buffer',f'mov ecx,{n}','xor eax,eax','rep stosb']
 lines+=['mov esi,OFFSET InputBuffer+32','mov edi,OFFSET RootBuffer','mov ecx,700','rep movsb','mov esi,OFFSET InputBuffer+732','mov edi,OFFSET DefinitionBuffer','mov ecx,524','rep movsb',
 'mov dword ptr [RootBuffer],OFFSET DefinitionBuffer','mov eax,dword ptr [InputBuffer+24]','mov dword ptr [ContextBuffer+70h],eax','mov dword ptr [BodyBuffer+2E4h],OFFSET RootBuffer','mov dword ptr [BodyBuffer+2E8h],OFFSET RootBuffer+700']
 for name,n in sizes.items():
  if name in ('Root','Orientation','DefaultPosition','Reply'):continue
  lines += [f'mov esi,OFFSET {name}Buffer',f'mov edi,OFFSET {name}Shadow',f'mov ecx,{n}','rep movsb']
 lines+=['fninit','fldcw word ptr [InputBuffer]','ldmxcsr RequestedMxcsr','movd xmm0,Poison','pshufd xmm0,xmm0,0']
 for i in range(1,8):lines += [f'movaps xmm{i},xmm0']
 lines += ['push OFFSET DefaultPositionBuffer','push OFFSET OrientationBuffer','push OFFSET RootBuffer','push OFFSET ContextBuffer','mov eax,009E82E0h','call eax','add esp,16','cmp esp,NativeStack','jne StageFailure',
 'mov esi,OFFSET RootBuffer','mov edi,OFFSET RootShadow','mov ecx,700','rep movsb','push OFFSET RootBuffer','push OFFSET ContextBuffer','mov eax,009E8930h','call eax','add esp,8','cmp esp,NativeStack','jne StageFailure',
 'mov esi,OFFSET RootBuffer','mov edi,OFFSET RootShadow','mov ecx,700','repe cmpsb','jne StageFailure',
 'push dword ptr [InputBuffer+20]','push OFFSET BodyBuffer','push dword ptr [InputBuffer+16]','push dword ptr [InputBuffer+12]','mov eax,009BD370h','call eax','add esp,16','cmp esp,NativeStack','jne StageFailure']
 for name,n in sizes.items():
  if name in ('Root','Orientation','DefaultPosition','Reply'):continue
  lines += [f'mov esi,OFFSET {name}Buffer',f'mov edi,OFFSET {name}Shadow',f'mov ecx,{n}','repe cmpsb','jne StageFailure']
 for name in sizes:
  for side in('Before','After'):
   for at in range(0,16,4):lines += [f'cmp dword ptr [{name}{side}+{at}],0A5A5A5A5h','jne StageFailure']
 lines += ['pushfd','pop eax','test eax,0400h','jnz StageFailure','mov esi,OFFSET RootBuffer','mov edi,OFFSET ReplyBuffer','mov ecx,700','rep movsb','mov esi,OFFSET OrientationBuffer','mov ecx,16','rep movsb','mov esi,OFFSET DefaultPositionBuffer','mov ecx,12','rep movsb',
 'fnstcw word ptr [ReplyBuffer+728]','fnstsw word ptr [ReplyBuffer+730]','stmxcsr dword ptr [ReplyBuffer+732]','mov ax,word ptr [InputBuffer]','cmp ax,word ptr [ReplyBuffer+728]','jne StageFailure','test word ptr [ReplyBuffer+730],03800h','jnz StageFailure','mov eax,dword ptr [ReplyBuffer+732]','and eax,0FFFFFFC0h','cmp eax,01F80h','jne StageFailure',
 'xor eax,eax','ret','RunStage ENDP','StageFailure PROC','cmp NativeStack,0','je NoStageStack','mov esp,NativeStack','NoStageStack:','mov eax,71','ret','StageFailure ENDP','StageOwnedEnd LABEL BYTE','END']
 (out/'stage-host-core-x86.asm').write_text('\n'.join(lines)+'\n')
 manifest=dict(schema='zero-child-root-stage-full-entry-host-core-preparation-v1',tool_sha256=sha(__file__),source_closure_sha256=CLOSURE_SHA,asm_sha256=sha(out/'stage-host-core-x86.asm'),source_bytes=5901,input_bytes=1256,model_output_bytes=728,reply_bytes=736,sizes=sizes,protocol='CW32@0,actualChildCount0@4,poseCount1@8,dt32@12,smoothing32@16,skipCommitBool@20,contextScale32@24,reserved0@28,root2BC@32,definition20C@732. Raw root pointer words0..C=0; definition+208=0 required; owned graph pointers bound after validation.',ABI={'009E82E0':['context*','root*','orientationOut16*','defaultPositionOut12*'],'009E8930':['context*','root*'],'009BD370':['dt32','smoothing32','body*','skipCommitBool']},compiled=False,native_executed=False,execution_admitted=False,missing=['Full carrier/transport integration and startup byte checks; this is an unlinked owned core fragment only.','Outer dual-XMMpoison execution/728Bprojection comparison.','Independent reference implementation and actual accepted accumulator fixture join.','Actual linked instruction/guard/page/import review and peer approval.'],caller_contract='RunStage enters on real owned stack; failure restores NativeStack and returns71 to owned outer driver, success0. No Windows API/source calls on fake stack. Original functions full cdecl entries and caller cleanup; no partial-GPR preservation claim.',domain='Genuine supplied definition childcount0 and exactlyone root record. Nonzero topology rejected, never rewritten. Finite used float fields; dt signbitclear. Contexts explicit, no actor association.')
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());prepare(a.out)
if __name__=='__main__':main()
