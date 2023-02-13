import pytest, origen
from tests.proc_funcs import target_proc_funcs
from origen.helpers.env import in_new_origen_proc
from shared import PythonAppCommon, Targets
from cli.core_cmds.target import TargetCLI

class TestTarget(PythonAppCommon, Targets.TargetFixtures):
    @classmethod
    def assert_retn(cls, retn, name, target, tester, dut, first_load_done, setup_pending):
        assert retn[f"target_{name}"] == target
        assert retn[f"tester_{name}"] == tester
        assert retn[f"dut_{name}"] == dut
        assert retn[f"first_load_done_{name}"] == first_load_done
        assert retn[f"setup_pending_{name}"] == setup_pending

    @classmethod
    def assert_pre_setup_retn(cls, retn, name):
        assert retn[f"target_{name}"] == None
        assert retn[f"tester_{name}"] == []
        assert retn[f"dut_{name}"] == None
        assert retn[f"first_load_done_{name}"] == False
        assert retn[f"setup_pending_{name}"] == False

    def test_loading_targets_set_by_app(self, eagle_with_smt7, smt7):
        TargetCLI.target_cmd.set.run(eagle_with_smt7.name)
        retn = in_new_origen_proc(mod=target_proc_funcs)
        self.assert_pre_setup_retn(retn, "pre_load")
        self.assert_retn(retn, "post_load", [eagle_with_smt7.fp], [smt7.tester_name], "dut", True, False)

    def test_multiple_targets_set_by_app(self, eagle, uflex, j750, sim):
        TargetCLI.target_cmd.set.run(eagle.name, uflex.name, j750.name, sim.name)
        retn = in_new_origen_proc(func=target_proc_funcs.test_loading_targets_set_by_app)
        self.assert_pre_setup_retn(retn, "pre_load")
        self.assert_retn(
            retn,
            "post_load",
            [eagle.fp, uflex.fp, j750.fp, sim.fp],
            [uflex.tester_name, j750.tester_name, sim.tester_name],
            "dut",
            True,
            False
        )

    def test_getting_the_current_target(self, eagle):
        TargetCLI.target_cmd.set.run(eagle.name)
        retn = in_new_origen_proc(mod=target_proc_funcs)
        t = [eagle.fp]
        assert retn['current_targets'] == t
        assert retn['current'] == t
        assert origen.target == origen.targets

    def test_setup_and_load(self, eagle, uflex, falcon, sim, smt7):
        retn = in_new_origen_proc(mod=target_proc_funcs)
        self.assert_pre_setup_retn(retn, "pre_setup")
        self.assert_retn(retn, "post_setup_1", [eagle.name, uflex.name], [], None, False, True)
        self.assert_retn(retn, "post_setup_2", [falcon.name, sim.name, smt7.name], [], None, False, True)
        self.assert_retn(retn, "post_load_1_2", [falcon.name, sim.name, smt7.name], [sim.tester_name, smt7.tester_name], "dut", True, False)
        self.assert_retn(retn, "post_setup_3", [eagle.name, sim.name, uflex.name], [sim.tester_name, smt7.tester_name], "dut", True, True)
        self.assert_retn(retn, "post_load_3", [eagle.name, sim.name, uflex.name], [sim.tester_name, uflex.tester_name], "dut", True, False)

    def test_loading_targets_explicitly(self, eagle, smt7):
        retn = in_new_origen_proc(mod=target_proc_funcs)
        self.assert_pre_setup_retn(retn, "pre_load")
        self.assert_retn(retn, "post_load", [eagle.name, smt7.name], [smt7.tester_name], "dut", True, False)

    def test_reloading_targets(self, eagle, j750):
        retn = in_new_origen_proc(mod=target_proc_funcs)
        
        n = "pre_config"
        self.assert_retn(retn, n, [eagle.name, j750.name], [j750.tester_name], "dut", True, False)
        assert retn[f"ast_size_{n}"] == 0
        assert retn[f"reg_data_{n}"] == 0
        assert retn[f"timeset_{n}"] is None

        n = "post_config"
        assert retn[f"ast_size_{n}"] == 3
        assert retn[f"reg_data_{n}"] == 0xC
        assert retn[f"timeset_{n}"] == "simple"

        # AST should remain but DUT and tester are reset
        n = "post_reload"
        assert retn[f"ast_size_{n}"] == 3
        assert retn[f"reg_data_{n}"] == 0
        assert retn[f"timeset_{n}"] is None

    def test_invalid_target(self, capfd, falcon):
        retn = in_new_origen_proc(mod=target_proc_funcs, expect_fail=True)
        self.assert_pre_setup_retn(retn, "pre_setup")
        self.assert_retn(retn, "post_setup", [falcon.name, TargetCLI.utn], [], None, False, True)
        assert TargetCLI.unknown_err_msg in capfd.readouterr().out


class TestTargetOpts(PythonAppCommon):
    def test_target_can_be_set(self):
        targets = self.targets.show_per_cmd_targets(targets=self.targets.eagle)
        assert targets == [self.targets.eagle.name]

        targets = self.targets.show_per_cmd_targets(targets=[self.targets.hawk, self.targets.uflex])
        assert targets == [self.targets.hawk.name, self.targets.uflex.name]

    def test_no_target_can_be_used_per_command(self):
        targets = self.targets.show_per_cmd_targets()
        assert len(targets) != 0

        targets = self.targets.show_per_cmd_targets(targets=False)
        assert len(targets) == 0
