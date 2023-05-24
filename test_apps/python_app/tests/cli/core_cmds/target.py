import pytest, origen
from ..shared import CLICommon
from tests.shared import Targets
from tests.proc_funcs import target_proc_funcs
from origen.helpers.env import in_new_origen_proc

class TargetCLI(CLICommon, Targets.TargetFixtures):
    target_cmd = CLICommon.in_app_commands.target

    @classmethod
    def clear_targets_cli(cls):
        return cls.target_cmd.run("clear")

    @classmethod
    def check_targets(cls, *targets, func_kwargs=None):
        retn = in_new_origen_proc(func=target_proc_funcs.show_target_setup, func_kwargs=func_kwargs)
        if len(targets) == 1 and targets[0] is None:
            targets = []
        assert retn['targets'] == [t.fp for t in targets]

    @pytest.fixture
    def set_eagle(self, eagle):
        # Add an initial target
        out = self.target_cmd.set.run(eagle.name)
        self.assert_out(out, eagle)
        self.check_targets(eagle)

    @classmethod
    def assert_out(cls, out, *targets, full_paths=False):
        t = out.split("The targets currently enabled are:\n")[1].split("\n")
        t = t[:t.index('')]
        if len(targets) == 1 and targets[0] is None:
            targets = []
        else:
            if full_paths:
                targets = [t.fp for t in targets]
            else:
                targets = [t.rp for t in targets]
        assert targets == t

    # Unknown target name
    utn = "unknown"
    unknown_err_msg = ("\n").join([
        f"No matching target '{utn}' found, here are the available targets:",
        *[f"    {t}" for t in CLICommon.targets.all_rp],
    ])

    no_set_or_default_targets_msg = "No targets have been enabled and this workspace does not enable any default targets"
    all_targets_removed_msg = "All targets were removed. Resetting to the default target."

    empty_default_env = {"origen_app_config_paths": str(CLICommon.to_config_path("target/empty_default_targets.toml"))}
    empty_default_run_opts = {"with_env": empty_default_env}
    bad_default_run_opts = {"with_env": {"origen_app_config_paths": str(CLICommon.to_config_path("target/bad_default_targets.toml"))}}

    @classmethod
    def duplicate_err_msg(cls, t):
        return f"Target '{t.name}' appears multiple times in the TARGETS list ({t.fp})"

    @classmethod
    def rm_non_active_target_err_msg(cls, t):
        return f"Tried to remove non-activated target '{t.name}' ({t.fp})"

    @pytest.fixture
    def clear_targets(self):
        self.clear_targets_cli()

