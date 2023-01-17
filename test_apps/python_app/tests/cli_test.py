# FOR_PR convert assert args
import pytest, pathlib
import subprocess
import os, re
import origen
from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg

Cmd = CLIShared.Cmd

origen_cli = os.getenv('TRAVIS_ORIGEN_CLI') or 'origen'

def test_origen_v():
    #import pdb; pdb.set_trace()
    process = subprocess.Popen([f'{origen_cli}', '-v'],
                               stdout=subprocess.PIPE,
                               universal_newlines=True)
    # wait for the process to finish and read the result, 0 is success
    assert process.wait() == 0
    # Process is done
    # Read std out
    first_stdout_line = process.stdout.readline()
    assert "App:" in first_stdout_line
    second_stdout_line = process.stdout.readline()
    assert "Origen" in second_stdout_line
    assert " 2." in second_stdout_line

def test_bad_command():
    process = subprocess.Popen([f'{origen_cli}', 'thisisnotacommand'],
                               stderr=subprocess.PIPE,
                               universal_newlines=True)
    assert process.wait() == 2
    assert "error:" in process.stderr.readline()


def test_origen_g():
    os.chdir(origen.root)
    process = subprocess.Popen([
        f'{origen_cli}', 'g', r'./example/patterns/toggle.py', '-t',
        r'./targets/eagle_with_smt7.py'
    ],
                               universal_newlines=True)
    assert process.wait() == 0

class TestBadConfigs:
    @property
    def bad_config_env(self):
        return {
            **os.environ,
            "origen_bypass_config_lookup": "1",
            "origen_config_paths": str(
                pathlib.Path(__file__).parent.joinpath("origen_utilities/configs/ldap/test_bad_ldap_config.toml").absolute()
            )
        }

    def test_origen_v(self):
        r = subprocess.run(
            [origen_cli, '-v'],
            capture_output=True,
            env=self.bad_config_env
        )
        assert r.returncode == 1
        out = r.stdout.decode("utf-8").strip()
        err = r.stderr.decode("utf-8").strip()
        p = pathlib.Path("tests/origen_utilities/configs/ldap/test_bad_ldap_config.toml")
        # TODO should this kill the process?
        # assert "Couldn't boot app to determine the in-application Origen version" in out
        assert f"invalid type: string \"hi\", expected an integer for key `ldaps.bad.timeout` in {str(p)}" in out
        assert err == ""

    def test_origen_cmd(self):
        r = subprocess.run(
            [origen_cli, 'g', r'./example/patterns/toggle.py', '-t', r'./targets/eagle_with_smt7.py'],
            capture_output=True,
            env=self.bad_config_env
        )
        assert r.returncode == 1
        out = r.stdout.decode("utf-8").strip()
        err = r.stderr.decode("utf-8").strip()
        p = pathlib.Path("tests/origen_utilities/configs/ldap/test_bad_ldap_config.toml")
        assert f"invalid type: string \"hi\", expected an integer for key `ldaps.bad.timeout` in {str(p)}" in out
        assert err == ""

    def test_bad_config_path(self):
        r = subprocess.run(
            [origen_cli, '-v'],
            capture_output=True,
            env={
                **self.bad_config_env,
                **{
                    "origen_config_paths": str(pathlib.Path(__file__).parent.joinpath("missing.toml").absolute())
                }
            }
        )
        assert r.returncode == 1
        out = r.stdout.decode("utf-8").strip()
        err = r.stderr.decode("utf-8").strip()
        # TODO should this kill the process?
        # assert "Couldn't boot app to determine the in-application Origen version" in out
        assert "missing.toml either does not exists or is not accessible" in out
        assert err == ""

