import unittest
from space_category_ordered_preview_v1 import select_first
class OrderedSelection(unittest.TestCase):
 def test_first_present_and_missing(self):
  a=(1,2,3,10);b=(1,2,3,20);other=(7,2,3,30)
  self.assertEqual(select_first([[other],[a],[b]],(1,2,3)),(1,a))
  self.assertEqual(select_first([[b],[a]],(1,2,3)),(0,b))
  with self.assertRaises(ValueError):select_first([[other]],(1,2,3))
 def test_duplicate_selected_package_rejected(self):
  with self.assertRaises(ValueError):select_first([[(1,2,3,4),(1,2,3,5)]],(1,2,3))
if __name__=='__main__':unittest.main()