class T_Target(TargetCLI):
    cmd = TargetCLI.target_cmd

    @classmethod
    def setup_class(cls):
        cls.clear_targets_cli()

    @pytest.mark.parametrize("cmd", [cmd, *cmd.subcmds.values()], ids=[cmd.name, *cmd.subcmds.keys()])
    def test_help_msg(self, cmd):
        help = cmd.get_help_msg()
        help.assert_summary(cmd.help)
        help.assert_args(cmd.args)
        if not cmd.name == "clear":
            help.assert_opts(cmd.full_paths, "h", "v", "vk")
        else:
            help.assert_bare_opts()
        if cmd.name == "target":
            help.assert_subcmds(cmd.subcmds, help=3)
        else:
            help.assert_subcmds(None)
        help.assert_not_extendable()

    def test_adding_targets(self, clear_targets, eagle, uflex, smt7, smt8):
        add = self.cmd.add
        out = add.run(eagle.name)
        self.assert_out(out, eagle)
        self.check_targets(eagle)

        out = add.run(uflex.name, add.full_paths)
        self.assert_out(out, eagle, uflex, full_paths=True)
        self.check_targets(eagle, uflex)

        out = add.run(smt7.name, smt8.name)
        self.assert_out(out, eagle, uflex, smt7, smt8)
        self.check_targets(eagle, uflex, smt7, smt8)

    def test_error_adding_unknown_targets(self, set_eagle, eagle, j750, uflex):
        add = self.cmd.add

        r = add.gen_error("unknown", return_full=True)
        assert r["returncode"] == 1
        assert r["stderr"] == ''
        assert self.unknown_err_msg in r["stdout"]
        
        # Targets should remain the same
        self.check_targets(eagle)

        # Try with some valid, some unknown
        r = add.gen_error(j750.name, "unknown", uflex.name, return_full=True)
        assert r["returncode"] == 1
        assert r["stderr"] == ''
        assert self.unknown_err_msg in r["stdout"]

        # Targets should remain the same
        self.check_targets(eagle)

    def test_adding_duplicate_target(self, set_eagle, eagle, j750, falcon):
        r = self.cmd.add.gen_error(j750.name, falcon.name, j750.name, return_full=True)
        self.check_targets(eagle)
        assert r["returncode"] == 1
        assert r["stderr"] == ''
        assert self.duplicate_err_msg(j750) in r["stdout"]

    def test_adding_already_added_targets(self, set_eagle, eagle, j750):
        add = self.cmd.add
        self.check_targets(eagle)

        out = add.run(eagle.name)
        self.assert_out(out, eagle)
        self.check_targets(eagle)

        out = add.run(j750.name)
        self.assert_out(out, eagle, j750)
        self.check_targets(eagle, j750)

        # Re-adding target shifts its position
        out = add.run(eagle.name)
        self.assert_out(out, j750, eagle)
        self.check_targets(j750, eagle)

    def test_setting_targets(self, clear_targets, eagle, falcon, j750, smt7, hawk):
        set = self.cmd.set
        out = set.run(eagle.name)
        self.assert_out(out, eagle)
        self.check_targets(eagle)

        out = set.run(falcon.name, j750.name, smt7.name)
        self.assert_out(out, falcon, j750, smt7)
        self.check_targets(falcon, j750, smt7)

        out = set.run(hawk.name, set.full_paths)
        self.assert_out(out, hawk, full_paths=True)
        self.check_targets(hawk)

    def test_error_setting_unknown_targets(self, set_eagle, eagle, j750, uflex):
        set = self.cmd.set
        r = set.gen_error("unknown", return_full=True)
        assert r["returncode"] == 1
        assert r["stderr"] == ''
        assert self.unknown_err_msg in r["stdout"]
        
        # Targets should remain the same
        self.check_targets(eagle)

        # Try with some valid, some unknown
        r = set.gen_error(j750.name, "unknown", uflex.name, return_full=True)
        assert r["returncode"] == 1
        assert r["stderr"] == ''
        assert self.unknown_err_msg in r["stdout"]

        # Targets should remain the same
        self.check_targets(eagle)

    def test_setting_duplicate_targets(self, set_eagle, eagle, j750, falcon):
        r = self.cmd.set.gen_error(j750.name, falcon.name, j750.name, return_full=True)
        self.check_targets(eagle)
        assert r["returncode"] == 1
        assert r["stderr"] == ''
        assert self.duplicate_err_msg(j750) in r["stdout"]

    def test_removing_targets(self, eagle, uflex, j750, smt7, smt8):
        self.cmd.set.run(eagle.name, uflex.name, j750.name, smt7.name, smt8.name)

        rm = self.cmd.remove
        out = rm.run(eagle.name, smt7.name)
        self.assert_out(out, uflex, j750, smt8)
        self.check_targets(uflex, j750, smt8)

        out = rm.run(smt8.name, rm.full_paths)
        self.assert_out(out, uflex, j750, full_paths=True)
        self.check_targets(uflex, j750)

    def test_error_removing_unset_or_unknown_targets(self, set_eagle, eagle, j750):
        rm = self.cmd.remove

        r = rm.gen_error(j750.name, return_full=True)
        self.check_targets(eagle)
        assert r["returncode"] == 1
        assert r["stderr"] == ''
        assert self.rm_non_active_target_err_msg(j750) in r["stdout"]

        r = rm.gen_error(eagle.name, j750.name, return_full=True)
        self.check_targets(eagle)
        assert r["returncode"] == 1
        assert r["stderr"] == ''
        assert self.rm_non_active_target_err_msg(j750) in r["stdout"]

    def test_removing_duplicate_targets(self, set_eagle, eagle):
        r = self.cmd.remove.gen_error(eagle.name, eagle.name, return_full=True)
        self.check_targets(eagle)
        assert r["returncode"] == 1
        assert r["stderr"] == ''
        assert self.duplicate_err_msg(eagle) in r["stdout"]

    def test_removing_all_targets_restores_default(self, set_eagle, eagle, falcon):
        out = self.cmd.remove.run(eagle.name)
        assert self.all_targets_removed_msg in out
        self.assert_out(out, falcon)
        self.check_targets(falcon)

    def test_restoring_default_targets(self, set_eagle, falcon):
        out = self.cmd.default.run()
        self.assert_out(out, falcon)
        self.check_targets(falcon)

    def test_restoring_empty_default_targets(self, set_eagle):
        out = self.cmd.default.run(with_env=self.empty_default_env)
        self.assert_out(out, None)
        self.check_targets(None, func_kwargs=self.empty_default_run_opts)

    def test_clearing_all_targets(self):
        out = self.cmd.clear.run()
        self.assert_out(out, None)
        self.check_targets(None)

    def test_viewing_targets(self, set_eagle, eagle, j750):
        v = self.cmd.view
        out = v.run()
        self.assert_out(out, eagle)
        self.check_targets(eagle)

        self.cmd.add.run(j750.name)
        out = v.run(v.full_paths)
        self.assert_out(out, eagle, j750, full_paths=True)
        self.check_targets(eagle, j750)

        self.cmd.clear.run()
        out = v.run(v.full_paths)
        self.assert_out(out, None, full_paths=True)
        self.check_targets(None)

    def test_delimited_targets(self, eagle, j750, uflex, smt7, smt8):
        out= self.cmd.set.run(','.join([eagle.name, uflex.name, j750.name]), smt7.name, smt8.name)
        targets = [eagle, uflex, j750, smt7, smt8]
        self.assert_out(out, *targets)
        self.check_targets(*targets)

        out = self.cmd.remove.run(','.join([uflex.name, j750.name]), ','.join([smt7.name, smt8.name]))
        self.assert_out(out, eagle)
        self.check_targets(eagle)

        out = self.cmd.add.run(','.join([smt7.name, smt8.name]), ','.join([uflex.name, j750.name]))
        targets = [eagle, smt7, smt8, uflex, j750]
        self.assert_out(out, *targets)
        self.check_targets(*targets)

    def test_base_cmd_acts_as_view(self, set_eagle, eagle, j750):
        h = "Run with 'help' or '-h' to see available subcommands"
        b = self.cmd

        out = b.run()
        self.assert_out(out, eagle)
        self.check_targets(eagle)
        assert h in out

        self.cmd.add.run(j750.name)
        out = b.run(b.full_paths)
        self.assert_out(out, eagle, j750, full_paths=True)
        self.check_targets(eagle, j750)
        assert h in out

        self.cmd.clear.run()
        out = b.run(b.full_paths)
        self.assert_out(out, None, full_paths=True)
        self.check_targets(None)
        assert h in out

    def test_no_default_set(self, eagle):
        e = self.bypass_origen_app_lookup_env

        out = self.cmd.view.run(with_env=e)
        assert self.no_set_or_default_targets_msg in out

        self.cmd.set.run(eagle.name)
        out = self.cmd.view.run()
        self.assert_out(out, eagle)

        out = self.cmd.default.run(with_env=e)
        assert self.no_set_or_default_targets_msg in out
        self.check_targets(None, func_kwargs={"with_env": e})

    def test_invalid_default_target(self, clear_targets, eagle, capfd):
        r = self.cmd.default.gen_error(run_opts=self.bad_default_run_opts, return_full=True)
        assert r["returncode"] == 1
        assert r["stderr"] == ''
        assert self.unknown_err_msg in r["stdout"]
        assert self.unknown_err_msg not in capfd.readouterr().out
        in_new_origen_proc(func=target_proc_funcs.show_target_setup, func_kwargs=self.bad_default_run_opts, expect_fail=True)
        assert self.unknown_err_msg in capfd.readouterr().out

        self.cmd.set.run(eagle.name)
        r = self.cmd.default.gen_error(run_opts=self.bad_default_run_opts, return_full=True)
        assert r["returncode"] == 1
        assert r["stderr"] == ''
        assert self.unknown_err_msg in r["stdout"]

        out = self.cmd.view.run()
        self.assert_out(out, eagle)
        self.check_targets(eagle)