class TestAppWorkspaceCoreCommands(CLIShared):
    @property
    def cmd_shortcuts__app(self):
        return {
            'arg_opt_warmup': 'arg_opt_warmup',
            "examples": "examples",
            "playground": "playground",
            "y": "playground",
            "nested_app_cmds": "nested_app_cmds",
            "disabling_app_opts": "disabling_app_opts",
        }

    def test_app_workspace_help_message(self):
        help = self.in_app_cmds.origen.get_help_msg()
        assert help.root_cmd is True
        assert "Origen CLI: 2." in help.version_str

        assert len(help.opts) == 3
        help.assert_help_opt_at(0)
        help.assert_vk_opt_at(1)
        help.assert_v_opt_at(2)

        assert set(help.subcmd_names) == set(self.in_app_cmds.all_names_add_help)
        print(help.app_cmd_shortcuts)
        assert help.app_cmd_shortcuts == self.cmd_shortcuts__app
        # TODO plugin commands
        # assert help.pl_cmd_shortcuts == self.cmd_shortcuts__default_plugins
        # TODO Aux commands
        # assert help.pl_cmd_shortcuts == {
        #     "plugin_says_hi": ("python_plugin", "plugin_says_hi"),
        #     "echo": ("python_plugin", "echo"),
        # }

    @pytest.mark.parametrize("cmd", CLIShared.in_app_cmds.cmds, ids=CLIShared.in_app_cmds.all_names)
    def test_core_commands_are_available(self, cmd):
        ''' Just testing that "-h" doesn't crash for all core commands '''
        help = cmd.get_help_msg()
        assert len(help.opts) >= 3

class TestAppCommandBuilding(CLIShared):
    warmup_cmd = CLIShared.app_sub_cmd(
        "arg_opt_warmup",
        help = "Gross test command demonstrating args/opts from app commands",
        args=[
            CmdArg("first", help="First Argument - Required", required=True),
            CmdArg("second", help="Second Multi-Argument - Not Required", use_delimiter=True, multi=True),
        ],
        opts=[
            CmdOpt("flag_opt", sn="f", help="Flag opt"),
            CmdOpt("single_opt", sn_aliases=["s"], takes_value=True, help="Single-value non-required opt"),
            CmdOpt("multi_opt", sn_aliases=["m"], ln_aliases=["m_opt"], multi=True, help="Multi-value non-required opt"),
            CmdOpt("hidden_flag_opt", hidden=True, ln="hidden", help="Hidden flag opt"),
        ]
    )
    nested_cmds = CLIShared.app_sub_cmd(
        "nested_app_cmds",
        help="Nested app cmds",
        subcmds=[
            Cmd(
                "nested_l1",
                help="Nested app cmds level 1",
                subcmds=[
                    Cmd(
                        "nested_l2_a",
                        help="Nested app cmds level 2 (A)",
                        subcmds=[
                            Cmd(
                                "nested_l3_a",
                                help="Nested app cmds level 3 (A-A)"
                            ),
                            Cmd(
                                "nested_l3_b",
                                help="Nested app cmds level 3 (A-B)"
                            ),
                        ]
                    ),
                    Cmd(
                        "nested_l2_b",
                        help="Nested app cmds level 2 (B)",
                        subcmds=[
                            Cmd(
                                "nested_l3_a",
                                help="Nested app cmds level 3 (B-A)"
                            ),
                            Cmd(
                                "nested_l3_b",
                                help="Nested app cmds level 3 (B-B)"
                            ),
                        ]
                    )
                ]
            )
        ]
    )

    def test_app_command_args_and_opts(self):
        cmd = self.warmup_cmd
        help = cmd.get_help_msg()
        help.assert_num_args(cmd.num_args)
        help.assert_num_opts(cmd.num_opts)
        help.assert_arg_at(0, cmd.first)
        help.assert_arg_at(1, cmd.second)

        help.assert_opt_at(0, cmd.flag_opt)
        help.assert_help_opt_at(1)
        help.assert_vk_opt_at(2)
        help.assert_opt_at(3, cmd.multi_opt)
        help.assert_opt_at(4, cmd.single_opt)
        help.assert_v_opt_at(5)

        rv = "req_arg"
        m0 = "m0"
        m12 = "m1,m2"
        mo0 = "mo0"
        mo12 = "m01,m02"
        mo4 = "mo4"
        sv = "s_opt"

        out = cmd.run(
            rv,
            m0, m12,
            "-f",
            "-s", sv,
            "-m", mo0, mo12,
            "--m_opt", mo4,
            "--hidden"
        )
        assert cmd.first.to_assert_str(rv) in out
        assert cmd.second.to_assert_str([m0, m12]) in out
        assert cmd.flag_opt.to_assert_str(1) in out
        assert cmd.single_opt.to_assert_str(sv) in out
        assert cmd.multi_opt.to_assert_str([mo0, mo12, mo4]) in out
        assert cmd.hidden_flag_opt.to_assert_str(1) in out
        assert cmd.parse_arg_keys(out) == [
            cmd.first.name,
            cmd.second.name,
            cmd.flag_opt.name,
            cmd.single_opt.name,
            cmd.multi_opt.name,
            cmd.hidden_flag_opt.name,
        ]

        out = cmd.run(
            rv,
            "-f", "-f",
            "-m", mo0, mo12,
        )
        assert cmd.first.to_assert_str(rv) in out
        assert cmd.flag_opt.to_assert_str(2) in out
        assert cmd.multi_opt.to_assert_str([mo0, mo12]) in out
        assert cmd.parse_arg_keys(out) == [
            cmd.first.name,
            cmd.flag_opt.name,
            cmd.multi_opt.name,
        ]

        out = cmd.gen_error()
        assert self.err_msgs.missing_required_arg(cmd.first) in out

    nested_cmd_testcases = [
        (nested_cmds, 0, None),
        (nested_cmds.nested_l1, 1, None),
        (nested_cmds.nested_l1.nested_l2_a, 2, 'A'),
        (nested_cmds.nested_l1.nested_l2_a.nested_l3_a, 3, 'A-A'),
        (nested_cmds.nested_l1.nested_l2_a.nested_l3_b, 3, 'A-B'),
        (nested_cmds.nested_l1.nested_l2_b, 2, 'B'),
        (nested_cmds.nested_l1.nested_l2_b.nested_l3_a, 3, 'B-A'),
        (nested_cmds.nested_l1.nested_l2_b.nested_l3_b, 3, 'B-B'),
    ]
    nested_cmd_ids = [f"{cmd[0].name} {'Base' if cmd[2] is None else cmd[2]}" for cmd in nested_cmd_testcases]
    @pytest.mark.parametrize("cmd,lvl,sublvl", nested_cmd_testcases, ids=nested_cmd_ids)
    def test_nested_cmds(self, cmd, lvl, sublvl):
        help = cmd.get_help_msg()
        subcs = list(cmd.subcmds.values())
        if len(subcs) == 0:
            help.assert_subcmds(None)
        else:
            subcs.insert(0, "help")
            help.assert_subcmds(*subcs)

        out = cmd.run()
        assert f"Hi from 'nested_app_cmds' level {lvl}{ ' (' + sublvl + ')' if sublvl else ''}!" in out

