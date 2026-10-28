import struct
import unittest
from creature_health_candidate_v2 import FIELD, inspect_bytes


def one(ident=FIELD, typ=0x0D, bits=0x3F800000):
    return struct.pack('>IIHHI', 1, ident, typ, 0, bits)


class CandidateTest(unittest.TestCase):
    def test_preserves_float_bits(self):
        self.assertEqual(inspect_bytes(one(bits=0x80000000))['float_bits'], '80000000')

    def test_rejects_missing_duplicate_and_wrong_type(self):
        for data in (one(ident=0), struct.pack('>I', 2) + one()[4:] * 2, one(typ=0x09), one()[:-1]):
            with self.assertRaises(ValueError):
                inspect_bytes(data)


if __name__ == '__main__':
    unittest.main()
