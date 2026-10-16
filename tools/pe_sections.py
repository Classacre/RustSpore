#!/usr/bin/env python3
"""pe_sections.py - PE structure dump + Rich header decode for SPORE executables.

Static analysis only. Never executes the target. Read-only.
Outputs a human readable section table and header facts for both binaries.
"""
import sys, math, struct, hashlib, datetime
import pefile

TARGETS = [
    ("BASE", r"D:\Games\SPORE Collection\SporeBin\SporeApp.exe"),
    ("EP1",  r"D:\Games\SPORE Collection\SporebinEP1\SporeApp.exe"),
]


def entropy(data: bytes) -> float:
    if not data:
        return 0.0
    counts = [0] * 256
    for b in data:
        counts[b] += 1
    n = len(data)
    ent = 0.0
    for c in counts:
        if c:
            p = c / n
            ent -= p * math.log2(p)
    return ent


def decode_rich(pe: pefile.PE, raw: bytes) -> dict:
    """Decode the Rich header: XOR key is the DanS dword following the header."""
    out = {"present": False, "key": None, "entries": [], "checksum_valid": None}
    try:
        e_lfanew = pe.DOS_HEADER.e_lfanew
    except Exception:
        return out
    # Rich header lives in the DOS stub, between 0x80 and e_lfanew
    region = raw[0x80:e_lfanew]
    idx = region.find(b"Rich")
    if idx < 0:
        return out
    out["present"] = True
    key = struct.unpack_from("<I", region, idx + 4)[0]
    out["key"] = key
    # DanS signature is "DanS" XOR key
    dans = bytes([ord("D") ^ (key & 0xFF), ord("a") ^ ((key >> 8) & 0xFF),
                  ord("n") ^ ((key >> 16) & 0xFF), ord("S") ^ ((key >> 24) & 0xFF)])
    start = region.rfind(dans, 0, idx)
    if start < 0:
        return out
    pos = start + 4
    # three padding dwords follow DanS, then pairs of (comp.id, count) until Rich
    pos += 12
    entries = []
    while pos + 8 <= idx:
        cid, cnt = struct.unpack_from("<II", region, pos)
        cid ^= key
        cnt ^= key
        pos += 8
        entries.append((cid, cnt))
    out["entries"] = entries
    out["rich_offset"] = 0x80 + idx
    # validate checksum: canonical algorithm (Pistelli / corkami):
    #   checksum = 0
    #   for each (id, count): checksum += (id & 0xFFFF) + (id>>16)*count
    #                         checksum = rol32(checksum, 8)
    #   checksum += offset_of_Rich_signature
    checksum = 0
    for cid, cnt in entries:
        checksum = (checksum + (cid & 0xFFFF) + (((cid >> 16) & 0xFFFF) * cnt)) & 0xFFFFFFFF
        checksum = ((checksum << 8) | (checksum >> 24)) & 0xFFFFFFFF
    checksum = (checksum + (0x80 + idx)) & 0xFFFFFFFF
    out["checksum_valid"] = (checksum == key)
    out["computed_key"] = checksum
    return out