class TestErrorCases(CLIShared):
    @pytest.mark.skip
    def test_invalid_cmd_toml(self):
        # FOR_PR need to make app specific
        out = self.in_app_cmds.origen.run(with_env={"ORIGEN_APP_COMMANDS": "test_case_cmds/invalid.toml,test_case_cmds/error_cases.toml"}) # run_cli_cmd(["-h"]) #.split("\n\n")
        print(out)
        help = self.HelpMsg(out)
        assert list(help.subcmds.keys()) == ["add_aux_cmd", "cmd_testers", "help", "python_no_app_aux_cmds"]
        assert f"Unable to add auxillary commands at '{self.aux_cmd_configs_dir}{os.sep}./invalid_aux_cmd_path.toml' from config '{self.aux_cmd_configs_dir}{os.sep}invalid_aux_cmd_path_config.toml'. The following error was met" in out

    @pytest.mark.skip
    def test_error_global_and_in_app_setting_used(self):
        fail

@pytest.mark.skip
class TestPluginCommandsAreAdded:
    ...

@pytest.mark.skip
class TestAuxCommandsAreAdded:
    ...

@pytest.mark.skip
class TestAppPluginAndAuxCommandClashing:
    def test_app_cmd_overrides_pl_cmd(self):
        fail

    def test_app_cmd_overrides_aux_cmd(self):
        fail

    # def test_extending_app_cmds(self):
    #     fail

