import unittest
from creature_health_lookup_semantics_v1 import ObservedVariant, select_health_word


class LookupTest(unittest.TestCase):
    def test_failure_uses_caller_default(self):
        for namespace, prop in ((False, False), (False, True), (True, False)):
            self.assertEqual(select_health_word(namespace, prop, None, 7), 0x42C80000)

    def test_success_uses_variant_or_runtime_global(self):
        for typ in (0x0D, 0x10):
            self.assertEqual(select_health_word(True, True, ObservedVariant(typ, 0x80000000), 7), 0x80000000)
        self.assertEqual(select_health_word(True, True, ObservedVariant(9, 1), 7), 7)
        with self.assertRaises(ValueError):
            select_health_word(True, True, None, 0)


if __name__ == '__main__':
    unittest.main()
