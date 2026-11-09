"""Prepare REVIEW-ONLY original-byte selector host inputs; never build or execute."""
import argparse, hashlib, json
from pathlib import Path
from capstone import Cs, CS_ARCH_X86, CS_MODE_32, CS_GRP_JUMP, CS_GRP_CALL
from cell_schema_probe import SourceImage, SOURCE_SHA256

def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
    ap=argparse.ArgumentParser(); ap.add_argument('exe',type=Path); ap.add_argument('out',type=Path); args=ap.parse_args()
    root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
    parent=args.out.parent.resolve(strict=True)
    if not parent.is_relative_to(root): raise ValueError('output parent must be under target')
    image=SourceImage(args.exe); start,end=0xe283a0,0xe28471
    raw=image.read(start,end-start); decoder=Cs(CS_ARCH_X86,CS_MODE_32); decoder.detail=True
    ins=list(decoder.disasm(raw,start))
    if not ins or ins[-1].address+ins[-1].size!=end: raise ValueError('incomplete decode')
    absolute=[]
    for i in ins:
        if i.group(CS_GRP_CALL): raise ValueError('unexpected call')
        if i.group(CS_GRP_JUMP):
            dest=i.operands[0].imm
            if not start<=dest<end: raise ValueError('branch outside slice')
        if '0x13f4f98' in i.op_str: absolute.append(i.address)
    if absolute != [0xe283c5,0xe28436,0xe2845f]: raise ValueError('unexpected absolute memory sites')
    args.out.mkdir()
    header='#pragma once\nstatic const unsigned char source_bytes[] = {'+','.join(f'0x{x:02x}' for x in raw)+'};\n'
    (args.out/'source_bytes.h').write_text(header,encoding='ascii')
    host=Path(__file__).with_name('cell_ui_selection_host_review_v1.c')
    adapter=Path(__file__).resolve().parent.parent/'crates/spore-core/examples/cell_ui_selection_compare_v1.rs'
    manifest={'scope':'review only; no native execution', 'source_pe_sha256':SOURCE_SHA256,
              'slice':'E283A0..E28470 inclusive', 'slice_sha256':hashlib.sha256(raw).hexdigest(),
              'header_sha256':digest(args.out/'source_bytes.h'), 'host_source_sha256':digest(host),
              'adapter_source_sha256':digest(adapter),'absolute_sites':[f'{a:08X}' for a in absolute],
              'absolute_site_admission':'unreachable only for five immutable words equal to 2 and sum 10',
              'instructions':[{'address':f'{i.address:08X}','bytes':i.bytes.hex(),'text':f'{i.mnemonic} {i.op_str}'} for i in ins]}
    (args.out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf8')
    print('review preparation',args.out,'slice sha256',manifest['slice_sha256'],'manifest sha256',digest(args.out/'manifest.json'))
if __name__=='__main__': main()
