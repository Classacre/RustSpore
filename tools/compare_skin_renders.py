"""Compare identically framed RW4 GPU/CPU renders and a GPU bind reference.

This checks renderer agreement, not retail visual fidelity or source geometry
through unrelated camera framing. Numerical source-space checks are printed by
rustspore --animate. Requires Pillow, like the other local image probes.
"""
import argparse
import json
from pathlib import Path
from PIL import Image, ImageChops


def difference(a, b, top):
    delta = ImageChops.difference(a, b).crop((0, top, a.width, a.height))
    pixels = list(delta.get_flattened_data())
    return {"pixels_over_2": sum(max(p) > 2 for p in pixels),
            "max_channel_error": max(map(max, pixels), default=0),
            "compared_pixels": len(pixels)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("gpu_pose", type=Path)
    parser.add_argument("cpu_pose", type=Path)
    parser.add_argument("gpu_bind", type=Path)
    parser.add_argument("--skip-top", type=int, default=180,
                        help="Exclude the HUD; all images must use the same camera/window")
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    images = [Image.open(p).convert("RGB") for p in (args.gpu_pose,args.cpu_pose,args.gpu_bind)]
    if any(im.size != images[0].size for im in images) or not 0 <= args.skip_top < images[0].height:
        parser.error("image sizes differ or crop is out of bounds")
    reference = difference(images[0], images[1], args.skip_top)
    movement = difference(images[0], images[2], args.skip_top)
    # Allow isolated raster edges; reject a missing render or motion hidden in
    # unrelated UI text. This is combined with source-space displacement checks.
    passed = reference["pixels_over_2"] <= 20 and movement["pixels_over_2"] >= 100
    result = {"passed": passed, "size": images[0].size,"skip_top":args.skip_top,
              "gpu_cpu": reference,"gpu_pose_bind":movement,
              "inputs":[str(p.resolve()) for p in (args.gpu_pose,args.cpu_pose,args.gpu_bind)]}
    args.out.parent.mkdir(parents=True,exist_ok=True)
    args.out.write_text(json.dumps(result,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(result))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
