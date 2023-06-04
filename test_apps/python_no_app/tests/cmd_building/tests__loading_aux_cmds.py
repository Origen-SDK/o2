import pytest, os
from .shared import CLICommon
from tests.test_configs import Common as ConfigCommmon

class T_LoadingAuxCommands(CLICommon, ConfigCommmon):
    def test_aux_commands_are_added(self):
        help = self.global_cmds.aux_cmds.get_help_msg()
        help.assert_subcmds(*self.aux_cmd_ns_subcs)

        out = self.cmd_testers_cmd.run("python_no_app_tests")
        assert "Hi from No-App Origen!!" in out

    def test_aux_commands_stack(self, with_cli_aux_cmds):
        help = self.global_cmds.aux_cmds.get_help_msg()
        help.assert_subcmds(self.aux.ns.aux_cmds_from_cli_dir.base_cmd, *self.aux_cmd_ns_subcs)

        out = self.aux.ns.aux_cmds_from_cli_dir.base_cmd.cli_dir_says_hi.run()
        assert "Hi from CLI dir!! " in out

    @pytest.mark.xfail
    def test_aux_command_tree_view(self):
        # FEATURE CLI aux/plugin/app command tree view
        fail

    class TestNestedCommands(CLICommon):
        def test_first_level_nested_aux_commands(self):
            # Try first level
            help = self.cmd_testers_cmd.get_help_msg()
            subcs = list(self.cmd_testers_cmd.subcmds.values())
            subcs.insert(1, "help")
            help.assert_subcmds(*subcs)

            out = self.cmd_testers.subc_l1.run()
            assert "Hi from 'cmd_tester' level 1!" in out

        def test_second_level_nested_aux_commands(self):
            # Try second level
            help = self.cmd_testers.subc_l1.get_help_msg()
            help.assert_subcmds("help", self.cmd_testers.subc_l2)

            out = self.cmd_testers.subc_l2.run()
            assert "Hi from 'cmd_tester' level 2!" in out

        def test_third_level_nested_aux_commands(self):
            # Try third level
            help = self.cmd_testers.subc_l2.get_help_msg()
            help.assert_subcmds("help", self.cmd_testers.subc_l3_a, self.cmd_testers.subc_l3_b)

            out = self.cmd_testers.subc_l3_a.run()
            assert "Hi from 'cmd_tester' level 3 (A)!" in out

        def test_third_level_nested_aux_commands_modulized_path(self):
            out = self.cmd_testers.subc_l3_b.run()
            assert "Hi from 'cmd_tester' level 3 (B)!" in out

    # TODO need to consolidate with other similar tests
    class TestErrorCases(CLICommon, ConfigCommmon):
        def test_conflicting_namespaces(self):
            orig_config = self.aux_cmd_configs_dir.joinpath("add_aux_cmd_config.toml")
            conflicting_config = self.aux_cmd_configs_dir.joinpath("conflicting_namespaces_config.toml")
            out = self.global_cmds.aux_cmds.get_help_msg_str(with_configs=[conflicting_config, orig_config])
            help = self.HelpMsg(out)
            help.assert_subcmds(self.aux.ns.add_aux_cmd.base_cmd, *self.aux_cmd_ns_subcs)
            assert "Auxillary commands namespaced 'add_aux_cmd' already exists." in out
            assert f"Cannot add namespace from config '{conflicting_config}'" in out
            assert f"Namespace first defined in config '{orig_config}'" in out

        @pytest.mark.xfail
        def test_conflicting_command_names_within_namespace(self):
            fail

        @pytest.mark.xfail
        def test_conflicting_subcommand_names_within_namespace(self):
            fail

        @pytest.mark.xfail
        def test_same_name_commands_in_different_namespace(self):
            fail

        @pytest.mark.xfail
        def test_same_name_commands_within_namespace_but_different_level(self):
            fail

        def test_invalid_aux_path(self):
            ''' Should generate a logger error message but not kill the process'''
            out = self.global_cmds.aux_cmds.get_help_msg_str(with_configs=self.aux_cmd_configs_dir.joinpath("invalid_aux_cmd_path_config.toml"))
            help = self.HelpMsg(out)
            help.assert_subcmds(self.aux.ns.add_aux_cmd.base_cmd, *self.aux_cmd_ns_subcs)
            assert f"Unable to add auxillary commands at '{self.aux_cmd_configs_dir}{os.sep}./invalid_aux_cmd_path.toml' from config '{self.aux_cmd_configs_dir}{os.sep}invalid_aux_cmd_path_config.toml'. The following error was met" in out

        def test_missing_aux_cmd_impl_dir(self):
            out = self.cmd_testers.error_cases.gen_error("missing_impl_dir", "missing_impl_dir_subc", return_stdout=True)
            assert "Could not find implementation for aux command 'error_cases.missing_impl_dir.missing_impl_dir_subc'" in out
            assert f"From root '{self.cmd_testers_root}', searched:" in out
            assert "error_cases.missing_impl_dir.missing_impl_dir_subc.py" in out
            assert f"error_cases{os.sep}missing_impl_dir.missing_impl_dir_subc.py" in out

        def test_missing_aux_cmd_impl_file(self):
            out = self.cmd_testers.error_cases.gen_error("missing_impl_file", return_stdout=True)
            assert "Could not find implementation for aux command 'error_cases.missing_impl_file'" in out
            assert "error_cases.missing_impl_file.py" in out
            assert f"error_cases{os.sep}missing_impl_file.py" in out

        def test_missing_run_function(self):
            out = self.cmd_testers.error_cases.gen_error("test_missing_run_function", return_stdout=True)
            f = self.cmd_testers_root.joinpath('error_cases/test_missing_run_function.py')
            assert f"Could not find 'run' function in module '{f}'" in out

        def test_exception_from_run_function(self):
            out = self.cmd_testers.error_cases.gen_error("test_exception_in_run")
            assert "test_exception_in_run.py\", line 2" in out
            assert "RuntimeError: Raising run time error in 'test_exception_in_run'" in out

    @pytest.mark.xfail
    def test_no_help_str_given(self):
        fail

    @pytest.mark.xfail
    def test_override_help(self):
        fail
