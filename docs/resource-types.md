# SPORE resource type IDs

Research date: 2026-10-16. Compiled by: general-purpose-2 (team `_auto_09c1c1dd`).

This file lists resource type IDs that appear inside SPORE (2008) DBPF packages. Each row gives the
numeric ID, the name, a short meaning or note, the source URL, and a confidence rating.

## Sources and confidence key

- SIMSWIKI: https://simswiki.info/wiki.php?title=Spore:Chunk_Types
  (the "Spore:Chunk Types" page, copied from the same CustomSims3 thread as the Spore DBPF page)
- REGTYPE: https://raw.githubusercontent.com/Spore-Community/SporeModder-FX/master/reg_type.txt
  (the name-to-hex dictionary shipped with SporeModder FX)

Confidence:

- high: SIMSWIKI and REGTYPE agree on the ID and the name.
- medium: only one source gives this ID, or the two sources agree on the ID but use slightly
  different names.
- low: only one source gives it, and the source itself flags uncertainty, or the name is clearly
  informal.

Where the two sources conflict on the NAME for the same ID, both names are shown and the row is
marked "conflict".

## 1. File and resource types

The IDs below are the resource types that can appear as entries in a package index. Values are
8-hex-digit, zero-padded.

| ID | Name | Meaning / note | Source | Confidence |
|---|---|---|---|---|
| 0x00000000 | properties.txt (context) | The property lookup list is referenced as resource `0x00000000-0x658B0AB2-0x024A0E52` in `main.package` (a comment in the source), not a type ID per se | SIMSWIKI | low |
| 0x002AE8E9 | spike | part name in the SporeModder list; likely a creature part asset type | REGTYPE | low |
| 0x00339479 | animstate | animation state | REGTYPE | low |
| 0x00B1B104 | prop | property file (see prior-art.md section 4.1) | SIMSWIKI + REGTYPE | high |
| 0xC7A91467 | layout | layout (second layout ID) | REGTYPE | low |
| 0x00E6BCE5 | gmdl | model (geometry model) | SIMSWIKI + REGTYPE | high |
| 0x011989B7 | plt | palette / PLT | SIMSWIKI + REGTYPE | high |
| 0x01A527DB | snr | unknown, name only | REGTYPE | low |
| 0x01AD2416 | creature_traits | creature trait data | SIMSWIKI + REGTYPE | high |
| 0x01AD2417 | building_traits | building trait data | SIMSWIKI + REGTYPE | high |
| 0x01AD2418 | vehicle_traits | vehicle trait data | SIMSWIKI + REGTYPE | high |
| 0x01C135DA | gmsh | model mesh (mesh) | SIMSWIKI + REGTYPE | high |
| 0x01C3C4B3 | trait_pill | trait pill data | SIMSWIKI + REGTYPE | high |
| 0x01E99B626 | bat | batch (SIMSWIKI "BATCH", added 2008-07-27) | SIMSWIKI + REGTYPE | high |
| 0x01EEF63A | sns | unknown, name only | REGTYPE | low |
| 0x022D2C83 | EAPD (SIMSWIKI) / pdr (REGTYPE) | conflict: SIMSWIKI calls it "EAPD" (added 2008-07-27), REGTYPE calls it "pdr" | SIMSWIKI + REGTYPE | conflict |
| 0x02393756 | Cursor (SIMSWIKI) / cur (REGTYPE) | cursor resource | SIMSWIKI + REGTYPE | high |
| 0x0248F226 | css | CSS (style sheet) | SIMSWIKI + REGTYPE | high |
| 0x024A0E52 | Cheat (SIMSWIKI) / trigger (REGTYPE) | conflict: SIMSWIKI "Cheat", REGTYPE "trigger" | SIMSWIKI + REGTYPE | conflict |
| 0x02523258 | formation (REGTYPE) / arg (REGTYPE, second) | formation data | REGTYPE | low |
| 0x027C5CEF | TrueType font (SIMSWIKI) / ttf (REGTYPE) | TrueType font (added 2008-07-27) | SIMSWIKI + REGTYPE | high |
| 0x029E333B | textual (SIMSWIKI) / voice (REGTYPE) | conflict: SIMSWIKI "textual" (added 2008-07-27), REGTYPE "voice" | SIMSWIKI + REGTYPE | conflict |
| 0x02B9F662 | soundProp / audioProp | sound property | REGTYPE | medium |
| 0x02C23DF0 | activeLog | active log | REGTYPE | low |
| 0x02C9EFF2 | submix | audio submix | REGTYPE | low |
| 0x02CB4F2F | pcaw | unknown, name only | REGTYPE | low |
| 0x02D5C9AF | SUMMARY | summary data | SIMSWIKI + REGTYPE | high |
| 0x02D5C9B0 | SUMMARY PILL | summary pill data | SIMSWIKI + REGTYPE | high |
| 0x02FAC0B6 | textual (SIMSWIKI) / locale (REGTYPE) | conflict: SIMSWIKI "textual", REGTYPE "locale" (the locale text resource) | SIMSWIKI + REGTYPE | conflict |
| 0x03055F61 | textual (SIMSWIKI) / instrument (REGTYPE) | conflict: SIMSWIKI "textual" (added 2008-07-27), REGTYPE "instrument" | SIMSWIKI + REGTYPE | conflict |
| 0x030BDEE3 | pollen_metadata | Pollinator (creation sharing) metadata | SIMSWIKI + REGTYPE | high |
| 0x0330A65E | eapdPixie | EAPD pixie (see 0x022D2C83) | REGTYPE | low |
| 0x0376C3DA | HM (SIMSWIKI) / hm (REGTYPE) | unknown, name only | SIMSWIKI + REGTYPE | medium |
| 0x03E421E9 | bitImage | 1-bit image | REGTYPE | low |
| 0x03E421EC | 8bitImage | 8-bit image | REGTYPE | low |
| 0x03E421ED | 32bitImage | 32-bit image | REGTYPE | low |
| 0x03E421EF | 48bitImage | 48-bit image | REGTYPE | low |
| 0x03E421F0 | 16bitImage | 16-bit image | REGTYPE | low |
| 0x03F51892 | children | children list | REGTYPE | low |
| 0x041E19BE | avi | AVI video | REGTYPE | medium |
| 0x042C9CBB | snd | sound | REGTYPE | medium |
| 0x0469A3F7 | smt | unknown, name only | REGTYPE | low |
| 0x0472329B | HTRA (SIMSWIKI) / htra and arth (REGTYPE) | REGTYPE itself notes "htra is just a wrong guess of a LE file header"; also mapped to "arth" | SIMSWIKI + REGTYPE | low |
| 0x047B8300 | lvl / scn | level / scene (renamed lvl to scn in a 2025 commit) | REGTYPE | low |
| 0x04805684 | advect | advection data | REGTYPE | low |
| 0x0497925E | mode | mode data | REGTYPE | low |
| 0x04AEB6BC6 | tlsa | animation list (SIMSWIKI "ANIM LIST") | SIMSWIKI + REGTYPE | high |
| 0x050F4731 | layout_bin | binary layout | REGTYPE | low |
| 0x055ADA24 | cnv | conversation data | REGTYPE | low |
| 0x065266B7 | xhtml | XHTML | REGTYPE | low |
| 0x06EFC6AA | package | nested package | REGTYPE | medium |
| 0x0A4D8D09 | bnk | audio bank | REGTYPE | low |
| 0x0A98EAF0 | json | JSON | REGTYPE | low |
| 0x08068AEC | er2 | unknown, name only | REGTYPE | low |
| 0x0D8D09... (see 0x0A4D8D09) | | | | |
| 0x0DAE69D1 | chemical | creature part category: chemical | REGTYPE | low |
| 0x0F43029A | creaturedata | creature data | REGTYPE | low |
| 0x12952634 | textual (SIMSWIKI) / dat (REGTYPE) | conflict: SIMSWIKI "textual", REGTYPE "dat" | SIMSWIKI + REGTYPE | conflict |
| 0x14885D1C | ico | icon image | REGTYPE | medium |
| 0x17952E6C | dds | DDS texture | REGTYPE | medium |
| 0x1A99B06B | BEM | BEM | SIMSWIKI + REGTYPE | high |
| 0x1E99B626 | BATCH (SIMSWIKI) / bat (REGTYPE) | batch (added 2008-07-27) | SIMSWIKI + REGTYPE | high |
| 0x1E99B639 | bak | backup | REGTYPE | low |
| 0x1E639C34 | xml | XML (second XML ID) | REGTYPE | medium |
| 0x1F639D98 | xls | spreadsheet | REGTYPE | low |
| 0x1F886EAD | ini | INI | REGTYPE | low |
| 0x2081B4F5 | log | log | REGTYPE | low |
| 0x2399BE55 | BLD (SIMSWIKI) / bld (REGTYPE) | building | SIMSWIKI + REGTYPE | high |
| 0x24682294 | VCL (SIMSWIKI) / vcl (REGTYPE) | vehicle | SIMSWIKI + REGTYPE | high |
| 0x250FE9A2 | spui | Spore UI | REGTYPE | low |
| 0x25DF0112 | Script (SIMSWIKI) / gait (REGTYPE) | conflict: SIMSWIKI "Script", REGTYPE "gait" | SIMSWIKI + REGTYPE | conflict |
| 0x2699C284 | bin | binary | REGTYPE | low |
| 0x276CA4B9 | ttf | TrueType font (second TTF ID) | REGTYPE | medium |
| 0x278CF8F2 | cfx / gfx | effect / graphics | REGTYPE | medium |
| 0x2B6CAB5F | NOTE (SIMSWIKI) / txt (REGTYPE) | conflict: SIMSWIKI "NOTE", REGTYPE "txt" | SIMSWIKI + REGTYPE | conflict |
| 0x2B978C46 | CRT (SIMSWIKI) / crt (REGTYPE) | creature | SIMSWIKI + REGTYPE | high |
| 0x2F4E681B | RW4 (SIMSWIKI) / rw4 (REGTYPE) | RenderWare 4 asset | SIMSWIKI + REGTYPE | high |
| 0x2F4E681C | RASTER (SIMSWIKI) / raster (REGTYPE) | raster image | SIMSWIKI + REGTYPE | high |
| 0x2F7D0002 | JPEG | JPEG image | SIMSWIKI + REGTYPE | high |
| 0x2F7D0004 | PNG | PNG image | SIMSWIKI + REGTYPE | high |
| 0x2F7D0005 | BMP | BMP image (added 2008-07-27) | SIMSWIKI + REGTYPE | high |
| 0x2F7D0006 | TGA | TGA image (added 2008-07-27) | SIMSWIKI + REGTYPE | high |
| 0x2F7D0007 | GIF | GIF image (added 2008-07-27) | SIMSWIKI + REGTYPE | high |
| 0x3055F61 (see 0x03055F61) | | | | |
| 0x3681D755 | lua | Lua script | REGTYPE | low |
| 0x376840D7 | vp6 | VP6 video | REGTYPE | low |
| 0x37979F71 | textual (SIMSWIKI) / cfg (REGTYPE) | conflict: SIMSWIKI "textual" (added 2008-07-27), REGTYPE "cfg" | SIMSWIKI + REGTYPE | conflict |
| 0x3B72D4F1 | rdx | unknown, name only | REGTYPE | low |
| 0x3C77532E | PSD | Photoshop PSD (added 2008-07-27) | SIMSWIKI + REGTYPE | high |
| 0x3C7E0F63 | MUSCLE | muscle (deformer) | SIMSWIKI + REGTYPE | high |
| 0x3D97A8E4 | cll | unknown, name only | REGTYPE | low |
| 0x3F8662EA | jpg | JPEG image | REGTYPE | medium |
| 0x3F9C28B5 | CURSOR (SIMSWIKI) / ani (REGTYPE) | conflict: SIMSWIKI "CURSOR" (added 2008-07-31), REGTYPE "ani" | SIMSWIKI + REGTYPE | conflict |
| 0x438F6347 | FLR | floor | SIMSWIKI + REGTYPE | high |
| 0x448AE7E2 | hkx | Havok packfile | REGTYPE | low |
| 0x476A98C7 | UFO | UFO | SIMSWIKI + REGTYPE | high |
| 0x477764F5 | pdn | unknown, name only | REGTYPE | low |
| 0x497767B9 | textual (SIMSWIKI) / pfx (REGTYPE) | conflict: SIMSWIKI "textual" (added 2008-07-27), REGTYPE "pfx" | SIMSWIKI + REGTYPE | conflict |
| 0x4AEB6BC6 | ANIM LIST / tlsa | animation list | SIMSWIKI + REGTYPE | high |
| 0x4B9EF6DC | structure | structure data | REGTYPE | low |
| 0x59770944 | XA | XA audio | REGTYPE | low |
| 0x5C74D18B | density | density map | REGTYPE | low |
| 0x5CAA5E28 | blockdata | block data | REGTYPE | low |
| 0x612B3191 | backgroundMap | background map | REGTYPE | low |
| 0x617715C4 | PYTHON (SIMSWIKI) / py (REGTYPE) | Python script (added 2008-07-27) | SIMSWIKI + REGTYPE | high |
| 0x617715D9 | textual (SIMSWIKI) / pd (REGTYPE) | conflict: SIMSWIKI "textual" (added 2008-07-27), REGTYPE "pd" | SIMSWIKI + REGTYPE | conflict |
| 0x67771F5C | js | JavaScript | REGTYPE | low |
| 0x6D7728C3 | db | database | REGTYPE | low |
| 0x72047DE2 | bmdl | building model | REGTYPE | low |
| 0x76A8F7D8 | geo / Noun.bin | geometry / noun data | REGTYPE | low |
| 0x76E1259D | physx_bin | PhysX binary | REGTYPE | low |
| 0x7C19AA7A | PART CAPABILITIES / pctp | part capabilities | SIMSWIKI + REGTYPE | high |
| 0x7EAB18FD | physx_xml | PhysX XML | REGTYPE | low |
| 0x83324B49 | cilia | creature part category: cilia | REGTYPE | low |
| 0x86C5375A | rdx9 | unknown, name only | REGTYPE | low |
| 0x8C042499 | look_table | look table | REGTYPE | low |
| 0x92EA4AAC | serverEventDef | server event definition | REGTYPE | low |
| 0x9B8E862F | world | world data | REGTYPE | low |
| 0xA11D3144 | markerSet | marker set | REGTYPE | low |
| 0xB8444447 | png.dds | combined PNG/DDS asset | REGTYPE | low |
| 0xCF6C21B8 | XML | XML (added 2008-07-27) | SIMSWIKI + REGTYPE | high |
| 0xD92AF091 | loottable | loot table | REGTYPE | low |
| 0xDA141C1B | populate | populate data | REGTYPE | low |
| 0xDBA35AE2 | look_algorithm | look algorithm | REGTYPE | low |
| 0xDD6233D6 | html | HTML | REGTYPE | low |
| 0xDFAD9F51 | cell | cell (stage) data | REGTYPE | low |
| 0xE1E7F839 | pollen_metadata.bak | backup of pollen metadata | REGTYPE | low |
| 0xE51118C3 | prop_new | new-format property file | REGTYPE | low |
| 0xE96B3B72 | blend | blend data | REGTYPE | low |
| 0xEA5118B0 | effdir | effect directory | REGTYPE | low |
| 0xEE17C6AD | ANIM / animation | animation | SIMSWIKI + REGTYPE | high |
| 0xEF7D16E1 | uiTexture | UI texture | REGTYPE | low |
| 0xEFBDA3FF | LAYOUT | layout | SIMSWIKI + REGTYPE | high |
| 0xF2B924B3 | ltp | unknown, name only | REGTYPE | low |
| 0xF43029A (see 0x0F43029A) | | | | |
| 0xF9C3D770 | random_creature | random creature | REGTYPE | low |
| 0x0A4D8D09 | bnk | audio bank | REGTYPE | low |
| 0x0A98EAF0 | json | JSON | REGTYPE | low |
| 0x0F43029A | creaturedata | creature data | REGTYPE | low |
| 0x1406A572 | kCreatureSpeciesMission | mission class (see section 2) | REGTYPE | low |
| 0x17F243B | kGameData / cGameData | game data class (see section 2) | REGTYPE | low |
| 0x1E4DAAE | kTribeHut | tribe hut (see section 2) | REGTYPE | low |

