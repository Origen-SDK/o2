import pytest, os
from .shared import CLICommon
from tests.test_configs import Common as ConfigCommmon
from origen.helpers.env import run_cli_cmd

class T_LoadingAuxCommands(CLICommon, ConfigCommmon):
    # cmdn_nested_l1 = "test_nested_level_1"
    # cmdn_nested_l2 = "test_nested_level_2"
    # cmdn_nested_l3a = "test_nested_level_3_a"
    # cmdn_nested_l3b = "test_nested_level_3_b"

    # nested_l1_cmd = CLICommon.Cmd(cmdn_nested_l1, CLICommon.cmd_base)
    # nested_l2_cmd = CLICommon.Cmd(cmdn_nested_l2, [*CLICommon.cmd_base, cmdn_nested_l1])
    # nested_l3a_cmd = CLICommon.Cmd(cmdn_nested_l3a, [*CLICommon.cmd_base, cmdn_nested_l1, cmdn_nested_l2])
    # nested_l3b_cmd = CLICommon.Cmd(cmdn_nested_l3b, [*CLICommon.cmd_base, cmdn_nested_l1, cmdn_nested_l2])

    def test_aux_commands_are_added(self):
        help = self.global_cmds.aux_cmds.get_help_msg() # run_cli_cmd(["auxillary_commands", "-h"])
        # help = self.parse_subcmd_help_dialogue(out)
        # assert list(help.subcmds.keys()) == ["cmd_testers", "help", "python_no_app_aux_cmds"]
        help.assert_subcmds(
            self.cmd_testers_cmd,
            'help',
            self.aux.ns.python_no_app_aux_cmds.base_cmd
        )

        # out = run_cli_cmd(["auxillary_commands", "cmd_testers", "python_no_app_tests"])
        out = self.cmd_testers_cmd.run("python_no_app_tests")
        assert "Hi from No-App Origen!!" in out

    def test_aux_commands_stack(self, with_cli_aux_cmds):
        # Copy origen config, commands toml, and commands to package root
        # shutil.copy(self.dummy_config, self.cli_config)
        # shutil.copy(, self.cli_config)
        # try:
        #     out = run_cli_cmd(["auxillary_commands", "-h"])
        #     print(out)
        #     # from pathlib import WindowsPath, PosixPath
        #     # configs = eval(out)
        #     # retn = in_new_origen_proc(mod=config_funcs)
        # finally:
        #     os.remove(self.cli_config)

        # out = run_cli_cmd(["auxillary_commands", "-h"])
        # help = self.parse_subcmd_help_dialogue(out)
        help = self.global_cmds.aux_cmds.get_help_msg()
        help.assert_subcmds(
            self.aux.ns.aux_cmds_from_cli_dir.base_cmd,
            self.cmd_testers_cmd,
            'help',
            self.aux.ns.python_no_app_aux_cmds.base_cmd
        )
        # assert list(help.subcmds.keys()) == ["aux_cmds_from_cli_dir", "cmd_testers", "help", "python_no_app_aux_cmds"]

        # out = self.global_cmds.aux_cmds.run("aux_cmds_from_cli_dir", "cli_dir_says_hi")
        out = self.aux.ns.aux_cmds_from_cli_dir.base_cmd.cli_dir_says_hi.run()
        assert "Hi from CLI dir!! " in out

        # TODO Add another toml in the environment
        # out = run_cli_cmd(["auxillary_commands", "-h"])
        # help = self.parse_subcmd_help_dialogue(out)
        # assert list(help["subcmds"].keys()) == ["aux_cmds_from_cli_dir", "cmd_testers", "help", "python_no_app_aux_cmds"]

    @pytest.mark.xfail
    def test_aux_command_tree_view(self):
        fail

    class TestNestedCommands(CLICommon):
        def test_first_level_nested_aux_commands(self):
            # Try first level
            help = self.cmd_testers_cmd.get_help_msg()
            # out = run_cli_cmd(["auxillary_commands", "cmd_testers", "-h"])
            # help = self.parse_subcmd_help_dialogue(out)
            # assert list(help.subcmds.keys()) == [
            #     "error_cases",
            #     "help",
            #     "python_no_app_tests",
            #     "test_arguments",
            #     "test_current_command",
            #     "test_nested_level_1"
            # ]
            subcs = list(self.cmd_testers_cmd.subcmds.values())
            subcs.insert(1, "help")
            help.assert_subcmds(*subcs)

            out = self.cmd_testers.subc_l1.run()
            # out = run_cli_cmd(["auxillary_commands", "cmd_testers", "test_nested_level_1"])
            assert "Hi from 'cmd_tester' level 1!" in out

        def test_second_level_nested_aux_commands(self):
            # Try second level
            # out = run_cli_cmd(["auxillary_commands", "cmd_testers", "test_nested_level_1", "-h"])
            # help = self.parse_subcmd_help_dialogue(out)
            help = self.cmd_testers.subc_l1.get_help_msg()
            help.assert_subcmds("help", self.cmd_testers.subc_l2)
            # assert list(help.subcmds.keys()) == ["help", "test_nested_level_2"]

            # out = run_cli_cmd(["auxillary_commands", "cmd_testers", "test_nested_level_1", "test_nested_level_2"])
            out = self.cmd_testers.subc_l2.run()
            assert "Hi from 'cmd_tester' level 2!" in out

        def test_third_level_nested_aux_commands(self):
            # Try third level
            # out = run_cli_cmd(["auxillary_commands", "cmd_testers", "test_nested_level_1", "test_nested_level_2", "-h"])
            # help = self.parse_subcmd_help_dialogue(out)
            help = self.cmd_testers.subc_l2.get_help_msg()
            help.assert_subcmds("help", self.cmd_testers.subc_l3_a, self.cmd_testers.subc_l3_b)
            # assert list(help.subcmds.keys()) == ["help", "test_nested_level_3_a", "test_nested_level_3_b"]

            # out = run_cli_cmd(["auxillary_commands", "cmd_testers", "test_nested_level_1", "test_nested_level_2", "test_nested_level_3_a"])
            out = self.cmd_testers.subc_l3_a.run()
            assert "Hi from 'cmd_tester' level 3 (A)!" in out

        def test_third_level_nested_aux_commands_modulized_path(self):
            # out = run_cli_cmd(["auxillary_commands", "cmd_testers", "test_nested_level_1", "test_nested_level_2", "test_nested_level_3_b"])
            out = self.cmd_testers.subc_l3_b.run()
            assert "Hi from 'cmd_tester' level 3 (B)!" in out

    # TODO need to consolidate with other similar tests
    class TestErrorCases(CLICommon, ConfigCommmon):
        def test_conflicting_namespaces(self):
            orig_config = self.aux_cmd_configs_dir.joinpath("add_aux_cmd_config.toml")
            conflicting_config = self.aux_cmd_configs_dir.joinpath("conflicting_namespaces_config.toml")
            out = self.global_cmds.aux_cmds.get_help_msg_str(with_configs=[conflicting_config, orig_config])
            # out = run_cli_cmd(
            #     ["auxillary_commands", "-h"],
            #     with_configs=[
            #         conflicting_config,
            #         orig_config,
            #     ]
            # )
            # help = self.parse_subcmd_help_dialogue(out)
            help = self.HelpMsg(out)
            # assert list(help.subcmds.keys()) == ["add_aux_cmd", "cmd_testers", "help", "python_no_app_aux_cmds"]
            help.assert_subcmds(
                self.aux.ns.add_aux_cmd.base_cmd,
                self.cmd_testers_cmd,
                "help",
                self.aux.ns.python_no_app_aux_cmds.base_cmd
            )
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
            # out = run_cli_cmd(["auxillary_commands", "-h"], with_configs=self.aux_cmd_configs_dir.joinpath("invalid_aux_cmd_path_config.toml"))
            out = self.global_cmds.aux_cmds.get_help_msg_str(with_configs=self.aux_cmd_configs_dir.joinpath("invalid_aux_cmd_path_config.toml"))
            help = self.HelpMsg(out)
            # assert list(help.subcmds.keys()) == ["add_aux_cmd", "cmd_testers", "help", "python_no_app_aux_cmds"]
            help.assert_subcmds(
                self.aux.ns.add_aux_cmd.base_cmd,
                self.cmd_testers_cmd,
                "help",
                self.aux.ns.python_no_app_aux_cmds.base_cmd
            )
            # assert help.subcmds.keys() == self.aux_base_cmds
            assert f"Unable to add auxillary commands at '{self.aux_cmd_configs_dir}{os.sep}./invalid_aux_cmd_path.toml' from config '{self.aux_cmd_configs_dir}{os.sep}invalid_aux_cmd_path_config.toml'. The following error was met" in out

        def test_missing_aux_cmd_impl_dir(self):
            # out = run_cli_cmd(["auxillary_commands", "cmd_testers", "error_cases", "missing_impl_dir", "missing_impl_dir_subc"], expect_fail=True)
            out = self.cmd_testers.error_cases.gen_error("missing_impl_dir", "missing_impl_dir_subc", return_stdout=True)
            f = self.cmd_testers_root.joinpath('error_cases/missing_impl_dir/missing_impl_dir_subc.py')
            f2 = self.cmd_testers_root.joinpath('error_cases.missing_impl_dir.missing_impl_dir_subc.py')
            # print(out)
            # print(f"Could not find implementation for auxillary command 'error_cases missing_impl_dir missing_impl_dir_subc' at '{f}' or '{f2}'")
            assert f"Could not find implementation for aux command 'error_cases missing_impl_dir missing_impl_dir_subc' at '{f}' or '{f2}'" in out

        def test_missing_aux_cmd_impl_file(self):
            # out = run_cli_cmd(["auxillary_commands", "cmd_testers", "error_cases", "missing_impl_file"], expect_fail=True)
            out = self.cmd_testers.error_cases.gen_error("missing_impl_file", return_stdout=True)
            f = self.cmd_testers_root.joinpath('error_cases/missing_impl_file.py')
            f2 = self.cmd_testers_root.joinpath('error_cases.missing_impl_file.py')
            assert f"Could not find implementation for aux command 'error_cases missing_impl_file' at '{f}' or '{f2}'" in out

        def test_missing_run_function(self):
            out = self.cmd_testers.error_cases.gen_error("test_missing_run_function", return_stdout=True)
            # out = run_cli_cmd(["auxillary_commands", "cmd_testers", "error_cases", "test_missing_run_function"], expect_fail=True)
            f = self.cmd_testers_root.joinpath('error_cases/test_missing_run_function.py')
            assert f"Could not find 'run' function in module '{f}'" in out

        def test_exception_from_run_function(self):
            out = self.cmd_testers.error_cases.gen_error("test_exception_in_run")
            # out = run_cli_cmd(["auxillary_commands", "cmd_testers", "error_cases", "test_exception_in_run"], expect_fail=True, return_details=True)
            # print(out)
            # err = out["stderr"]
            assert "test_exception_in_run.py\", line 2" in out
            assert "RuntimeError: Raising run time error in 'test_exception_in_run'" in out

    @pytest.mark.xfail
    def test_no_help_str_given(self):
        fail

    @pytest.mark.xfail
    def test_override_help(self):
        fail