from origen.helpers.regressions.cli.command import SrcTypes #, CmdExtOpt
from test_apps_shared_test_helpers.cli import CmdExtOpt, Cmd

class TestExtendingFromAppCommands(CLIShared):
    extend_core_cmds_env = {"ORIGEN_APP_EXTEND_CORE_CMDS": "1"}
    extend_pl_test_ext_stacking = {"ORIGEN_APP_EXT_PL_TEST_EXT_STACKING": "1"}
    exts = CmdExtOpt.from_src(
        "example",
        SrcTypes.APP,
        CmdExtOpt(
            "generic_app_ext_action",
            help="Action from the app",
            multi=True,
        ),
        CmdExtOpt(
            "generic_app_ext_flag",
            help="Flag ext from the app",
        ),
    )
    generic_app_ext_action = exts[0]
    generic_app_ext_flag = exts[1]
    core_cmd = origen.helpers.regressions.cli.CLI.in_app_cmds.eval.extend(
        exts,
        with_env=extend_core_cmds_env,
        from_configs=CLIShared.configs.suppress_plugin_collecting_config,
    )
    stacked_core_cmd = origen.helpers.regressions.cli.CLI.in_app_cmds.eval.extend(
        [
            *CLIShared.exts.exts["generic_core_ext"]["exts"],
            *exts
        ],
        from_configs=[CLIShared.exts.core_cmd_exts_cfg],
        with_env=extend_core_cmds_env,
    )
    py_pl_cmd = CLIShared.python_plugin.plugin_test_ext_stacking.extend(
        [
            *CLIShared.exts.exts["plugin.python_plugin.plugin_test_ext_stacking"]["exts"][1:3],
            *CLIShared.exts.test_apps_shared_generic_exts,
            *exts
        ],
        from_configs=[CLIShared.exts.pl_ext_stacking_from_aux_cfg],
        with_env=extend_pl_test_ext_stacking,
    )
    py_pl_subc = CLIShared.python_plugin.plugin_test_ext_stacking.subc.extend(
        [
            *CLIShared.exts.exts["plugin.python_plugin.plugin_test_ext_stacking.subc"]["exts"][1:3],
            *CLIShared.exts.test_apps_shared_generic_exts,
            *exts
        ],
        from_configs=[CLIShared.exts.pl_ext_stacking_from_aux_cfg],
        with_env=extend_pl_test_ext_stacking,
    )
    aux_cmd = CLIShared.aux.ns.dummy_cmds.dummy_cmd.extend(
        [
            *CLIShared.exts.exts["aux.dummy_cmds.dummy_cmd"]["exts"][2:6],
            *exts
        ],
        from_configs=[CLIShared.exts.pl_ext_stacking_from_aux_cfg],
        with_env=CLIShared.exts.exts["aux.dummy_cmds.dummy_cmd"]["env"]
    )
    aux_subc = CLIShared.aux.ns.dummy_cmds.dummy_cmd.subc.extend(
        [
            *CLIShared.exts.exts["aux.dummy_cmds.dummy_cmd.subc"]["exts"][2:6],
            *exts
        ],
        from_configs=[CLIShared.exts.pl_ext_stacking_from_aux_cfg],
        with_env=CLIShared.exts.exts["aux.dummy_cmds.dummy_cmd"]["env"]
    )
    na = "no_action"

    missing_ext_impl_cmd = CLIShared.python_plugin.plugin_test_args.extend(
        CmdExtOpt.from_src(
            "example",
            SrcTypes.APP,
            CmdExtOpt(
                "app_ext_missing_impl",
                help="App extension missing the implementation",
            ),
        ),
        with_env={"ORIGEN_APP_PL_CMD_MISSING_EXT_IMPL": "1"}
    )

    def test_extending_global_cmds(self):
        cmd = self.core_cmd
        help = cmd.get_help_msg()
        help.assert_args(cmd.code)
        help.assert_opts(
            cmd.generic_app_ext_action,
            cmd.generic_app_ext_flag,
            *self.in_app_cmds.standard_opts()
        )

        assert help.aux_exts == None
        assert help.pl_exts == None
        assert help.app_exts == True

        d = cmd.demos["minimal"]
        out = d.run()
        cmd.generic_app_ext_flag.assert_present(None, out)
        d.assert_present(out)

        out = d.run(add_args=[cmd.generic_app_ext_action.ln_to_cli(), self.na, cmd.generic_app_ext_flag.ln_to_cli()])
        cmd.generic_app_ext_action.assert_present([self.na], out)
        cmd.generic_app_ext_flag.assert_present(1, out)
        d.assert_present(out)

    def test_stacking_pl_aux_and_app_ext(self):
        cmd = self.stacked_core_cmd
        help = cmd.get_help_msg()
        help.assert_args(cmd.code)
        help.assert_opts(
            cmd.core_cmd_exts_generic_core_ext,
            cmd.generic_app_ext_action,
            cmd.generic_app_ext_flag,
            "help", "vk", "mode", "no_targets",
            cmd.pl_ext_cmds_generic_ext,
            "targets", "v"
        )
        assert help.aux_exts == ['core_cmd_exts']
        assert help.pl_exts == ['pl_ext_cmds']
        assert help.app_exts == True

        d = cmd.demos["minimal"]
        out = d.run(add_args=[
            cmd.generic_app_ext_action.ln_to_cli(), self.na,
            cmd.generic_app_ext_flag.ln_to_cli(),
            cmd.pl_ext_cmds_generic_ext.ln_to_cli(),
            cmd.core_cmd_exts_generic_core_ext.ln_to_cli(),
        ])
        cmd.generic_app_ext_action.assert_present([self.na], out)
        cmd.generic_app_ext_flag.assert_present(1, out)
        cmd.pl_ext_cmds_generic_ext.assert_present(1, out)
        cmd.core_cmd_exts_generic_core_ext.assert_present(1, out)
        d.assert_present(out)

    def test_extending_pl_cmd(self):
        cmd = self.py_pl_cmd
        help = cmd.get_help_msg()
        help.assert_args(cmd.single_arg)
        help.assert_opts(
            cmd.flag_opt,
            cmd.generic_app_ext_action,
            cmd.generic_app_ext_flag,
            "help", "vk",
            cmd.pl_ext_stacking_from_aux_action,
            cmd.pl_ext_stacking_from_aux_flag,
            cmd.test_apps_shared_ext_action,
            cmd.test_apps_shared_ext_flag,
            "v",
        )
        help.assert_subcmds("help", cmd.subc)

        assert help.aux_exts == ['pl_ext_stacking_from_aux']
        assert help.pl_exts == ['test_apps_shared_test_helpers']
        assert help.app_exts == True

        out = cmd.run()
        cmd.assert_args(
            out,
            (cmd.single_arg, None),
            (cmd.pl_ext_stacking_from_aux_action, None),
            (cmd.test_apps_shared_ext_action, None),
            (cmd.generic_app_ext_flag, None),
        )

        sa_v = "single_arg_val"
        out = cmd.run(sa_v, cmd.generic_app_ext_action.ln_to_cli(), self.na, cmd.generic_app_ext_flag.ln_to_cli())
        cmd.assert_args(
            out,
            (cmd.single_arg, sa_v),
            (cmd.pl_ext_stacking_from_aux_action, None),
            (cmd.test_apps_shared_ext_action, None),
            (cmd.generic_app_ext_action, [self.na]),
            (cmd.generic_app_ext_flag, 1),
        )

    def test_extending_pl_subc(self):
        cmd = self.py_pl_subc
        help = cmd.get_help_msg()
        help.assert_args(cmd.single_arg)
        help.assert_opts(
            cmd.flag_opt,
            cmd.generic_app_ext_action,
            cmd.generic_app_ext_flag,
            "help", "vk",
            cmd.pl_ext_stacking_from_aux_action_subc,
            cmd.pl_ext_stacking_from_aux_flag_subc,
            cmd.test_apps_shared_ext_action,
            cmd.test_apps_shared_ext_flag,
            "v",
        )
        help.assert_subcmds(None)

        out = cmd.run()
        cmd.assert_args(
            out,
            (cmd.single_arg, None),
            (cmd.pl_ext_stacking_from_aux_action_subc, None),
            (cmd.test_apps_shared_ext_action, None),
            (cmd.generic_app_ext_action, None),
            (cmd.generic_app_ext_flag, None),
        )

        sa_v = "single_arg_val"
        out = cmd.run(sa_v, cmd.generic_app_ext_action.ln_to_cli(), self.na, cmd.generic_app_ext_flag.ln_to_cli())
        cmd.assert_args(
            out,
            (cmd.single_arg, sa_v),
            (cmd.pl_ext_stacking_from_aux_action_subc, None),
            (cmd.test_apps_shared_ext_action, None),
            (cmd.generic_app_ext_action, [self.na]),
            (cmd.generic_app_ext_flag, 1),
        )

    def test_extending_aux_cmd(self):
        cmd = self.aux_cmd
        help = cmd.get_help_msg()
        help.assert_args(cmd.action_arg)
        help.assert_opts(
            cmd.generic_app_ext_action,
            cmd.generic_app_ext_flag,
            "help", "vk",
            cmd.pl_ext_stacking_from_aux_action,
            cmd.pl_ext_stacking_from_aux_flag,
            cmd.python_plugin_action,
            cmd.python_plugin_flag,
            "v",
        )
        help.assert_subcmds("help", cmd.subc)

        out = cmd.run()
        cmd.assert_args(
            out,
            (cmd.action_arg, None),
            (cmd.pl_ext_stacking_from_aux_action, None),
            (cmd.python_plugin_action, None),
            (cmd.generic_app_ext_action, None),
            (cmd.generic_app_ext_flag, None),
        )

        sa_v = "single_arg_val"
        out = cmd.run(sa_v, cmd.generic_app_ext_action.ln_to_cli(), self.na, cmd.generic_app_ext_flag.ln_to_cli())
        cmd.assert_args(
            out,
            (cmd.action_arg, [sa_v]),
            (cmd.pl_ext_stacking_from_aux_action, None),
            (cmd.python_plugin_action, None),
            (cmd.generic_app_ext_action, [self.na]),
            (cmd.generic_app_ext_flag, 1),
        )

    def test_extending_aux_subc(self):
        cmd = self.aux_subc
        help = cmd.get_help_msg()
        help.assert_args(cmd.action_arg)
        help.assert_opts(
            cmd.flag_opt,
            cmd.generic_app_ext_action,
            cmd.generic_app_ext_flag,
            "help", "vk",
            cmd.pl_ext_stacking_from_aux_action_subc,
            cmd.pl_ext_stacking_from_aux_flag_subc,
            cmd.python_plugin_action_subc,
            cmd.python_plugin_flag_subc,
            "v",
        )
        help.assert_subcmds(None)

        out = cmd.run()
        cmd.assert_args(
            out,
            (cmd.action_arg, None),
            (cmd.pl_ext_stacking_from_aux_action_subc, None),
            (cmd.python_plugin_action_subc, None),
            (cmd.generic_app_ext_action, None),
            (cmd.generic_app_ext_flag, None),
        )

        sa_v = "single_arg_val"
        out = cmd.run(sa_v, cmd.generic_app_ext_action.ln_to_cli(), self.na, cmd.generic_app_ext_flag.ln_to_cli())
        cmd.assert_args(
            out,
            (cmd.action_arg, [sa_v]),
            (cmd.pl_ext_stacking_from_aux_action_subc, None),
            (cmd.python_plugin_action_subc, None),
            (cmd.generic_app_ext_action, [self.na]),
            (cmd.generic_app_ext_flag, 1),
        )

    def test_error_msg_on_missing_implementation(self):
        cmd = self.missing_ext_impl_cmd
        help = cmd.get_help_msg()
        help.assert_args(cmd.single_arg, cmd.multi_arg)
        help.assert_opts(
            cmd.app_ext_missing_impl,
            cmd.flag_opt,
            "help",
            "vk",
            cmd.opt_taking_value,
            "v"
        )

        assert help.aux_exts == None
        assert help.pl_exts == None
        assert help.app_exts == True

        out = cmd.run()
        assert "Could not find implementation for app extension 'None'" in out
        assert re.search("From root .*test_apps/python_app/example/commands/extensions", out)

    @pytest.mark.skip
    def error_msg_on_extending_unknown_cmd(self):
        fail

    @pytest.mark.skip
    def test_error_in_before(self):
        fail

    @pytest.mark.skip
    def test_error_in_after(self):
        fail

    @pytest.mark.skip
    def test_error_in_cleanup(self):
        fail

