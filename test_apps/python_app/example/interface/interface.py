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
    # testing and speed of implementation. Under Python it seems overly complex since context managers are
    # not closure/iterators like they are in Ruby, probably if writing this from scratch in Python it would
    # have been done differently.
    def func(self, name, duration="static", number=None, **kwargs):
        with tester().eq("igxl") as igxl:
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
                                                 template="functional",
                                                 **kwargs)
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
                            self.add_cz_test(ins, kwargs.pop("cz_setup"),
                                             **kwargs)
                        else:
                            self.add_test(ins, **kwargs)

                if group is not None:
                    for i, block in enumerate(dut().blocks):
                        func_igxl(i, block)
                else:
                    func_igxl(None, None)

        with tester().eq("v93k") as v93k:
            with self.v93k_group(name, v93k, **kwargs) as (group):

                def func_v93k(i, block):
                    nonlocal name
                    nonlocal duration
                    nonlocal number
                    nonlocal v93k
                    nonlocal kwargs
                    nonlocal group
                    if number and i:
                        number += i
                    tm = v93k.new_test_method("functional_test",
                                              library="ac_tml",
                                              **kwargs)
                    ts = v93k.new_test_suite(name, **kwargs)
                    ts.test_method = tm
                    with tester().eq("v93ksmt8"):
                        if kwargs.get("pin_levels"):
                            ts.spec = kwargs.pop("pin_levels")
                        else:
                            ts.spec = 'specs.Nominal'
                    with tester().eq("v93ksmt7"):
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

    def por(self, **kwargs):
        options = {"instance_not_available": True}
        options.update(kwargs)
        with tester().eq("igxl"):
            self.add_test('por_ins', **options)
        with tester().neq("igxl"):
            self.func('por_ins', **options)

    def mto_memory(self, name, **kwargs):
        options = {"duration": "static"}
        options.update(kwargs)

        with tester().eq("j750") as j750:
            ins = j750.new_test_instance(name,
                                         library="std",
                                         template="mto_memory",
                                         **options)
            if options["duration"] == "dynamic":
                j750.set_wait_flags(ins, "a")
            if options.get("pin_levels"):
                ins.pin_levels = options.pop("pin_levels")
            p = j750.new_patset(f"{name}_pset")
            p.append(f"{name}.PAT")
            p.append('nvm_global_subs.PAT', start_label="subr")
            ins.pattern = p.name
            if options.get("cz_setup"):
                self.add_cz_test(ins, options.pop["cz_setup"], **options)
            else:
                self.add_test(ins, **options)

    def meas(self, name, **kwargs):
        options = {"duration": "static"}
        options.update(kwargs)

        if "meas" not in name:
            name = f"meas_#{name}"

        with tester().eq("igxl") as igxl:
            with tester().eq("uflex") as uflex:
                ins = uflex.new_test_instance(name,
                                              library="std",
                                              template="functional",
                                              **options)
                if options["duration"] == "dynamic":
                    uflex.set_wait_flags(ins, "a")
                if options.get("pin_levels"):
                    ins.pin_levels = options.pop("pin_levels")
                ins.lo_limit = options.get("lo_limit")
                ins.hi_limit = options.get("hi_limit")
                ins.scale = options.get("scale")
                ins.units = options.get("units")
                ins.defer_limits = options.get("defer_limits")
                p = uflex.new_patset(f"{name}_pset")
                p.append(f"{name}.PAT")
                p.append('nvm_global_subs.PAT', start_label="subr")
                ins.pattern = p.name
                if options.get("cz_setup"):
                    self.add_cz_test(ins, options.pop["cz_setup"], **options)
                else:
                    self.add_test(ins, **options)

            with tester().eq("j750") as j750:
                if options.get("pins") == "hi_v":
                    ins = j750.new_test_instance(name,
                                                 library="std",
                                                 template="board_pmu",
                                                 **options)
                elif options.get("pins") == "power":
                    ins = j750.new_test_instance(name,
                                                 library="std",
                                                 template="powersupply",
                                                 **options)
                else:
                    ins = j750.new_test_instance(name,
                                                 library="std",
                                                 template="pin_pmu",
                                                 **options)
                if options["duration"] == "dynamic":
                    j750.set_wait_flags(ins, "a")
                if options.get("pin_levels"):
                    ins.pin_levels = options.pop("pin_levels")
                ins.lo_limit = options.get("lo_limit")
                ins.hi_limit = options.get("hi_limit")
                p = j750.new_patset(f"{name}_pset")
                p.append(f"{name}.PAT")
                p.append('nvm_global_subs.PAT', start_label="subr")
                ins.pattern = p.name
                if options.get("cz_setup"):
                    self.add_cz_test(ins, options.pop["cz_setup"], **options)
                else:
                    self.add_test(ins, **options)

        with tester().eq("v93k") as v93k:
            tm = v93k.new_test_method("general_pmu",
                                      library="dc_test",
                                      **options)
            ts = v93k.new_test_suite(name, **options)
            ts.test_method = tm
            with tester().eq("v93ksmt8"):
                if kwargs.get("pin_levels"):
                    ts.spec = kwargs.pop("pin_levels")
                else:
                    ts.spec = 'specs.Nominal'
            with tester().eq("v93ksmt7"):
                if kwargs.get("pin_levels"):
                    ts.levels = kwargs.pop["pin_levels"]

            ts.lo_limit = options.get("lo_limit")
            ts.hi_limit = options.get("hi_limit")
            ts.pattern = name
            self.add_test(ts, **options)
