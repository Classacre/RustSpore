import unittest,json,importlib.util
from pathlib import Path
p=Path(__file__).with_name('creature_synthetic_halfturn_export.py');sp=importlib.util.spec_from_file_location('halfturn_export',p);m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
class Halfturn(unittest.TestCase):
 def setUp(self):
  self.args=[m.BASE.read_bytes(),(m.FOLDER/'result.json').read_bytes(),(m.FOLDER/'host-stdout.bin').read_bytes(),(m.ROOT/'target/creature-skin-quaternion-product-deform-prep-v3-20261109/host-stdout.bin').read_bytes(),(m.FOLDER/'manifest.json').read_bytes(),(m.FOLDER/'deform-model.bin').read_bytes(),(m.FOLDER/'directions-model.bin').read_bytes(),(m.FOLDER/'synthetic-palette.bin').read_bytes()]
 def test_changed_complete_outputs(self):
  x=m.compose(*self.args);self.assertEqual(len(x['conditional_synthetic_cpu_deformation']['position_words_hex']),85);self.assertEqual(len(x['conditional_synthetic_cpu_directions']['normal_words_hex']),85);self.assertEqual(len(x['conditional_synthetic_cpu_directions']['tangent_words_hex']),85);self.assertEqual(len(x['conditional_synthetic_cpu_deformation']['synthetic_palette_words_hex']),120);old=json.loads(self.args[0]);self.assertNotEqual(x['conditional_synthetic_cpu_deformation']['position_words_hex'],old['conditional_synthetic_cpu_deformation']['position_words_hex'])
 def test_rejects_each_tampered_input(self):
  for i in range(len(self.args)):
   a=self.args.copy();a[i]+=b'x'
   with self.assertRaises(ValueError):m.compose(*a)
 def test_rejects_wrong_mapping_even_if_rehashed(self):
  a=self.args.copy();x=json.loads(a[0]);x['conditional_eight_group_export']['attribute_words_hex'][1][0][0]='00000000';a[0]=json.dumps(x).encode();old=m.PINS['base'];m.PINS['base']=m.sha(a[0])
  try:
   with self.assertRaisesRegex(ValueError,'mapping'):m.compose(*a)
  finally:m.PINS['base']=old
if __name__=='__main__':unittest.main()
