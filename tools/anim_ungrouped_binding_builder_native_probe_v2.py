"""Owned-only fullbuilder memcpy maximum-count branch correction; preservev1."""
import sys,json,hashlib
from pathlib import Path
import anim_ungrouped_binding_builder_native_probe_v1 as base
BASE_SHA='c4b691a506e7ede73a2a839edfca250b295a479d1e970511b635e62b1c9f8c79'
OLD_ASM='563e4595094da421993b7f941b123a530268adc7a54ae1b4418870c1054f21aa'
BLOCKER='6c9a61c30f1cbf478cf737af5acfa094281d30527153a000add74dc10d6bb9c2'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 assert sha(base.__file__)==BASE_SHA
 original=base.assembly
 def corrected(out,p):
  text=original(out,p);assert hashlib.sha256(text.replace(chr(10),chr(13)+chr(10)).encode()).hexdigest()==OLD_ASM
  before='inc ProviderCalls\nmov edx,dword ptr [HeaderBuffer+8]\nshl edx,1\ncmp ProviderCalls,edx\njne HostFailure\npushfd'
  after=before.replace('jne HostFailure','ja HostFailure');assert text.count(before)==1
  return text.replace(before,after)
 base.assembly=corrected
 try:base.main()
 finally:base.assembly=original
 args=sys.argv[1:];out=Path(args[args.index('--out')+1])
 if '--owned-exe'in args:return
 manifest=json.loads((out/'manifest.json').read_text());manifest.update(schema='full-original9A1E00-prepared-ungrouped-source-entry-host-v2',tool_sha256=sha(__file__),base_tool_sha256=BASE_SHA,base_asm_sha256=OLD_ASM,asm_sha256=sha(out/'host-x86.asm'),owned_change='Onlypost-CMP ProviderCalls,2*channelcount JNE becomesJA. Thisisamaximumguard,notexacttotal. Originalsource/model/input/buffers/ABI/source addresses unchanged.',prior_blocked_peer_sha256=BLOCKER)
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
 r=dict(schema='fullbuilder-owned-provider-count-correction-v2',base_tool_sha256=BASE_SHA,base_asm_sha256=OLD_ASM,tool_sha256=sha(__file__),asm_sha256=sha(out/'host-x86.asm'),prior_blocked_peer_sha256=BLOCKER,source_bytes=13380,source_model_input_buffers_ABI_unchanged=True,oldpreparationspreserved=True,owned_change='Exactlyoneconditionalbranch JNE->JA afterINC ProviderCalls andCMPagainst2*channelcount. Declaredconstraintis<=2*C; firstvalidcopyis1 andmustpass.',compiled=False,native_executed=False,execution_admitted=False,open_gates=['freshrootcompile/linkandv2audit','actualdifferentworkerlinkedv2peer','newv2runnerandrunnerpeer','explicitroot-onlysingleattemptexecutionandRustcomparison'])
 (out/'owned-provider-correction.json').write_text(json.dumps(r,indent=2)+'\n')
if __name__=='__main__':main()
