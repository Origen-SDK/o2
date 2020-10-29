import origen
from origen.interface import BaseInterface, dut, tester
#import pdb; pdb.set_trace()


class Interface(BaseInterface):
    def func(self, name):
        with tester().specific("v93k_smt7") as v93k:
            t = v93k.test_suites.add(name)
            t.test_method = v93k.test_methods.ac_tml.functional_test
            t.force_serial = True
            t.output = "ReportUI"
            self.add_test(t)

        with tester().specific("v93ksmt7", "v93ksmt8") as (v93k7, v93k8):
            pass

        with tester().specific("uflex") as uflex:
            t = uflex.test_instances.std.functional(name)
            t.threading = True
            self.add_test(t)