Note: several REGTYPE entries have two names for the same ID (for example `xml` at both
`0xCF6C21B8` and `0x1E639C34`; `layout` at both `0xEFBDA3FF` and `0xC7A91467`; `ttf` at both
`0x027C5CEF` and `0x276CA4B9`; `lvl` and `scn` both `0x047B8300`; `soundProp` and `audioProp` both
`0x02B9F662`; `htra` and `arth` both `0x0472329B`). These are aliases or historical duplicates, not
distinct types.

## 2. Simulator class hashes (not type IDs)

The REGTYPE file continues with two long sections of class-name hashes. These are the game's
internal C++ class names used in the object/Simulator system, used as instance or group identifiers
rather than as resource type IDs. They are included here because they document the engine's class
architecture and are useful for reverse engineering. All from REGTYPE.

Representative examples (the full list is in the cited file):

- Game/Simulator classes: `cSimulatorSpaceGame` (0x21F15DD), `cSimulatorUniverse` (0x2A8BFBE),
  `cSimulatorPlayerUFO` (0x2F011ED), `cSimulatorWalkAround` (0x2F404EA),
  `cCitySimulator` (0x21F1187), `cCombatSimulator` (0x23BA44D), `cPosseSimulator` (0x21F11AE),
  `cTimeOfDay` (0x21F10B2), `cPlanetRecord` (0x5220CB8), `cStarRecord` (0x3E4353C),
  `cSolarSystem` (0x38CF94C), `cEmpire` (0x18EB9D2), `cRelationshipManager` (0x18EB5C7),
  `cMissionManager` (0x2AA56D0), `cDramaManager` (0x3226D7D4), `cEmpireGrowthSim` (0x4238ECC),
  `cBadgeManager` (0x2CB0EE3).