# Known MSVC product ids (from public Rich header research)
PROD_IDS = {
    0x00: "Unknown/Import0",
    0x01: "Import0", 0x02: "Linker510", 0x03: "Cvtomf510", 0x04: "Linker600",
    0x05: "Cvtomf600", 0x06: "Cvtres500", 0x07: "Utc11_Basic", 0x08: "Utc11_C",
    0x09: "Utc12_Basic", 0x0A: "Utc12_C", 0x0B: "Utc12_CPP", 0x0C: "AliasObj60",
    0x0D: "VisualBasic60", 0x0E: "Masm613", 0x0F: "Masm710",
    0x10: "Linker511", 0x11: "Cvtomf511", 0x12: "Masm614", 0x13: "Linker512",
    0x14: "Cvtomf512", 0x15: "Utc12_C_Std", 0x16: "Utc12_CPP_Std",
    0x17: "Utc12_C_Book", 0x18: "Utc12_CPP_Book", 0x19: "Implib700",
    0x1A: "Cvtomf700", 0x1B: "Utc13_Basic", 0x1C: "Utc13_C", 0x1D: "Utc13_CPP",
    0x1E: "Linker610", 0x1F: "Cvtomf610", 0x20: "Linker601", 0x21: "Cvtomf601",
    0x22: "Utc12_1_Basic", 0x23: "Utc12_1_C", 0x24: "Utc12_1_CPP",
    0x25: "Linker620", 0x26: "Cvtomf620", 0x27: "AliasObj70",
    0x28: "Linker621", 0x29: "Cvtomf621", 0x2A: "Masm615",
    0x2B: "Utc13_LTCG_C", 0x2C: "Utc13_LTCG_CPP", 0x2D: "Masm620",
    0x2E: "ILAsm100", 0x2F: "Utc12_2_Basic", 0x30: "Utc12_2_C",
    0x31: "Utc12_2_CPP", 0x32: "Utc12_2_C_Std", 0x33: "Utc12_2_CPP_Std",
    0x34: "Utc12_2_C_Book", 0x35: "Utc12_2_CPP_Book", 0x36: "Implib622",
    0x37: "Cvtomf622", 0x38: "Cvtres501", 0x39: "Utc13_C_Std",
    0x3A: "Utc13_CPP_Std", 0x3B: "Cvtpgd1300", 0x3C: "Linker622",
    0x3D: "Linker700", 0x3E: "Export622", 0x3F: "Export700", 0x40: "Masm700",
    0x41: "Utc13_POGO_I_C", 0x42: "Utc13_POGO_I_CPP", 0x43: "Utc13_POGO_O_C",
    0x44: "Utc13_POGO_O_CPP", 0x45: "Cvtres700", 0x46: "Cvtres710p",
    0x47: "Linker710p", 0x48: "Cvtomf710p", 0x49: "Export710p",
    0x4A: "Implib710p", 0x4B: "Masm710p", 0x4C: "Utc1310p_C",
    0x4D: "Utc1310p_CPP", 0x4E: "Utc1310p_C_Std", 0x4F: "Utc1310p_CPP_Std",
    0x50: "Utc1310p_LTCG_C", 0x51: "Utc1310p_LTCG_CPP", 0x52: "Utc1310p_POGO_I_C",
    0x53: "Utc1310p_POGO_I_CPP", 0x54: "Utc1310p_POGO_O_C", 0x55: "Utc1310p_POGO_O_CPP",
    0x56: "Linker624", 0x57: "Cvtomf624", 0x58: "Export624", 0x59: "Implib624",
    0x5A: "Linker710", 0x5B: "Cvtomf710", 0x5C: "Export710", 0x5D: "Implib710",
    0x5E: "Cvtres710", 0x5F: "Utc1310_C", 0x60: "Utc1310_CPP", 0x61: "Utc1310_C_Std",
    0x62: "Utc1310_CPP_Std", 0x63: "Utc1310_LTCG_C", 0x64: "Utc1310_LTCG_CPP",
    0x65: "Utc1310_POGO_I_C", 0x66: "Utc1310_POGO_I_CPP", 0x67: "Utc1310_POGO_O_C",
    0x68: "Utc1310_POGO_O_CPP", 0x69: "AliasObj710", 0x6A: "AliasObj710p",
    0x6B: "Cvtpgd1310", 0x6C: "Cvtpgd1310p", 0x6D: "Utc1400_C", 0x6E: "Utc1400_CPP",
    0x6F: "Utc1400_C_Std", 0x70: "Utc1400_CPP_Std", 0x71: "Utc1400_LTCG_C",
    0x72: "Utc1400_LTCG_CPP", 0x73: "Utc1400_POGO_I_C", 0x74: "Utc1400_POGO_I_CPP",
    0x75: "Utc1400_POGO_O_C", 0x76: "Utc1400_POGO_O_CPP", 0x77: "Cvtpgd1400",
    0x78: "Linker800", 0x79: "Cvtomf800", 0x7A: "Export800", 0x7B: "Implib800",
    0x7C: "Cvtres800", 0x7D: "Masm800", 0x7E: "AliasObj800",
    0x7F: "PhoenixPrerelease", 0x80: "Utc1400_CVTCIL_C", 0x81: "Utc1400_CVTCIL_CPP",
    0x82: "Utc1400_LTCG_MSIL", 0x83: "Utc1500_C", 0x84: "Utc1500_CPP",
    0x85: "Utc1500_C_Std", 0x86: "Utc1500_CPP_Std", 0x87: "Utc1500_CVTCIL_C",
    0x88: "Utc1500_CVTCIL_CPP", 0x89: "Utc1500_LTCG_C", 0x8A: "Utc1500_LTCG_CPP",
    0x8B: "Utc1500_LTCG_MSIL", 0x8C: "Utc1500_POGO_I_C", 0x8D: "Utc1500_POGO_I_CPP",
    0x8E: "Utc1500_POGO_O_C", 0x8F: "Utc1500_POGO_O_CPP", 0x90: "Cvtpgd1500",
    0x91: "Linker900", 0x92: "Export900", 0x93: "Implib900", 0x94: "Cvtres900",
    0x95: "Masm900", 0x96: "AliasObj900", 0x97: "Resource900",
    0x98: "AliasObj1000", 0x99: "Cvtpgd1600", 0x9A: "Cvtres1000",
    0x9B: "Export1000", 0x9C: "Implib1000", 0x9D: "Linker1000", 0x9E: "Masm1000",
    0x9F: "Utc1600_C", 0xA0: "Utc1600_CPP", 0xA1: "Utc1600_CVTCIL_C",
    0xA2: "Utc1600_CVTCIL_CPP", 0xA3: "Utc1600_LTCG_C", 0xA4: "Utc1600_LTCG_CPP",
    0xA5: "Utc1600_LTCG_MSIL", 0xA6: "Utc1600_POGO_I_C", 0xA7: "Utc1600_POGO_I_CPP",
    0xA8: "Utc1600_POGO_O_C", 0xA9: "Utc1600_POGO_O_CPP", 0xAA: "Linker1010",
    0xAB: "Export1010", 0xAC: "Implib1010", 0xAD: "Cvtres1010", 0xAE: "Masm1010",
    0xAF: "AliasObj1010", 0xB0: "Utc1700_C", 0xB1: "Utc1700_CPP",
    0xB2: "Utc1700_CVTCIL_C", 0xB3: "Utc1700_CVTCIL_CPP", 0xB4: "Utc1700_LTCG_C",
    0xB5: "Utc1700_LTCG_CPP", 0xB6: "Utc1700_LTCG_MSIL", 0xB7: "Utc1700_POGO_I_C",
    0xB8: "Utc1700_POGO_I_CPP", 0xB9: "Utc1700_POGO_O_C", 0xBA: "Utc1700_POGO_O_CPP",
    0xBB: "Cvtpgd1700", 0xBC: "Linker1100", 0xBD: "Export1100", 0xBE: "Implib1100",
    0xBF: "Cvtres1100", 0xC0: "Masm1100", 0xC1: "AliasObj1100", 0xC2: "Resource1100",
    0xC3: "Utc1800_C", 0xC4: "Utc1800_CPP", 0xC5: "Utc1800_CVTCIL_C",
    0xC6: "Utc1800_CVTCIL_CPP", 0xC7: "Utc1800_LTCG_C", 0xC8: "Utc1800_LTCG_CPP",
    0xC9: "Utc1800_LTCG_MSIL", 0xCA: "Utc1800_POGO_I_C", 0xCB: "Utc1800_POGO_I_CPP",
    0xCC: "Utc1800_POGO_O_C", 0xCD: "Utc1800_POGO_O_CPP", 0xCE: "Cvtpgd1800",
    0xCF: "Linker1200", 0xD0: "Export1200", 0xD1: "Implib1200", 0xD2: "Cvtres1200",
    0xD3: "Masm1200", 0xD4: "AliasObj1200", 0xD5: "Resource1200",
}


