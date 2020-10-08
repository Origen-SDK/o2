import origen
from origen.interface import BaseInterface, dut, tester


class Interface(BaseInterface):
    def func(self, name):
        with tester().specific("v93k_smt7") as v93k:
            t = v93k.new_test_suite(name)
            t.test_method = v93k.test_methods.ac_tml.functional_test
            self.add_test(t)

        with tester().specific("uflex") as uflex:
            t = uflex.new_test_instance(name)
            self.add_test(t)
