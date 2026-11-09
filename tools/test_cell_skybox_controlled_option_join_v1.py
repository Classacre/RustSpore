import unittest
import struct
from cell_skybox_controlled_option_join_v1 import select, pick_option
class ProviderOrder(unittest.TestCase):
 def test_first_match_and_duplicate_rejection(self):
  row={'type':1,'group':2,'instance':3}
  self.assertEqual(select([('first',[row]),('second',[row])],1,2,3)[0],'first')
  self.assertEqual(select([('first',[]),('second',[row])],1,2,3)[0],'second')
  with self.assertRaises(ValueError):select([('bad',[row,row])],1,2,3)
  with self.assertRaises(ValueError):select([('empty',[])],1,2,3)
 def test_catalog_bounds_and_option_identity(self):
  def field(key,values):
   return struct.pack('>IHHII',key,10,0x9c,len(values),4)+struct.pack('>'+str(len(values))+'I',*values)
  catalog=bytes(4)+field(0x05daafff,[0x046170a0])+field(0x05dab000,[1])+field(0x05dab001,[12])
  self.assertEqual(pick_option(catalog,2),(0,1,12))
  with self.assertRaises(ValueError):pick_option(catalog,0)
  with self.assertRaises(ValueError):pick_option(catalog,12)
if __name__=='__main__':unittest.main()
