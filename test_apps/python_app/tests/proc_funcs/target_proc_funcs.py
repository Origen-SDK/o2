def show_target_setup(q, options):
    if options and "with_env" in options:
        import os
        os.environ.update(options["with_env"])
    import origen
    origen.target.setup()
    q.put(('targets', origen.targets.current))

def put_target(origen, q, name):
    q.put((f'target_{name}', origen.target.current))
    q.put((f'tester_{name}', origen.tester.target()))
    if origen.dut is None:
        q.put((f'dut_{name}', origen.dut))
    else:
        q.put((f'dut_{name}', origen.dut.name))
    q.put((f'first_load_done_{name}', origen.target.first_load_done))
    q.put((f'setup_pending_{name}', origen.target.setup_pending))

def test_loading_targets_set_by_app(q, options):
    import origen
    put_target(origen, q, "pre_load")

    origen.target.load()
    put_target(origen, q, "post_load")

def test_getting_the_current_target(q, options):
    import origen
    origen.target.load()
    q.put(('current_targets', origen.target.current))
    q.put(('current', origen.target.current))

def test_setup_and_load(q, options):
    from tests.shared import Targets

    import origen
    put_target(origen, q, "pre_setup")

    origen.target.setup([Targets.eagle.name, Targets.uflex.name])
    put_target(origen, q, "post_setup_1")

    origen.target.setup([Targets.falcon.name, Targets.sim.name, Targets.smt7.name])
    put_target(origen, q, "post_setup_2")

    origen.target.load()
    put_target(origen, q, "post_load_1_2")

    origen.target.setup([Targets.eagle.name, Targets.sim.name, Targets.uflex.name])
    put_target(origen, q, "post_setup_3")

    origen.target.load()
    put_target(origen, q, "post_load_3")

def test_loading_targets_explicitly(q, options):
    from tests.shared import Targets

    import origen
    put_target(origen, q, "pre_load")

    origen.target.load(Targets.eagle.name, Targets.smt7.name)
    put_target(origen, q, "post_load")

def test_reloading_targets(q, options):
    from tests.shared import Targets
    r = "creg0"

    import origen
    origen.target.load(Targets.eagle.name, Targets.j750.name)
    put_target(origen, q, "pre_config")
    q.put(("ast_size_pre_config", len(origen.test_ast()["children"])))
    q.put(("reg_data_pre_config", origen.dut.reg(r).data()))
    q.put(("timeset_pre_config", origen.tester.timeset))

    origen.tester.set_timeset("simple")
    origen.dut.arm_debug.switch_to_swd()
    origen.dut.reg(r).write(0xC)

    q.put(("ast_size_post_config", len(origen.test_ast()["children"])))
    q.put(("reg_data_post_config", origen.dut.reg(r).data()))
    q.put(("timeset_post_config", origen.tester.timeset.name))

    origen.target.load()
    put_target(origen, q, "post_reload")
    q.put(("ast_size_post_reload", len(origen.test_ast()["children"])))
    q.put(("reg_data_post_reload", origen.dut.reg(r).data()))
    q.put(("timeset_post_reload", origen.tester.timeset))

def test_invalid_target(q, options):
    from tests.shared import Targets
    from tests.cli.core_cmds.target import TargetCLI
    import origen
    put_target(origen, q, "pre_setup")

    origen.target.setup([Targets.falcon.name, TargetCLI.utn])
    put_target(origen, q, "post_setup")

    origen.target.load()