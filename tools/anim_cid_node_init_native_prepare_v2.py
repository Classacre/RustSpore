"""Build the existing CID host carrier with source-grounded data audit; no execution."""
import argparse, json
from pathlib import Path
import anim_cid_node_init_native_prepare_v1 as v1
from cell_schema_probe import SourceImage
ROOT=v1.ROOT
EXPECTED_AUDIT={"data":[{"address":"014861D0","bytes":4}]}
def build(exe,mapping,image):
    spans=[(v1.source.START,v1.source.END),(v1.source.ONE,v1.source.ONE+4),(v1.source.END,v1.source.END+7)]
    assert all(b<=v1.BASE or a>=v1.BASE+4096 for a,b in spans)
    assert image.read(v1.source.ONE,4)==bytes.fromhex("0000803f")
    old=(v1.carrier.BASE,v1.carrier.source_pages)
    try:
        v1.carrier.BASE,v1.carrier.source_pages=v1.BASE,v1.pages
        return v1.carrier.build(exe,mapping,image,EXPECTED_AUDIT)
    finally:
        v1.carrier.BASE,v1.carrier.source_pages=old
def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--host-dir",type=Path,required=True);p.add_argument("--out",type=Path,required=True)
    a=p.parse_args();a.out.resolve().relative_to((ROOT/"target").resolve())
    image=SourceImage(Path("D:/Games/SPORE Collection/SporeBin/SporeApp.exe"))
    assert (a.host_dir/"host-x86.asm").read_text()==v1.host(image)
    blob,review=build(a.host_dir/"host-x86.exe",a.host_dir/"host-x86.map",image)
    a.out.mkdir(parents=True,exist_ok=False)
    (a.out/"carrier-x86.exe").write_bytes(blob)
    review.update(schema="CID-node-carrier-build-v2",source_audit=EXPECTED_AUDIT,native_executed=False,admitted_for_execution=False,carrier_sha256=v1.sha(a.out/"carrier-x86.exe"))
    (a.out/"carrier-build-review.json").write_text(json.dumps(review,indent=2)+"\n")
if __name__=="__main__":main()
