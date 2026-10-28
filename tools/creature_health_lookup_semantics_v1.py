"""Bounded result selection from SporeApp C1F940 -> BCE6B0, version 1.

Static evidence: target/rea/20261028-source-runtime-v6/{80,81}-analyze_function.json,
SporeApp.exe SHA-256 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a.
This models raw 32-bit selection after externally performed lookups; no native
floating-point execution, lookup, ownership, variant storage, side effects, or
complete Creature behavior is claimed. Wrong-type data is an explicit runtime
input: the static image has zero-filled storage, but runtime writes are unknown.
"""
from dataclasses import dataclass


@dataclass(frozen=True)
class ObservedVariant:
    type_code: int
    selected_word: int


def select_health_word(namespace_found: bool, property_found: bool,
                       variant: ObservedVariant | None, wrong_type_word: int) -> int:
    """Select the source memory word before the x87 load, as raw bits.

    Failure of either lookup selects the caller's 100.0 fallback. A successful
    property lookup selects the variant word for type 0x0D or 0x10; other types
    select a word loaded from a global whose runtime contents are not established.
    The result does not model x87 NaN handling or identify any PROP resource or active value.
    """
    if not 0 <= wrong_type_word <= 0xFFFFFFFF:
        raise ValueError('wrong_type_word outside u32')
    if not namespace_found or not property_found:
        return 0x42C80000
    if variant is None:
        raise ValueError('successful property lookup needs a variant')
    if not 0 <= variant.selected_word <= 0xFFFFFFFF:
        raise ValueError('selected_word outside u32')
    return variant.selected_word if variant.type_code in (0x0D, 0x10) else wrong_type_word

