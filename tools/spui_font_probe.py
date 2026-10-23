"""Independent original UI SFNT design-data comparison, using FontTools.

This reads the installed fonts without selecting a native fallback, scaling,
shaping, hinting or rasterizing. Original font bytes are never written.
"""
import argparse
import hashlib
import io
import json
from pathlib import Path

import fontTools
from fontTools.ttLib import TTFont

from dbpf_extract import payload
from gmdl_skin_probe import indexes

FACES = {0x511B49CA, 0x61639DFB, 0x6BBC4E0E, 0xBACAAAA5, 0xFFE355DF}


def escaped(text):
    return text.replace('\\', '\\\\').replace('\t', '\\t').replace('\r', '\\r').replace('\n', '\\n')


def source(package):
    rows, faces = [], []
    glyphs = mapped = 0
    for entry in indexes(package):
        if (entry['type'] != 0x027C5CEF or entry['instance'] >> 32 != 0x0248E873
                or entry['instance'] & 0xffffffff not in FACES):
            continue
        raw = payload(package, entry)
        key = f'{entry["group"]:08X}:{entry["instance"]:016X}'
        with TTFont(io.BytesIO(raw), recalcBBoxes=False, recalcTimestamp=False) as font:
            head, hhea, maxp = (font[k] for k in ('head', 'hhea', 'maxp'))
            rows.append('\t'.join(map(str, [key, 'head', head.unitsPerEm, maxp.numGlyphs,
                hhea.ascent, hhea.descent, hhea.lineGap, head.xMin, head.yMin, head.xMax, head.yMax])))
            for name in font['name'].names:
                if (name.platformID == 3 and name.platEncID == 1 and name.langID == 0x409
                        and name.nameID in (1, 2, 4, 6)):
                    text = name.toUnicode(errors='strict')
                    rows.append(f'{key}\tname\t{name.platformID}\t{name.platEncID}\t{name.langID:04X}\t{name.nameID}\t{escaped(text)}')
            for cmap in font['cmap'].tables:
                rows.append(f'{key}\tcmap-record\t{cmap.platformID}\t{cmap.platEncID}\t{cmap.format}')
                if cmap.format != 4:
                    rows.append(f'{key}\tcmap-unsupported\t{cmap.platformID}\t{cmap.platEncID}\t{cmap.format}')
                    continue
                for code, name in sorted(cmap.cmap.items()):
                    glyph = font.getGlyphID(name)
                    if glyph:
                        assert 0 <= code <= 0xffff
                        rows.append(f'{key}\tcmap\t{cmap.platformID}\t{cmap.platEncID}\t{code:04X}\t{glyph}')
                        mapped += 1
            locations = font['loca'].locations
            for glyph, name in enumerate(font.getGlyphOrder()):
                advance, bearing = font['hmtx'].metrics[name]
                if locations[glyph] == locations[glyph + 1]:
                    bbox = 'empty'
                else:
                    g = font['glyf'][name]
                    bbox = '\t'.join(str(getattr(g, f)) for f in ('xMin', 'yMin', 'xMax', 'yMax'))
                rows.append(f'{key}\tglyph\t{glyph}\t{advance}\t{bearing}\t{bbox}')
                glyphs += 1
            family = [n.toUnicode(errors='strict') for n in font['name'].names
                      if n.platformID == 3 and n.platEncID == 1 and n.langID == 0x409 and n.nameID == 1]
            faces.append({'key': key, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(),
                          'family': family, 'glyphs': maxp.numGlyphs, 'units_per_em': head.unitsPerEm})
    assert len(faces) == 5, f'expected five source faces, found {len(faces)}'
    rows.sort()
    content = ('\n'.join(rows) + '\n').encode('utf8')
    return content, {'package': str(package), 'fonttools_version': fontTools.__version__,
        'faces': faces, 'glyphs': glyphs, 'format4_mapped': mapped, 'rows': len(rows),
        'sorted_lf_sha256': hashlib.sha256(content).hexdigest(),
        'scope': 'SFNT design data only; no native fallback, layout, glyph rasterization or GPU upload'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('package', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--compare', type=Path)
    args = parser.parse_args()
    content, report = source(args.package)
    if args.compare:
        actual = args.compare.read_bytes().replace(b'\r\n', b'\n')
        report['rust_rows'] = len(actual.splitlines())
        report['rust_sorted_lf_sha256'] = hashlib.sha256(actual).hexdigest()
        report['exact_match'] = actual == content
        if actual != content:
            expected_lines, actual_lines = content.splitlines(), actual.splitlines()
            report['first_difference'] = next(({'line': i + 1, 'python': e.decode('utf8'), 'rust': a.decode('utf8')}
                for i, (e, a) in enumerate(zip(expected_lines, actual_lines)) if e != a), {'rows_differ': True})
    args.out.write_bytes(content)
    args.report.write_text(json.dumps(report, indent=2) + '\n', encoding='utf8')
    print(json.dumps(report, indent=2))
    if args.compare and not report['exact_match']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
