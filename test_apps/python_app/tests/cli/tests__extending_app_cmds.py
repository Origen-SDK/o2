import pytest
from .shared import CLICommon

class T_ExtendingAppCmds(CLICommon):
    class TestExtendingAppCmds(CLICommon):
        config = CLICommon.app_cmds.exts["app.arg_opt_warmup"]
        cmd = CLICommon.app_cmds.arg_opt_warmup.extend(
            config["exts"],
            from_configs=[config["cfg"]],
            with_env=config["env"],
        )

        config_shallow_subc = CLICommon.app_cmds.exts["app.nested_app_cmds.nested_l1"]
        shallow_subc = CLICommon.app_cmds.nested_cmds.nested_l1.extend(
            config_shallow_subc["exts"],
            from_configs=[config_shallow_subc["cfg"]],
            with_env=config_shallow_subc["env"],
        )

        config_deep_subc = CLICommon.app_cmds.exts["app.nested_app_cmds.nested_l1.nested_l2_b.nested_l3_a"]
        deep_subc = CLICommon.app_cmds.nested_cmds.nested_l1.nested_l2_b.nested_l3_a.extend(
            config_deep_subc["exts"],
            from_configs=[config_deep_subc["cfg"]],
            with_env=config_deep_subc["env"],
        )

        @pytest.fixture
        def first(self):
            return "1st"

        @pytest.fixture
        def second(self):
            return ["2nd", "second"]

        @pytest.fixture
        def sv(self):
            return "sv"

        @pytest.fixture
        def mv(self):
            return ["mv0", "mv1"]

        @pytest.fixture
        def pypl_sv(self):
            return "pypl_sv"

        @pytest.fixture
        def pypl_mv(self):
            return ["pypl_mv_0", "pypl_mv_1", "pypl_mv_2"]

        @pytest.fixture
        def tas_sv(self):
            return "tas_sv"

        @pytest.fixture
        def tas_mv(self):
            return ["tas_mv_0", "tas_mv_1", "tas_mv_2"]

        @pytest.fixture
        def ec_sv(self):
            return "ec_sv"

        @pytest.fixture
        def ec_mv(self):
            return ["ec_mv_0", "ec_mv_1"]

        def test_help_msg(self):
            cmd = self.cmd
            help = cmd.get_help_msg()
            help.assert_args(cmd.first, cmd.second)
            help.assert_opts(
                cmd.ec_multi_opt,
                cmd.ec_single_opt,
                cmd.flag_opt,
                "help",
                "m",
                cmd.multi_opt,
                "nt",
                cmd.pypl_multi_opt,
                cmd.pypl_single_opt,
                cmd.single_opt,
                "t",
                cmd.tas_multi_opt,
                cmd.tas_single_opt,
                "v", "vk",
            )
            help.assert_subcmds(None)

            assert help.aux_exts == ["python_app_exts"]
            assert help.pl_exts == ["python_plugin", "test_apps_shared_test_helpers"]
            assert help.app_exts == False

        def test_exts(self, first, second, sv, mv, pypl_sv, pypl_mv, tas_sv, tas_mv, ec_sv, ec_mv):
            cmd = self.cmd
            out = cmd.run(
                first, second[0], second[1],
                "--single_opt", sv,
                "--m_opt", mv[0], mv[1],
                "-f", "--hidden",

                "--pypl_single_opt", pypl_sv,
                "--PYPL", pypl_mv[0], "--pypl_multi_opt", pypl_mv[1], pypl_mv[2],
                "-p", "--pypl_h_opt", "-p", '--pypl_h_opt',

                "--tas_sv", tas_sv,
                "-a", tas_mv[0], "--TAS", tas_mv[1], "--tas_multi_opt", tas_mv[2],
                "--tas_hidden", "--tas_hidden",

                "--ec_single_opt", ec_sv,
                "--ec_multi_opt", ec_mv[0], "-e", ec_mv[1],
                "--ec_h_opt", "--ec_h_opt", "--ec_h_opt",
            )
            cmd.assert_args(
                out,
                (cmd.first, first),
                (cmd.second, second),
                (cmd.single_opt, sv),
                (cmd.multi_opt, mv),
                (cmd.flag_opt, 1),
                (cmd.hidden_flag_opt, 1),

                (cmd.pypl_single_opt, pypl_sv),
                (cmd.pypl_multi_opt, pypl_mv),
                (cmd.pypl_hidden, 4),

                (cmd.tas_single_opt, tas_sv),
                (cmd.tas_multi_opt, tas_mv),
                (cmd.tas_hidden, 2),

                (cmd.ec_single_opt, ec_sv),
                (cmd.ec_multi_opt, ec_mv),
                (cmd.ec_hidden, 3),
            )

        def test_shallow_subc_help_msg(self):
            cmd = self.shallow_subc
            help = cmd.get_help_msg()
            help.assert_opts(
                cmd.ec_flag_opt_shallow,
                cmd.ec_single_opt_shallow,
                cmd.tas_flag_opt_shallow,
                "help", "m", "nt",
                cmd.pypl_single_opt_shallow,
                cmd.pypl_flag_opt_shallow,
                "t",
                cmd.tas_multi_opt_shallow,
                "v", "vk",
            )
            help.assert_subcmds("help", cmd.nested_l2_a, cmd.nested_l2_b)

            assert help.aux_exts == ["python_app_exts"]
            assert help.pl_exts == ["python_plugin", "test_apps_shared_test_helpers"]
            assert help.app_exts == False

        def test_shallow_subc_exts(self, pypl_sv, tas_sv, tas_mv, ec_sv):
            cmd = self.shallow_subc
            out = cmd.run(
                "-p", pypl_sv,
                "--pypl_flag_opt_shallow",
                "--tas_shallow", tas_mv[0], tas_mv[1], "--tas_m_opt", tas_mv[2], tas_sv,
                "-f", "--tas_f",
                "--ec_single_opt_shallow", ec_sv,
                "--ec_f", "--ec_f", "--ec_f",
            )
            cmd.assert_args(
                out,
                (cmd.pypl_single_opt_shallow, pypl_sv),
                (cmd.pypl_flag_opt_shallow, 1),
                (cmd.tas_multi_opt_shallow, [*tas_mv, tas_sv]),
                (cmd.tas_flag_opt_shallow, 2),
                (cmd.ec_single_opt_shallow, ec_sv),
                (cmd.ec_flag_opt_shallow, 3),
            )

        def test_deep_subc_help_msg(self):
            cmd = self.deep_subc
            help = cmd.get_help_msg()
            help.assert_opts(
                cmd.ec_flag_opt_deep,
                cmd.ec_single_opt_deep,
                cmd.pypl_flag_opt_deep,
                "help", "m", "nt",
                cmd.pypl_single_opt_deep,
                "t",
                cmd.tas_flag_opt_deep,
                cmd.tas_multi_opt_deep,
                "v", "vk",
            )
            help.assert_subcmds(None)

            assert help.aux_exts == ["python_app_exts"]
            assert help.pl_exts == ["python_plugin", "test_apps_shared_test_helpers"]
            assert help.app_exts == False

        def test_deep_subc_exts(self, pypl_sv, tas_mv, ec_sv):
            cmd = self.deep_subc
            out = cmd.run(
                "-q", pypl_sv,
                "-f", "-f", "--py_f", "--py_f",
                "--tas_opt", tas_mv[0], "--tas_opt", tas_mv[1], "--tas_opt", tas_mv[2], 
                "--tas_flag_opt_deep",
                "--ec_opt", ec_sv,
                "--ec_df", "-c", "--ec_df",
            )
            cmd.assert_args(
                out,
                (cmd.pypl_single_opt_deep, pypl_sv),
                (cmd.pypl_flag_opt_deep, 4),
                (cmd.tas_multi_opt_deep, tas_mv),
                (cmd.tas_flag_opt_deep, 1),
                (cmd.ec_single_opt_deep, ec_sv),
                (cmd.ec_flag_opt_deep, 3),
            )

    class TestAppCmdExtConflicts(CLICommon):
        ''' Test what happens when extensions conflict with app commands '''
        config = CLICommon.app_cmds.conflict_exts["app.arg_opt_warmup"]
        cmd = CLICommon.app_cmds.arg_opt_warmup.extend(
            config["exts"],
            from_configs=[config["cfg"]],
            with_env=config["env"],
        )
        _exts = CLICommon.exts.partition_exts(config["exts"])

        @classmethod
        def setup_class(cls):
            cls.cmd_help = cls.cmd.get_help_msg()
            cls.cmd_conflicts = cls.cmd_help.logged_errors

        @classmethod
        def teardown_class(cls):
            delattr(cls, "cmd_help")
            delattr(cls, "cmd_conflicts")
        
        @pytest.fixture
        def exts(self):
            return self._exts

        def test_help_msg(self, exts):
            cmd = self.cmd
            help = self.cmd_help

            help.assert_args(cmd.first, cmd.second)
            help.assert_opts(
                exts.tas.conflicts_from_test_apps_shared,
                exts.ec.conflicts_from_ext_conflicts,
                exts.pypl.conflicts_from_python_plugin,
                exts.ec.flag_opt,
                exts.tas.flag_opt,
                cmd.flag_opt,
                exts.pypl.flag_opt,
                "h", "m",
                cmd.multi_opt,
                "nt",
                cmd.single_opt,
                "t", "v", "vk",
            )
            help.assert_subcmds(None)

            assert help.aux_exts == ["ext_conflicts"]
            assert help.pl_exts == ["python_plugin", "test_apps_shared_test_helpers"]
            assert help.app_exts == False

        def test_conflict_messages(self, exts):
            cmd = self.cmd
            print(self.cmd_help.text)
            conflicts = [
                ["inter_ext_lna_ln", exts.tas.conflicts_from_test_apps_shared, "TAS"],

                ["ln", "iln", exts.pypl.conflicts_from_python_plugin, cmd.single_opt, "single_opt"],
                ["sn", "sna", exts.pypl.conflicts_from_python_plugin, cmd.multi_opt, "m"],
                ["lna", "lna", exts.pypl.conflicts_from_python_plugin, cmd.multi_opt, "m_opt"],

                ["iln", "iln", exts.tas.flag_opt, exts.pypl.flag_opt],
                ["lna", "ln", exts.tas.conflicts_from_test_apps_shared, cmd.hidden_flag_opt, "hidden"],
                ["lna", "lna", exts.tas.conflicts_from_test_apps_shared, exts.pypl.conflicts_from_python_plugin, "python_plugin_conflicts"],
                ["sna", "sna", exts.tas.conflicts_from_test_apps_shared, cmd.multi_opt, "m"],
                ["sna", "sna", exts.tas.conflicts_from_test_apps_shared, exts.pypl.conflicts_from_python_plugin, "a"],

                ["iln", "iln", exts.ec.flag_opt, exts.pypl.flag_opt],
                ["lna", "lna", exts.ec.conflicts_from_ext_conflicts, exts.pypl.conflicts_from_python_plugin, "python_plugin_conflicts"],
                ["lna", "ln", exts.ec.conflicts_from_ext_conflicts, exts.tas.conflicts_from_test_apps_shared, "TAS"],
                ["lna", "ln", exts.ec.conflicts_from_ext_conflicts, cmd.hidden_flag_opt, "hidden"],
                ["sna", "sna", exts.ec.conflicts_from_ext_conflicts, exts.pypl.conflicts_from_python_plugin, "a"],
                ["sna", "sna", exts.ec.conflicts_from_ext_conflicts, exts.tas.conflicts_from_test_apps_shared, "b"],
            ]
            for c in reversed(conflicts):
                m = self.cmd_conflicts.pop()
                assert self.to_conflict_msg(self.cmd, c) in m
            assert len(self.cmd_conflicts) == 0
        
        def test_conflicts_resolve_correctly(self, exts):
            cmd = self.cmd
            out = cmd.run(
                "a1", "a2,a3",
                "--TAS", "-b",
                "--EX_Conflicts", "-c", "-c",
                "--python_plugin_conflicts", "-a", "-a", "--conflicts_from_python_plugin",
                "--ext_opt.aux.ext_conflicts.flag_opt",
                "--ext_opt.plugin.test_apps_shared_test_helpers.flag_opt", "--ext_opt.plugin.test_apps_shared_test_helpers.flag_opt",
                "--flag_opt", "--flag_opt", "--flag_opt",
                "-f", "-f", "-f", "-f",
                "--m_opt", "m0", "m1", "-m", "m2",
                "-s", "s0"
            )
            cmd.assert_args(
                out,
                (cmd.first, "a1"),
                (cmd.second, ["a2", "a3"]),
                (cmd.flag_opt, 4),
                (exts.ec.flag_opt, 1),
                (exts.tas.flag_opt, 2),
                (exts.pypl.flag_opt, 3),
                (exts.tas.conflicts_from_test_apps_shared, 2),
                (exts.ec.conflicts_from_ext_conflicts, 3),
                (exts.pypl.conflicts_from_python_plugin, 4),
                (cmd.multi_opt, ["m0", "m1", "m2"]),
                (cmd.single_opt, "s0"),
            )

        @pytest.mark.skip
        def test_subc_help_msg(self):
            fail
        
        @pytest.mark.skip
        def test_subc_conflict_msgs(self):
            fail

        @pytest.mark.skip
        def test_subc_exts(self):
            fail

        @pytest.mark.skip
        def test_app_cmd_ext_conflicts(self):
            ''' This is likely a mistake case but should resolve nonetheless '''
            fail