import struct
import unittest
from space_category_candidate_graph_v1 import read_shape, decompress_bounded

def frame(tail):
    return struct.pack('>IIHHII',2,0x7435a2d0,0x22,0x1c,1,0x208)+bytes(0x208)+tail

class ShapeTests(unittest.TestCase):
    def test_array_and_single_preserve_word_order(self):
        a=frame(struct.pack('>IHHII',0x7435a2d3,0x20,0x9c,1,12)+struct.pack('<III',1,2,3))
        b=frame(struct.pack('>IHH',0x7435a2d2,0x20,0)+struct.pack('<IIII',4,5,6,7))
        self.assertEqual(read_shape(a)['words_le'],[[1,2,3]])
        self.assertEqual(read_shape(b)['words_le'],[[4,5,6,7]])
        for bad in (a[:-1],a+b'\0',b[:-1],b+b'\0'):
            with self.assertRaises(ValueError): read_shape(bad)
    def test_rejects_refpack_header_size_before_decompression(self):
        # 10 FB + BE 24-bit decoded size; no stream commands needed.
        with self.assertRaisesRegex(ValueError, 'header size mismatch'):
            decompress_bounded(bytes.fromhex('10fbffffff'), 1)

    def test_rejects_invalid_array_count_stride(self):
        for count,stride in ((65,12),(1,8)):
            with self.assertRaises(ValueError): read_shape(frame(struct.pack('>IHHII',0x7435a2d3,0x20,0x9c,count,stride)))
if __name__=='__main__': unittest.main()
