#!/usr/bin/env python3
"""pe_meta.py - Version resource, resources, debug dir, raw data directories,
export presence, and helper-DLL fingerprints for SPORE executables.

Static analysis only, read-only.
"""
import struct, hashlib
import pefile

TARGETS = [
    ("BASE", r"D:\Games\SPORE Collection\SporeBin\SporeApp.exe"),
    ("EP1",  r"D:\Games\SPORE Collection\SporebinEP1\SporeApp.exe"),
    ("steam_api_BASE", r"D:\Games\SPORE Collection\SporeBin\steam_api.dll"),
    ("goggame_EP1", r"D:\Games\SPORE Collection\SporebinEP1\goggame-1948823323.dll"),
]

RT_NAMES = {1: "CURSOR", 2: "BITMAP", 3: "ICON", 4: "MENU", 5: "DIALOG", 6: "STRING",
            7: "FONTDIR", 8: "FONT", 9: "ACCELERATOR", 10: "RCDATA", 11: "MESSAGETABLE",
            12: "GROUP_CURSOR", 14: "GROUP_ICON", 16: "VERSION", 17: "DLGINCLUDE",
            19: "PLUGPLAY", 20: "VXD", 21: "ANICURSOR", 22: "ANIICON", 23: "HTML",
            24: "MANIFEST"}


def raw_data_dirs(raw):
    e = struct.unpack_from("<I", raw, 0x3C)[0]
    # optional header starts at e+24; PE32 optional header data dirs at e+24+96
    opt = e + 24
    magic = struct.unpack_from("<H", raw, opt)[0]
    ddoff = opt + (96 if magic == 0x10b else 112)
    out = []
    for i in range(16):
        rva, size = struct.unpack_from("<II", raw, ddoff + i * 8)
        out.append((i, rva, size))
    return out


def parse_version(pe, raw):
    """Extract VS_VERSIONINFO strings (hex-encoded UTF-16)."""
    result = {}
    if not hasattr(pe, "DIRECTORY_ENTRY_RESOURCE"):
        return result
    for rt in pe.DIRECTORY_ENTRY_RESOURCE.entries:
        if rt.id != 16:  # RT_VERSION
            continue
        for name_e in rt.directory.entries:
            for lang_e in name_e.directory.entries:
                d = lang_e.data.struct
                off = pe.get_offset_from_rva(d.OffsetToData)
                blob = raw[off:off + d.Size]
                # crude: extract UTF-16 strings
                strs = []
                i = 0
                cur = []
                while i + 1 < len(blob):
                    ch = blob[i] | (blob[i + 1] << 8)
                    if 32 <= ch < 0x7F:
                        cur.append(chr(ch))
                        i += 2
                    else:
                        if len(cur) >= 2:
                            strs.append("".join(cur))
                        cur = []
                        i += 2
                if len(cur) >= 2:
                    strs.append("".join(cur))
                result.setdefault("strings", []).extend(strs)
                result["raw_len"] = len(blob)
    return result


def parse_fixed_version(pe, raw):
    """Parse VS_FIXEDFILEINFO at start of RT_VERSION blob."""
    for rt in pe.DIRECTORY_ENTRY_RESOURCE.entries:
        if rt.id != 16:
            continue
        for name_e in rt.directory.entries:
            for lang_e in name_e.directory.entries:
                d = lang_e.data.struct
                off = pe.get_offset_from_rva(d.OffsetToData)
                blob = raw[off:off + d.Size]
                # find signature 0xFEEF04BD
                sig = blob.find(b"\xbd\x04\xef\xfe")
                if sig >= 0:
                    vals = struct.unpack_from("<13I", blob, sig)
                    (sigv, strucver, fvms, fvls, pvms, pvls,
                     fflagsmask, fflags, fos, ftype, fsubtype, fdate1, fdate2) = vals
                    return {
                        "filevers": f"{fvms >> 16}.{fvms & 0xffff}.{fvls >> 16}.{fvls & 0xffff}",
                        "prodvers": f"{pvms >> 16}.{pvms & 0xffff}.{pvls >> 16}.{pvls & 0xffff}",
                        "fileflags": hex(fflags), "fileos": hex(fos),
                        "filetype": hex(ftype), "filesubtype": hex(fsubtype),
                    }
    return {}


def parse_manifest(pe, raw):
    for rt in pe.DIRECTORY_ENTRY_RESOURCE.entries:
        if rt.id != 24:
            continue
        for name_e in rt.directory.entries:
            for lang_e in name_e.directory.entries:
                d = lang_e.data.struct
                off = pe.get_offset_from_rva(d.OffsetToData)
                return raw[off:off + d.Size].decode("utf-8", "replace")
    return None