- Object classes: `cGameData` (0x17F243B), `cCheatObject` (0x2A37E35), `cCity` (0x18C43E8),
  `cVehicle` (0x18C6DE8), `cCreatureAnimal` (0x18EB45E), `cCreatureCitizen` (0x18EB4B7),
  `cBuildingCityHall` (0x18EA1EB), `cBuildingIndustry` (0x18EA2CC), `cBuildingHouse` (0x18EB106),
  `cBuildingEntertainment` (0x1A56ABA), `cTribe` (0x18C6D19), `cTribeHut` (0x1E4DAAE),
  `cOrnament` (0x18C88E4), `cRock` (0x2A8FB3F), `cNest` (0x52AA6122), `cHerd` (0x1BE418E),
  `cEgg` (0x2A034CD), `cPlanet` (0x3275728), `cStar` (0x355C93A), `cToolObject` (0x4E3FAB5).
- Mission classes: `kMissionFetch` (0x2AFD284), `kMissionColonize` (0x2BA2A0E),
  `kMissionEradicate` (0x2F99994), `kMissionExplore` (0x32A12F1), `kMissionWar` (0x330FC49),
  `kMissionScan` (0x347092D), `kMissionTerraform` (0x35988F4), `kMissionTrade` (0x2ADEDFF),
  `kMissionAdventure` (0x46332A5), and many more.

