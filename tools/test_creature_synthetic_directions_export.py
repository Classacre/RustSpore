import importlib.util,unittest,json
from pathlib import Path
p=Path(__file__).with_name('creature_synthetic_directions_export.py');spec=importlib.util.spec_from_file_location('direction_export',p);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Directions(unittest.TestCase):
 def setUp(self):
  self.args=[m.BASE.read_bytes(),(m.FOLDER/'result.json').read_bytes(),(m.FOLDER/'host-stdout.bin').read_bytes(),(m.ROOT/'target/creature-skin-deform-position-prep-v1-20261109/host-stdout.bin').read_bytes()]
 def test_matched(self):
  a=m.compose(*self.args);self.assertEqual(len(a['conditional_synthetic_cpu_directions']['normal_words_hex']),85);self.assertEqual(len(a['conditional_synthetic_cpu_directions']['tangent_words_hex']),85)
 def test_tampering_each_pinned_input(self):
  for i in range(4):
   args=self.args.copy();args[i]+=b'x'
   with self.assertRaises(ValueError):m.compose(*args)
 def test_mapped_source_even_when_repinning(self):
  args=self.args.copy();a=json.loads(args[0]);a['conditional_eight_group_export']['attribute_words_hex'][1][0][0]='00000000';args[0]=json.dumps(a).encode();old=m.PINS['base'];m.PINS['base']=m.sha(args[0])
  try:
   with self.assertRaisesRegex(ValueError,'mapping'):m.compose(*args)
  finally:m.PINS['base']=old
if __name__=='__main__':unittest.main()