def parse_debug(pe, raw):
    out = []
    try:
        for dbg in pe.DIRECTORY_ENTRY_DEBUG:
            s = dbg.struct
            entry = {"type": s.Type, "size": s.SizeOfData,
                     "addr_raw": hex(s.AddressOfRawData), "ptr_raw": hex(s.PointerToRawData),
                     "timestamp": hex(s.TimeDateStamp)}
            if s.Type == 2:  # CODEVIEW
                off = s.PointerToRawData
                data = raw[off:off + s.SizeOfData]
                if data[:4] == b"RSDS":
                    guid = data[4:20].hex()
                    age = struct.unpack_from("<I", data, 20)[0]
                    path = data[24:].split(b"\x00")[0].decode("latin1")
                    entry["pdb"] = path
                    entry["pdb_guid"] = guid
                    entry["pdb_age"] = age
                elif data[:4] == b"NB10":
                    entry["pdb"] = data[16:].split(b"\x00")[0].decode("latin1")
            out.append(entry)
    except Exception as e:
        out.append({"error": repr(e)})
    return out


def list_resources(pe):
    out = []
    if not hasattr(pe, "DIRECTORY_ENTRY_RESOURCE"):
        return out
    for rt in pe.DIRECTORY_ENTRY_RESOURCE.entries:
        tid = rt.id
        tname = rt.name if rt.name else RT_NAMES.get(tid, f"TYPE{tid}")
        names = []
        for ne in rt.directory.entries:
            nm = ne.name if ne.name else str(ne.id)
            if ne.directory:
                langs = [str(le.id) if not le.name else str(le.name) for le in ne.directory.entries]
                names.append(f"{nm}[{','.join(langs)}]")
            else:
                names.append(str(nm))
        out.append((tid, tname, names))
    return out


def do(label, path, out):
    raw = open(path, "rb").read()
    pe = pefile.PE(path, fast_load=False)
    W = out.append
    W("=" * 78)
    W(f"BINARY: {label}  ({path})")
    W(f"size={len(raw)} sha256={hashlib.sha256(raw).hexdigest()}")
    W("")
    W("--- RAW DATA DIRECTORIES (16) ---")
    for i, rva, size in raw_data_dirs(raw):
        W(f"  [{i:2d}] RVA=0x{rva:08x} Size=0x{size:x}")
    W("")
    W("--- DEBUG DIRECTORY ---")
    try:
        for d in parse_debug(pe, raw):
            W(f"  {d}")
    except Exception as ex:
        W(f"  error: {ex!r}")
    W("")
    W("--- FIXED FILE INFO ---")
    try:
        W(f"  {parse_fixed_version(pe, raw)}")
    except Exception as ex:
        W(f"  error: {ex!r}")
    W("")
    W("--- VERSION STRINGS ---")
    try:
        v = parse_version(pe, raw)
        for s in v.get("strings", []):
            W(f"  {s!r}")
    except Exception as ex:
        W(f"  error: {ex!r}")
    W("")
    W("--- MANIFEST ---")
    try:
        m = parse_manifest(pe, raw)
    except Exception as ex:
        m = None
        W(f"  error: {ex!r}")
    if m:
        W(m[:4000])
    else:
        W("  (none)")
    W("")
    W("--- RESOURCE TREE ---")
    try:
        for tid, tname, names in list_resources(pe):
            W(f"  type {tid} ({tname}): {len(names)} entries")
            for n in names[:40]:
                W(f"      {n}")
    except Exception as ex:
        W(f"  error: {ex!r}")
    W("")
    # exports
    W("--- EXPORTS ---")
    if hasattr(pe, "DIRECTORY_ENTRY_EXPORT"):
        e = pe.DIRECTORY_ENTRY_EXPORT
        W(f"  dll name: {e.name}")
        W(f"  symbols: {len(e.symbols)}")
        for s in e.symbols[:200]:
            W(f"    ord={s.ordinal} rva=0x{s.address:08x} name={s.name}")
    else:
        W("  no export directory")
    W("")


def main():
    out = []
    for label, path in TARGETS:
        try:
            do(label, path, out)
        except Exception as ex:
            out.append(f"ERROR {label}: {ex!r}")
    text = "\n".join(out)
    print(text)
    with open(r"D:\AI\RustSpore\re\data\pe-meta-raw.txt", "w", encoding="utf-8") as f:
        f.write(text)


if __name__ == "__main__":
    main()
