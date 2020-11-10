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

    # This is not really a good example of how to structure a test type handler, it was originally written
    # for Ruby (Origen 1) and has been ported to Python as close as possible to the original structure for
    # testing and speed of implementation
    def func(self, name, duration="static", number=None, **kwargs):
        with tester().specific("igxl") as igxl:
            with self.instance_group(name, igxl, **kwargs) as (group):

                def func_igxl(i, block):
                    nonlocal name
                    nonlocal duration
                    nonlocal number
                    nonlocal igxl
                    nonlocal kwargs
                    nonlocal group
                    if number and i:
                        number += i
                    ins = igxl.new_test_instance(name,
                                                 library="std",
                                                 template="functional")
                    if duration == "dynamic":
                        igxl.set_wait_flags(ins, "a")
                    if kwargs.get("pin_levels"):
                        ins.pin_levels = kwargs.pop("pin_levels")
                    if group:
                        p = igxl.new_patset(f"{name}_b{i}_pset")
                        p.append(f"{name}_b{i}.PAT")
                        p.append('nvm_global_subs.PAT', start_label="subr")
                        ins.pattern = p.name
                        if i == 0:
                            self.add_test(ins, **kwargs)
                    else:
                        p = igxl.new_patset(f"{name}_pset")
                        p.append(f"{name}.PAT")
                        p.append('nvm_global_subs.PAT', start_label="subr")
                        ins.pattern = p.name
                        if kwargs.get("cz_setup"):
                            self.cz(ins, kwargs["cz_setup"], **kwargs)
                        else:
                            self.add_test(ins, **kwargs)

                if group is not None:
                    for i, block in enumerate(dut().blocks):
                        func_igxl(i, block)
                else:
                    func_igxl(None, None)

        with tester().specific("v93k") as v93k:
            with self.v93k_group(name, v93k, **kwargs) as (group):

                def func_v93k(i, block):
                    nonlocal name
                    nonlocal duration
                    nonlocal number
                    nonlocal v93k
                    if number and i:
                        number += i
                    tm = v93k.new_test_method("functional_test",
                                              library="ac_tml")
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

                if group is not None:
                    for i, block in enumerate(dut().blocks):
                        func_v93k(i, block)
                else:
                    func_v93k(None, None)

    @contextmanager
    def instance_group(self, name, igxl, **kwargs):
        if kwargs.get("by_block"):
            with igxl.test_instance_group(name) as group:
                yield group
        else:
            yield None

    @contextmanager
    def v93k_group(self, name, v93k, **kwargs):
        if kwargs.get("by_block"):
            with self.group(name) as group:
                yield group
        else:
            yield None
