import struct
import unittest
from raster_extract import decode_dxt1_block, decode_dxt5_block
from rw4_texture_digest import FMT_DXT5, decode_dxt

class Dxt5ColorTest(unittest.TestCase):
    def test_four_color_with_ordered_and_equal_endpoints(self):
        for c0, c1, value in ((0, 0xffff, 170), (0xffff, 0xffff, 255)):
            color = struct.pack('<HHI', c0, c1, 0xffffffff)
            self.assertEqual(decode_dxt5_block(bytes([42, 0]) + bytes(6) + color), [(value, value, value, 42)] * 16)
            self.assertEqual(decode_dxt1_block(color), [(0, 0, 0, 0)] * 16)
            self.assertEqual(decode_dxt(bytes([42, 0]) + bytes(6) + color, 4, 4, FMT_DXT5), bytes([value, value, value, 42]) * 16)

if __name__ == '__main__': unittest.main()
