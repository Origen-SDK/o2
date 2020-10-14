import origen
from origen.interface import BaseInterface, dut, tester


class Interface(BaseInterface):
    def func(self, name):
        with tester().specific("v93k_smt7") as v93k:
            t = v93k.new_test_suite(name)
            #import pdb; pdb.set_trace()
            t.test_method = v93k.test_methods.ac_tml.functional_test
            self.add_test(t)

        with tester().specific("v93ksmt7", "v93ksmt8") as (v93k7, v93k8):
            t = v93k.new_test_suite(name)
            t.test_method = v93k.test_methods.ac_tml.functional_test
            self.add_test(t)

        with tester().specific("uflex") as uflex:
            t = uflex.new_test_instance(name)
            self.add_test(t)