## 3. Group, palette, and part names

The REGTYPE file also contains group/palette names (for example `UserCreated` 0x37216F9C,
`UserPollinated` 0x89CA8A12, `Creatures` 0xD49DE19D, `Buildings` 0x484FF0B8, `Vehicles` 0x39E64A16,
`CellGame` 0xA426730B, `CreatureGame` 0xAD56080C, `TribeGame` 0xF71FA311, `CivGame` 0xBEB528CB,
`SpaceGame` 0x2DB6DAD3, and many `FixedObject*` and `Scenario*` names) and creature part category
names (for example `cilia`, `flagella`, `jaw`, `poker`, `electric`, `jet`, `skinned`, `flag`, `gear`,
`light`, `roof`, `wall`, `wheel`, `flapping`). These are not resource type IDs. Source: REGTYPE.

## 4. Conflict summary

IDs where SIMSWIKI and REGTYPE give different names:

| ID | SIMSWIKI name | REGTYPE name |
|---|---|---|
| 0x022D2C83 | EAPD | pdr |
| 0x024A0E52 | Cheat | trigger |
| 0x029E333B | textual | voice |
| 0x02FAC0B6 | textual | locale |
| 0x03055F61 | textual | instrument |
| 0x12952634 | textual | dat |
| 0x25DF0112 | Script | gait |
| 0x2B6CAB5F | NOTE | txt |
| 0x37979F71 | textual | cfg |
| 0x3F9C28B5 | CURSOR | ani |
| 0x497767B9 | textual | pfx |
| 0x617715D9 | textual | pd |

