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
    #        t = uflex.new_test_instances.std.functional(name)
    #        t.threading = True
    #        self.add_test(t)

    def func(self, name, duration="static", number=None, **kwargs):
        with tester().specific("igxl") as igxl:
            with self.block_loop(name, **kwargs) as (block, i, group):
                if number and i:
                    number += i
                ins = igxl.new_test_instance(name,
                                             library="std",
                                             template="functional")
                if duration == "dynamic":
                    ins.set_wait_flags("a")
                if kwargs.get("pin_levels"):
                    ins.pin_levels = kwargs.pop("pin_levels")
                if group:
                    p = igxl.new_patset(f"{name}_b{i}_pset")
                    p.append(f"{name}_b{i}.PAT")
                    p.append('nvm_global_subs.PAT', start_label="subr")
                    ins.pattern = p.name
                    if i == 0:
                        self.add_test(group, **kwargs)
                else:
                    p = igxl.new_patset(f"{name}_pset")
                    p.append(f"{name}.PAT")
                    p.append('nvm_global_subs.PAT', start_label="subr")
                    ins.pattern = p.name
                    if kwargs.get("cz_setup"):
                        self.cz(ins, kwargs["cz_setup"], **kwargs)
                    else:
                        self.add_test(ins, **kwargs)

        with tester().specific("v93k") as v93k:
            with self.block_loop(name, **kwargs) as (block, i, _):
                if number and i:
                    number += i
                tm = v93k.new_test_method("functional_test", library="ac_tml")
                ts = v93k.new_test_suite(name, **kwargs)
                ts.test_method = tm
                with tester().specific("v93ksmt8"):
                    if kwargs.get("pin_levels"):
                        ts.spec = kwargs.pop("pin_levels")
                    else:
                        ts.spec = 'specs.Nominal'
                with tester().specific("v93ksmt7"):
                    if kwargs.get("pin_levels"):
                        ts.levels = kwargs.pop["pin_levels"]
                if block:
                    ts.pattern = f"{name}_b{i}"
                else:
                    ts.pattern = name
                self.add_test(ts, **kwargs)

    @contextmanager
    def block_loop(self, name, **kwargs):
        if kwargs.get("by_block"):
            with tester().specific("igxl") as igxl:
                with igxl.test_instances.group() as group:
                    group.name = name
                    for i, block in enumerate(dut.blocks):
                        yield block, i, group

            with tester().specific("v93k") as v93k:
                with self.group(name, **kwargs) as group:
                    for i, block in enumerate(dut.blocks):
                        yield block, i, None
        else:
            yield None, None, None
