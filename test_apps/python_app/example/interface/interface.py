import origen
from origen.interface import BaseInterface, dut, tester
from contextlib import contextmanager, ContextDecorator
#import pdb; pdb.set_trace()


class Interface(BaseInterface):
    #def func(self, name):
    #    with tester().specific("v93k_smt7") as v93k:
    #        t = v93k.test_suites.add(name)
    #        t.test_method = v93k.test_methods.ac_tml.functional_test
    #        t.force_serial = True
    #        t.output = "ReportUI"
    #        self.add_test(t)

    #    with tester().specific("v93ksmt7", "v93ksmt8") as (v93k7, v93k8):
    #        pass

    #    with tester().specific("uflex") as uflex:
    #        t = uflex.test_instances.std.functional(name)
    #        t.threading = True
    #        self.add_test(t)

    def func(self, name, duration="static", number=None, **kwargs):
        with tester().specific("igxl") as igxl:
            with self.block_loop(name, kwargs) as (block, i, group):
                if number and i:
                    number += i
                ins = igxl.test_instances.std.functional(name)
                if duration == "dynamic":
                    ins.set_wait_flags("a")
                if kwargs["pin_levels"]:
                    ins.pin_levels = kwargs.pop("pin_levels")
                if group:
                    pname = f"{name}_b{i}_pset"
                    igxl.patsets.add(pname, [{
                        "pattern": f"{name}_b{i}.PAT"
                    }, {
                        "pattern": 'nvm_global_subs.PAT',
                        "start_label": 'subr'
                    }])
                    ins.pattern = pname
                    if i == 0:
                        flow.add_test(group, kwargs)
                else:
                    pname = f"{name}_pset"
                    igxl.patsets.add(pname, [{
                        "pattern": f"{name}.PAT"
                    }, {
                        "pattern": 'nvm_global_subs.PAT',
                        "start_label": 'subr'
                    }])
                    ins.pattern = pname
                    if kwargs["cz_setup"]:
                        flow.cz(ins, kwargs["cz_setup"], kwargs)
                    else:
                        flow.add_test(ins, kwargs)

        with tester().specific("v93k") as v93k:
            with self.block_loop(name, kwargs) as (block, i):
                if number and i:
                    number += i
                tm = v93k.test_methods.ac_tml.ac_test.functional_test
                ts = test_suites.add(name, kwargs)
                ts.test_method = tm
                with tester().specific("v93ksmt8"):
                    if kwargs["pin_levels"]:
                        ts.spec = kwargs.pop("pin_levels")
                    else:
                        ts.spec = 'specs.Nominal'
                with tester().specific("v93ksmt7"):
                    if kwargs["pin_levels"]:
                        ts.levels = kwargs.pop["pin_levels"]
                if block:
                    ts.pattern = f"{name}_b{i}"
                else:
                    ts.pattern = name
                flow.add_test(ts, kwargs)

    @contextmanager
    def block_loop(self, name, **kwargs):
        if kwargs["by_block"]:
            with tester().specific("igxl") as igxl:
                with igxl.test_instances.group() as group:
                    group.name = name
                    for i, block in enumerate(dut.blocks):
                        yield block, i, group

            with tester().specific("v93k") as v93k:
                with self.group(name, kwargs) as group:
                    for i, block in enumerate(dut.blocks):
                        yield block, i, None
        else:
            yield None, None, None