class DisablingAppOpts():
    @pytest.mark.skip
    def test_app_opts_are_added_by_default(self):
        cmd = self.disabling_app_opts
        help = cmd.get_help_msg()
        help.assert_args(None)
        help.assert_opts(
            cmd.app_ext_missing_impl,
            cmd.flag_opt,
            "help",
            "vk",
            cmd.opt_taking_value,
            "v"
        )

    @pytest.mark.skip
    def test_target_is_not_loaded_by_default(self):
        cmd = self.disabling_app_opts
        cmd.get_help_msg()

    @pytest.mark.skip
    def test_disabling_app_opts(self):
        fail

    @pytest.mark.skip
    def test_disabling_app_opts_individually(self):
        fail

@pytest.mark.skip
class PluginCmdsInApp():
    def test_app_opts_are_added_by_default(self):
        fail
    
    def test_disabling_app_opts(self):
        fail
    
    def test_disabling_app_opts_individually(self):
        fail

@pytest.mark.skip
class AuxCmdsInApp():
    def test_app_opts_are_added_by_default(self):
        fail
    
    def test_disabling_app_opts(self):
        fail
    
    def test_disabling_app_opts_individually(self):
        fail


from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg

class CLICommon(CLIShared):
    _no_config_run_opts = {
        "with_configs": CLIShared.configs.suppress_plugin_collecting_config,
        "bypass_config_lookup": True
    }

    @pytest.fixture
    def no_config_run_opts(self):
        return self._no_config_run_opts

