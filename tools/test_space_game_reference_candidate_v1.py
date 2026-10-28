import struct
import unittest
from space_game_reference_candidate_v1 import inspect_bytes


def fixture():
    first = struct.pack('>IIHHII', 2, 0x7435A2D0, 0x22, 0x1C, 1, 0x208) + bytes(0x208)
    second = struct.pack('>IHHII', 0x7435A2D3, 0x20, 0x9C, 5, 12)
    return first + second + b''.join(struct.pack('<III', i, 0, 0) for i in range(5))


class ReferenceShapeTest(unittest.TestCase):
    def test_five_little_endian_triples(self):
        result = inspect_bytes(fixture())
        self.assertEqual(result['reference_words_le'][4], ['00000004', '00000000', '00000000'])

    def test_rejects_truncation_trailing_and_wrong_stride(self):
        data = fixture()
        wrong = bytearray(data)
        struct.pack_into('>I', wrong, 20 + 0x208 + 12, 8)
        for invalid in (data[:-1], data + b'\0', bytes(wrong)):
            with self.subTest(length=len(invalid)):
                with self.assertRaises(ValueError):
                    inspect_bytes(invalid)


if __name__ == '__main__':
    unittest.main()
