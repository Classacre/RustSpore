#!/usr/bin/env python3
"""pe_strings.py - Extract and classify ASCII + UTF-16LE strings from SPORE exes.

Static analysis only, read-only. Writes re/data/strings-classified.txt.
"""
import re, os, sys, collections

TARGETS = [
    ("BASE", r"D:\Games\SPORE Collection\SporeBin\SporeApp.exe"),
    ("EP1",  r"D:\Games\SPORE Collection\SporebinEP1\SporeApp.exe"),
]

ASCII_RE = re.compile(rb"[\x20-\x7e]{5,}")
UTF16_RE = re.compile(rb"(?:[\x20-\x7e]\x00){4,}")


def extract(raw):
    ascii_s = [m.decode("latin1") for m in ASCII_RE.findall(raw)]
    utf16_s = []
    for m in UTF16_RE.findall(raw):
        try:
            utf16_s.append(m.decode("utf-16-le"))
        except Exception:
            pass
    return ascii_s, utf16_s


# Classification rules: (bucket, compiled regex applied to the string)
RULES = [
    ("MIDDLEWARE_HAVOK", re.compile(r"havok|hkp?[A-Z]\w|\bhkWorld|hkShape|hkRigid", re.I)),
    ("MIDDLEWARE_PUREDATA", re.compile(r"pdtk_|loadbang|\bcanvas\b|EAPd/|\.gfxstub|outlet|inlet|\bbang\b|hslider|vslider|\btgl\b|\bnbx\b|\bvradio\b", re.I)),
    ("MIDDLEWARE_STEAM", re.compile(r"steam_?api|SteamUser|SteamApps|SteamClient|SteamUtils", re.I)),
    ("MIDDLEWARE_GOG", re.compile(r"goggame|gog_?galaxy|GDF_GOG", re.I)),
    ("SCRIPT_ARGSCRIPT", re.compile(r"ArgScript|cArgScript|arg_script|ArgScripts?", re.I)),
    ("SCRIPT_LUA", re.compile(r"\blua\b|LuaState|lua_State", re.I)),
    ("SCRIPT_PYTHON", re.compile(r"\bpython\b|Py_|\.pyc", re.I)),
    ("RESOURCE_SIMULATOR", re.compile(r"Simulator/[A-Za-z]")),
    ("RESOURCE_TYPE", re.compile(r"^c[A-Z][A-Za-z0-9]{2,}(Resource|Factory|Container)$")),
    ("DBPF_PACKAGE", re.compile(r"\.package$|\.prop$|\.spore$|\.dbpf$|DBPF|dbpf", re.I)),
    ("FILEPATH", re.compile(r"[A-Za-z]:\\|\\\\|\.(dll|exe|txt|xml|ini|png|dds|ttf|dae|prop|package|spore|pdb|bmp|jpg|wav|mp3|ogg|hlsl|fx|bin|dat)\b", re.I)),
    ("SHADER_D3D", re.compile(r"\bvs_?\d|\bps_?\d|shader|HLSL|D3DX|technique|pass\b|semantic|POSITION\d?|TEXCOORD\d?|NORMAL\d?|COLOR\d?|diffuse|specular|vertexshader|pixelshader", re.I)),
    ("AUDIO", re.compile(r"plugin|Butterworth|VBAP|reverb|ReverbIR|GainFader|TimeStretch|Pan3D|\bDac\b|mixer|sample ?rate|44\.1|48 ?kHz|compressor|limiter|oscillator|fader", re.I)),
    ("PHYSICS", re.compile(r"physics|collision|rigid ?body|constraint|ragdoll|convex|broadphase|narrowphase", re.I)),
    ("NETWORK", re.compile(r"http://|https://|socket|winsock|WS2_32|connect\(|download|\.com/|\burl\b", re.I)),
    ("REGISTRY", re.compile(r"SOFTWARE\\|HKEY_|HKCU|HKLM|CurrentVersion|\\Spore\b|Electronic Arts", re.I)),
    ("ASSERT_DEBUG", re.compile(r"Assert|assert|\bFile:|line %|\.cpp\b|\.h\b|\bWarning\b|\bError\b|failed|invalid", re.I)),
    ("CONFIG_KEY", re.compile(r"ConfigManager|VideoCards|config|setting|option|graphics quality|resolution|fullscreen", re.I)),
    ("LOCALE", re.compile(r"locale|localized|language|\.lang|en-us|de-de|fr-fr|es-es|it-it|nl-nl|ru-ru|pl-pl|cs-cz|hu-hu|fi-fi|no-no|sv-se|da-dk|pt-pt", re.I)),
    ("MAGIC_HEX", re.compile(r"^0x[0-9A-Fa-f]{4,}")),
    ("UI_EDITOR", re.compile(r"editor|palette|toolbar|menu|button|slider|checkbox|dialog|skinpaint|DNA|creature|terraformin", re.I)),
]


def classify(strings):
    buckets = collections.defaultdict(set)
    for s in strings:
        placed = False
        for name, rx in RULES:
            if rx.search(s):
                buckets[name].add(s)
                placed = True
        if not placed:
            buckets["UNCLASSIFIED"].add(s)
    return buckets


def is_identifier_like(s):
    return re.match(r"^[A-Za-z_][A-Za-z0-9_:<>/.]{3,80}$", s) is not None


def main():
    out = []
    W = out.append
    for label, path in TARGETS:
        raw = open(path, "rb").read()
        asc, u16 = extract(raw)
        W("#" * 90)
        W(f"# STRING INVENTORY: {label}  ({path})")
        W(f"# ascii runs>=5: {len(asc)} (unique {len(set(asc))})   utf16 runs>=4: {len(u16)} (unique {len(set(u16))})")
        W("#" * 90)
        W("")
        all_s = sorted(set(asc) | set(u16))
        buckets = classify(all_s)
        order = ["RESOURCE_SIMULATOR", "RESOURCE_TYPE", "DBPF_PACKAGE", "SCRIPT_ARGSCRIPT",
                 "MIDDLEWARE_PUREDATA", "MIDDLEWARE_HAVOK", "MIDDLEWARE_STEAM", "MIDDLEWARE_GOG",
                 "SCRIPT_LUA", "SCRIPT_PYTHON", "SHADER_D3D", "AUDIO", "PHYSICS", "NETWORK",
                 "REGISTRY", "LOCALE", "CONFIG_KEY", "MAGIC_HEX", "FILEPATH", "UI_EDITOR",
                 "ASSERT_DEBUG", "UNCLASSIFIED"]
        for b in order:
            items = sorted(buckets.get(b, set()))
            W(f"===== [{b}]  {len(items)} unique =====")
            if b == "UNCLASSIFIED":
                # only show identifier-like unclassified to keep file useful
                ids = [s for s in items if is_identifier_like(s)]
                W(f"  (identifier-like subset shown: {len(ids)} of {len(items)})")
                for s in ids[:3000]:
                    W(f"  {s}")
            else:
                cap = 4000
                for s in items[:cap]:
                    W(f"  {s}")
                if len(items) > cap:
                    W(f"  ... ({len(items)-cap} more)")
            W("")
        # frequency section for ascii
        W("===== TOP 200 ASCII BY FREQUENCY =====")
        c = collections.Counter(asc)
        for s, n in c.most_common(200):
            W(f"  {n:6d}  {s}")
        W("")
    text = "\n".join(out)
    with open(r"D:\AI\RustSpore\re\data\strings-classified.txt", "w", encoding="utf-8") as f:
        f.write(text)
    print(f"wrote {len(text)} bytes; ascii(baseline)={len(asc)}")


if __name__ == "__main__":
    main()