class TestEval(CLICommon):
    _cmd= origen.helpers.regressions.cli.CLI.in_app_cmds.eval

    def test_help_msg(self, cmd, no_config_run_opts):
        help = cmd.get_help_msg(run_opts=no_config_run_opts)
        help.assert_summary(cmd.help)
        help.assert_args(cmd.code)
        help.assert_bare_app_opts()

    def test_basic_eval(self, cmd, no_config_run_opts):
        d = cmd.demos["multi_statement_single_arg"]
        out = d.run(run_opts=no_config_run_opts)
        d.assert_present(out)

class TestInteractive(CLICommon):
    _cmd= origen.helpers.regressions.cli.CLI.in_app_cmds.i

    def test_help_msg(self, cmd, no_config_run_opts):
        help = cmd.get_help_msg(run_opts=no_config_run_opts)
        help.assert_summary(cmd.help)
        help.assert_args(None)
        help.assert_bare_app_opts()

    @pytest.mark.skip
    def test_interactive(self, cmd, no_config_run_opts):
        # TODO try to get an interactive test that just starts/stops
        proc = subprocess.Popen(["poetry", "run", "origen", "i"], universal_newlines=True, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            proc.stdin.flush()
            #proc.stdout.flush()
            m = 'print("hi from interactive!")'
            import time
            # time.sleep(10)
            assert proc.poll() is None
            # proc.stdin.write(f"{m}\n".encode())
            assert proc.poll() is None
            # lines = proc.stdout.readlines()
            # print(lines)
            # assert lines[-1] == m

            m = "print('hi again!')"
            # proc.stdin.write(f"{m}\n".encode())
            assert proc.poll() is None
            # lines = proc.stdout.readlines()
            # assert lines[0] == m

            proc.stdin.write("exit()\n")
            assert proc.wait(3) == 0
            lines = proc.stdout.readline()
            print(lines)
        finally:
            if proc.poll() is None:
                proc.kill()
            # print(proc.stdout.readline())
            # print(proc.stdout.readline())
            # print(proc.stdout.readline())
            # print(proc.stdout.readline())
            for l in proc.stdout:
                # lines = proc.stdout.readlines()
                if "CMD" in l:
                    break
                print(l)
        fail

# class TestCredentials(CLICommon):
#     def test_credentials(self):
#         ?
