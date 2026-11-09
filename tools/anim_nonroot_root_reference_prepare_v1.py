"""Prepare a source-body root input; no host, compiler, Cargo or native run."""
import hashlib,json,struct
from pathlib import Path
R=Path(__file__).resolve().parent.parent
OUT=R/'target/animation/nonroot-root-reference-preparation-v1'
PACK=R/'target/animation/nonroot-source-fixture-v1'
ROOTINPUT=R/'target/animation/root-pose-conversion-inputs-v1'
COMPARE=R/'target/animation/root-pose-sparse-carrier-native-run-v3/rust-comparison-v1.json'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 assert sha(PACK/'manifest.json')=='eeb9516a99f26b4ec167574c25ed811ba206e09151b092c4f90024519785a89a'
 assert sha(PACK/'root-definition-projection.bin')=='b6267eb00357c7390ae7292c89348e728490e47938a99d6db387a01d2c0dc9bd'
 manifest=json.loads((ROOTINPUT/'manifest.json').read_text())
 for name,expect in manifest['files'].items():assert sha(ROOTINPUT/name)==expect
 report=json.loads(COMPARE.read_text());assert report['accepted'] and report['source_cases']==22212 and report['pins_unchanged']['unchanged'] and report['projection_sha256']==report['native_projection_sha256']==report['rust_projection_sha256']
 meta=json.loads((ROOTINPUT/'input-index.json').read_text())[12]
 assert meta['key']==['Data/PatchData.package','71D0A3D5','EE17C6AD','00000000'] and meta['frame_bits']=='00000000' and meta['context']=='explicit_unit_reference' and meta['pass_index']==0 and meta['binding_words']==['00000001','00000000','0000001E','20FF0000']
 raw=(ROOTINPUT/'inputs.bin').read_bytes()[12*144:13*144];assert hashlib.sha256(raw).hexdigest()==meta['input_sha256'];word=list(struct.unpack('<36I',raw))
 body=(PACK/'root-definition-projection.bin').read_bytes();assert struct.unpack_from('<I',body,4)[0]==9
 fields=[0x104,0x108,0x1f8,0x1fc,0x200,0x21c,0x208,0x10c,0x110,0x114,0x118,0x11c,0x120,0x124,0x128,0x12c,0x130,0x134,0x138,0x13c,0x140,0x144,0x148,0x14c,0x344,0x348,0x34c,0x350,0x354,0x358,0x35c,0x360,0x364,0x368,0x36c,0x378,0x37c,0x204,0x20c]
 get=lambda o:struct.unpack_from('<I',body,168+4*fields.index(o))[0]
 xyz=[get(o) for o in (0x10c,0x110,0x114)];q=[get(o) for o in (0x118,0x11c,0x120,0x124)]
 word[10]=9;word[12:15]=xyz;word[16:20]=q;word[20:23]=xyz;word[23:27]=q
 request=struct.pack('<36I',*word)
 OUT.mkdir(parents=True,exist_ok=False);(OUT/'pending-root-input.bin').write_bytes(request)
 paths=[Path(__file__),PACK/'manifest.json',PACK/'root-definition-projection.bin',ROOTINPUT/'manifest.json',ROOTINPUT/'input-index.json',ROOTINPUT/'inputs.bin',COMPARE]
 result={'schema':'nonroot-root-reference-preparation-v1','compiled':False,'native_executed':False,'admitted_for_execution':False,'source_row':meta,'root_position_bits':[f'{x:08X}' for x in xyz],'root_quaternion_bits':[f'{x:08X}' for x in q],'root_nonzero_mask':9,'input_sha256':hashlib.sha256(request).hexdigest(),'source_sample_words':[f'{x:08X}' for x in word[27:]],'assumptions':'explicit unit scale/dimensions/blend and binding selected pass 0; root record processed before nonroot; snapshot binding flags stable; no live actor','limitations':'existing root host rejects mask 9; this request is unexecuted and has no accepted result; requires reviewed host, 99CB70 normalization and intervening bindings before use','pins':{str(p.relative_to(R)):sha(p) for p in paths}}
 (OUT/'manifest.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ('input_sha256','root_position_bits','root_quaternion_bits','source_sample_words','native_executed')},indent=2))
if __name__=='__main__':main()
