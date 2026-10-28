import struct
import unittest
from civilization_prop_candidate_v1 import CONSUMER_FIELDS, FIELDS, KEY, inspect_bytes
from audio_event_parse import fnv1_32


def item(ident, typ, bits):
    return struct.pack('>IHHI', ident, typ, 0, bits)


def source():
    return (struct.pack('>I', 5) + item(0x134CA1E3, 9, 25) +
            item(0x231A946B, 13, 0x80000000) +
            b''.join(item(ident, 13, ident) for ident in sorted(CONSUMER_FIELDS)))


class CandidateTests(unittest.TestCase):
    def test_candidate_names_hash_to_measured_identifiers(self):
        self.assertEqual(fnv1_32("communityeditor"), KEY[0])
        for ident, (name, _) in FIELDS.items():
            self.assertEqual(fnv1_32(name.lower()), ident)

    def test_bits_are_preserved_including_negative_zero(self):
        result = inspect_bytes(source())
        self.assertEqual(result['fields']['CivicObjectCost']['value_bits'], '00000019')
        self.assertEqual(result['fields']['CityRotationRingScaleFactor']['value_bits'], '80000000')
        self.assertEqual(result['native_consumer_float_bits']['039D48E7'], '039D48E7')
        self.assertIn('unverified', result['status'])

    def test_missing_duplicate_wrong_type_and_truncated_rejected(self):
        a = item(0x134CA1E3, 9, 25)
        b = item(0x231A946B, 13, 0x41600000)
        extra = b''.join(item(ident, 13, ident) for ident in sorted(CONSUMER_FIELDS))
        samples = [struct.pack('>I', 4) + a + extra,
                   struct.pack('>I', 6) + a + a + b + extra,
                   struct.pack('>I', 5) + item(0x134CA1E3, 13, 25) + b + extra,
                   struct.pack('>I', 5) + a + b + item(0x039D48E7, 9, 0) +
                   b''.join(item(ident, 13, ident) for ident in sorted(CONSUMER_FIELDS - {0x039D48E7})),
                   source()[:-1]]
        for data in samples:
            with self.subTest(data=data), self.assertRaises(ValueError):
                inspect_bytes(data)


if __name__ == '__main__':
    unittest.main()