def dump(label, path, out):
    raw = open(path, "rb").read()
    pe = pefile.PE(path, fast_load=False)
    W = out.append
    W("=" * 78)
    W(f"BINARY: {label}")
    W(f"path: {path}")
    W(f"size: {len(raw)} bytes")
    W(f"sha256: {hashlib.sha256(raw).hexdigest()}")
    W(f"md5: {hashlib.md5(raw).hexdigest()}")
    W("")
    fh = pe.FILE_HEADER
    oh = pe.OPTIONAL_HEADER
    W("--- COFF / OPTIONAL HEADER ---")
    W(f"Machine: 0x{fh.Machine:04x}")
    W(f"NumberOfSections: {fh.NumberOfSections}")
    ts = fh.TimeDateStamp
    try:
        tstr = datetime.datetime.utcfromtimestamp(ts).strftime("%Y-%m-%d %H:%M:%S UTC")
    except Exception:
        tstr = "out of range"
    W(f"TimeDateStamp: 0x{ts:08x} ({tstr})")
    W(f"PointerToSymbolTable: 0x{fh.PointerToSymbolTable:08x}")
    W(f"NumberOfSymbols: {fh.NumberOfSymbols}")
    W(f"SizeOfOptionalHeader: 0x{fh.SizeOfOptionalHeader:x}")
    W(f"Characteristics: 0x{fh.Characteristics:04x}")
    W(f"Magic: 0x{oh.Magic:04x}  (0x10b=PE32, 0x20b=PE32+)")
    W(f"MajorLinkerVersion: {oh.MajorLinkerVersion}  MinorLinkerVersion: {oh.MinorLinkerVersion}")
    W(f"AddressOfEntryPoint: 0x{oh.AddressOfEntryPoint:08x}  (VA 0x{oh.ImageBase + oh.AddressOfEntryPoint:08x})")
    W(f"ImageBase: 0x{oh.ImageBase:08x}")
    W(f"SectionAlignment: 0x{oh.SectionAlignment:x}  FileAlignment: 0x{oh.FileAlignment:x}")
    W(f"OS Version: {oh.MajorOperatingSystemVersion}.{oh.MinorOperatingSystemVersion}")
    W(f"Image Version: {oh.MajorImageVersion}.{oh.MinorImageVersion}")
    W(f"Subsystem Version: {oh.MajorSubsystemVersion}.{oh.MinorSubsystemVersion}")
    W(f"SizeOfImage: 0x{oh.SizeOfImage:x}  SizeOfHeaders: 0x{oh.SizeOfHeaders:x}")
    W(f"CheckSum: 0x{oh.CheckSum:08x}")
    W(f"Subsystem: {oh.Subsystem}  (2=GUI, 3=CUI)")
    W(f"DllCharacteristics: 0x{oh.DllCharacteristics:04x}")
    W(f"SizeOfStackReserve: 0x{oh.SizeOfStackReserve:x}  Commit: 0x{oh.SizeOfStackCommit:x}")
    W(f"SizeOfHeapReserve: 0x{oh.SizeOfHeapReserve:x}  Commit: 0x{oh.SizeOfHeapCommit:x}")
    W(f"NumberOfRvaAndSizes: {oh.NumberOfRvaAndSizes}")
    W("")
    W("--- SECTIONS ---")
    W(f"{'name':<10}{'VA':>12}{'VSize':>12}{'RawPtr':>12}{'RawSize':>12}{'Chars':>12}{'Entropy':>9}")
    for s in pe.sections:
        nm = s.Name.rstrip(b"\x00").decode("latin1")
        data = raw[s.PointerToRawData:s.PointerToRawData + s.SizeOfRawData]
        ent = entropy(data)
        W(f"{nm:<10}0x{s.VirtualAddress:08x}  0x{s.Misc_VirtualSize:08x}  0x{s.PointerToRawData:08x}  0x{s.SizeOfRawData:08x}  0x{s.Characteristics:08x}  {ent:8.4f}")
    W("")
    W("--- DATA DIRECTORIES ---")
    dd_names = ["EXPORT", "IMPORT", "RESOURCE", "EXCEPTION", "SECURITY", "BASERELOC",
                "DEBUG", "ARCHITECTURE", "GLOBALPTR", "TLS", "LOAD_CONFIG", "BOUND_IMPORT",
                "IAT", "DELAY_IMPORT", "COM_DESCRIPTOR", "RESERVED"]
    for i, d in enumerate(oh.DATA_DIRECTORY):
        name = dd_names[i] if i < len(dd_names) else f"DIR{i}"
        W(f"{name:<16} RVA 0x{d.VirtualAddress:08x}  Size 0x{d.Size:x}")
    W("")
    # TLS
    W("--- TLS ---")
    try:
        if hasattr(pe, "DIRECTORY_ENTRY_TLS") and pe.DIRECTORY_ENTRY_TLS:
            tls = pe.DIRECTORY_ENTRY_TLS.struct
            W(f"StartAddressOfRawData: 0x{tls.StartAddressOfRawData:08x}")
            W(f"EndAddressOfRawData: 0x{tls.EndAddressOfRawData:08x}")
            W(f"AddressOfIndex: 0x{tls.AddressOfIndex:08x}")
            W(f"AddressOfCallBacks: 0x{tls.AddressOfCallBacks:08x}")
            cbs = pe.DIRECTORY_ENTRY_TLS.struct.AddressOfCallBacks
            if cbs:
                p = cbs - oh.ImageBase
                cbsize = (tls.EndAddressOfRawData - tls.StartAddressOfRawData)
                # read callback array
                try:
                    off = pe.get_offset_from_rva(p)
                    callbacks = []
                    for k in range(16):
                        val = struct.unpack_from("<I", raw, off + k * 4)[0]
                        if val == 0:
                            break
                        callbacks.append(val)
                    W(f"Callbacks: {[hex(c) for c in callbacks]}")
                except Exception as e:
                    W(f"callback read error: {e}")
        else:
            W("no TLS directory")
    except Exception as e:
        W(f"TLS error: {e}")
    W("")
    # Load config
    W("--- LOAD CONFIG ---")
    lc_dir = oh.DATA_DIRECTORY[10]
    if lc_dir.VirtualAddress == 0 and lc_dir.Size == 0:
        W("LOAD_CONFIG data directory is empty (RVA 0, size 0): no load config")
    else:
        try:
            lc = pe.DIRECTORY_ENTRY_LOAD_CONFIG.struct
            for field in ("Size", "TimeDateStamp", "MajorVersion", "MinorVersion",
                          "GlobalFlagsClear", "GlobalFlagsSet", "CriticalSectionDefaultTimeout",
                          "SecurityCookie", "SEHandlerTable", "SEHandlerCount"):
                if hasattr(lc, field):
                    v = getattr(lc, field)
                    W(f"{field}: {hex(v) if isinstance(v, int) else v}")
        except Exception as e:
            W(f"load config not parsed: {e}")
    W("")
    # Rich
    W("--- RICH HEADER ---")
    rich = decode_rich(pe, raw)
    if not rich["present"]:
        W("no Rich header")
    else:
        W(f"XOR key: 0x{rich['key']:08x}")
        W(f"checksum valid: {rich['checksum_valid']} (computed 0x{rich.get('computed_key',0):08x})")
        W(f"entries: {len(rich['entries'])}")
        for cid, cnt in rich["entries"]:
            prod = (cid >> 16) & 0xFFFF
            build = cid & 0xFFFF
            pname = PROD_IDS.get(prod, f"Prod0x{prod:x}")
            W(f"  prod={prod} (0x{prod:x}) build={build} count={cnt}  # {pname}")
    W("")
    W("--- IMPORTS (summary) ---")
    try:
        for entry in pe.DIRECTORY_ENTRY_IMPORT:
            W(f"{entry.dll.decode('latin1')}: {len(entry.imports)} imports")
    except Exception as e:
        W(f"import error: {e}")
    W("")
    W("--- EXPORTS ---")
    try:
        if hasattr(pe, "DIRECTORY_ENTRY_EXPORT"):
            exp = pe.DIRECTORY_ENTRY_EXPORT
            W(f"dll name: {exp.name}")
            W(f"symbols: {len(exp.symbols)}")
            for s in exp.symbols[:400]:
                nm = s.name.decode("latin1") if s.name else None
                W(f"  ordinal={s.ordinal} addr=0x{s.address:08x} name={nm}")
        else:
            W("no export directory parsed")
    except Exception as e:
        W(f"export error: {e}")
    W("")
    W("--- RESOURCES ---")
    try:
        if hasattr(pe, "DIRECTORY_ENTRY_RESOURCE"):
            for rt in pe.DIRECTORY_ENTRY_RESOURCE.entries:
                W(f"type id={rt.id} name={rt.name} entries={len(rt.directory.entries)}")
        else:
            W("no resource directory")
    except Exception as e:
        W(f"resource error: {e}")
    W("")
    return raw


def main():
    out = []
    for label, path in TARGETS:
        try:
            dump(label, path, out)
        except Exception as e:
            out.append(f"ERROR processing {label}: {e!r}")
    text = "\n".join(out)
    print(text)
    with open(r"D:\AI\RustSpore\re\data\pe-structure-raw.txt", "w", encoding="utf-8") as f:
        f.write(text)


if __name__ == "__main__":
    main()