Observation: SIMSWIKI uses the placeholder "textual" for a whole family of IDs that REGTYPE names
specifically (locale, cfg, dat, pd, pfx). SIMSWIKI's "textual" entries are therefore weaker and
should defer to REGTYPE where they conflict. The REGTYPE names themselves come from a modding tool
dictionary and are not official EA names.

## 5. Licenses and reuse (condensed)

Full details and URLs are in `prior-art.md` section 6. In short:

- GPL-3.0 (do not copy code into a permissive project): SporeModder FX, SporeModder FX Blender
  Addons, S3PI.
- No license shown (all rights reserved; do not copy code): Spore ModAPI SDK, PNG-Decoder-Python,
  PNG-Decoder-NetCore, Adventure-Decoder.
- MIT or Unlicense (permissive, verify the LICENSE file): ModAPI Launcher Kit, SporeTools,
  SporeHash-VSCode, Spore-CreationZIP-Porter, SporeGlobals-Python, Spore-AdventureLocalizer,
  sporeget, Spore_EP1Stegano-Python.
- Unresolved (resolve before any reuse): DBPF.js, chieltbest/dbpf (Rust, multi-license REUSE
  layout), kitlith/sims3-rs, Gibbed.Spore, DBPFSharp, jDBPFX, qfs.c.
- Data files such as `reg_type.txt` and `reg_property.txt` ship inside the GPL-3.0 SporeModder FX
  repository, so redistributing them in a non-GPL project needs the same care as copying code. They
  can be consulted for research regardless.

## 6. Not found publicly

- No single authoritative, EA-published resource type registry for Spore was found. The two sources
  above (a 2008 community page and a modding tool dictionary) are the best available.
- No public source maps every `0x` ID to a confirmed binary layout. The numeric IDs and names are
  documented; the per-type payload formats mostly are not.
