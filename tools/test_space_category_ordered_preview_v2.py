import unittest
import struct
from space_category_ordered_preview_v2 import select_first, read_shape
class OrderedSelection(unittest.TestCase):
 def test_first_present_and_missing(self):
  a=(1,2,3,10);b=(1,2,3,20);other=(7,2,3,30)
  self.assertEqual(select_first([[other],[a],[b]],(1,2,3)),(1,a))
  self.assertEqual(select_first([[b],[a]],(1,2,3)),(0,b))
  with self.assertRaises(ValueError):select_first([[other]],(1,2,3))
 def test_duplicate_selected_package_rejected(self):
  with self.assertRaises(ValueError):select_first([[(1,2,3,4),(1,2,3,5)]],(1,2,3))
class StrictNode(unittest.TestCase):
 def test_three_properties_and_rejection(self):
  scalar=struct.pack('>IHH',0x00b2cccb,0x20,0)+struct.pack('<IIII',1,2,3,4)
  opaque=struct.pack('>IHHII',0x7435a2d0,0x22,0x1c,1,0x208)+bytes(0x208)
  array=struct.pack('>IHHII',0x7435a2d3,0x20,0x9c,1,12)+struct.pack('<III',5,0,0)
  node=struct.pack('>I',3)+scalar+opaque+array
  result=read_shape(node);self.assertEqual(result['words_le'],[[5,0,0]]);self.assertEqual(result['other_scalar_properties'][0]['words_le'],[1,2,3,4])
  for bad in (node[:-1],node+b'\0',struct.pack('>I',4)+node[4:]):
   with self.assertRaises(ValueError):read_shape(bad)
 def test_duplicate_or_unrecognized_property(self):
  opaque=struct.pack('>IHHII',0x7435a2d0,0x22,0x1c,1,0x208)+bytes(0x208)
  scalar=struct.pack('>IHH',0x7435a2d2,0x20,0)+bytes(16)
  with self.assertRaises(ValueError):read_shape(struct.pack('>I',3)+opaque+scalar+scalar)
if __name__=='__main__':unittest.main()
