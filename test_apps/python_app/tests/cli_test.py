# FOR_PR convert assert args
import pytest, pathlib
import subprocess
import os
import origen
from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg

class CLICommon(CLIShared):
    _no_config_run_opts = {
        "with_configs": CLIShared.configs.suppress_plugin_collecting_config,
        "bypass_config_lookup": True
    }

    @pytest.fixture
    def no_config_run_opts(self):
        return self._no_config_run_opts

    def show_per_cmd_targets(self, targets=None, **kwargs):
        prefix = "Current Targets: "
        if targets is not None:
            kwargs.setdefault("run_opts", {})["targets"] = targets
        out = self.eval(f"print( f'{prefix}{{origen.target.current_targets}}' )", **kwargs)
        out = out.split("\n")
        return eval(next(t.replace(prefix, '') for t in out if t.startswith(prefix)))

    class Targets:
        hawk = "hawk"
        falcon = "falcon"
        eagle = "eagle"
        uflex = "uflex"
    
    targets = Targets()


Cmd = CLIShared.Cmd

origen_cli = os.getenv('TRAVIS_ORIGEN_CLI') or 'origen'

def test_origen_v():
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

        # FOR_PR
        assert len(help.opts) == 3
        help.assert_help_opt_at(0)
        help.assert_v_opt_at(1)
        help.assert_vk_opt_at(2)

        assert set(help.subcmd_names) == set(self.in_app_cmds.all_names_add_help)
        print(help.app_cmd_shortcuts)
        assert help.app_cmd_shortcuts == self.cmd_shortcuts__app
        # FOR_PR plugin commands
        # assert help.pl_cmd_shortcuts == self.cmd_shortcuts__default_plugins
        # FOR_PR Aux commands
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
        help.assert_args(cmd.first, cmd.second)
        help.assert_opts(
            cmd.flag_opt,
            'h', 'm',
            cmd.multi_opt,
            'nt',
            cmd.single_opt,
            't', 'v', 'vk'
        )
        help.assert_subcmds(None)

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
            cmd.flag_opt.sn_to_cli(),
            cmd.single_opt.sna_to_cli(), sv,
            cmd.multi_opt.sna_to_cli(), mo0, mo12,
            cmd.multi_opt.ln_to_cli(), mo4,
            cmd.hidden_flag_opt.ln_to_cli(),
        )
        cmd.assert_args(
            out,
            (cmd.first, rv),
            (cmd.second, [m0, m12]),
            (cmd.flag_opt, 1),
            (cmd.single_opt, sv),
            (cmd.multi_opt, [mo0, mo12, mo4]),
            (cmd.hidden_flag_opt, 1)
        )

        out = cmd.run(
            rv,
            cmd.flag_opt.sn_to_cli(), cmd.flag_opt.sn_to_cli(),
            cmd.multi_opt.sna_to_cli(), mo0, mo12,
        )
        cmd.assert_args(
            out,
            (cmd.first, rv),
            (cmd.flag_opt, 2),
            (cmd.multi_opt, [mo0, mo12])
        )

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
            "help", "mode", "no_targets",
            cmd.pl_ext_cmds_generic_ext,
            "targets", "v", 'vk'
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
            "help", "m", "nt",
            cmd.pl_ext_stacking_from_aux_action,
            cmd.pl_ext_stacking_from_aux_flag,
            "t",
            cmd.test_apps_shared_ext_action,
            cmd.test_apps_shared_ext_flag,
            "v", 'vk',
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
            "help", "m", "nt",
            cmd.pl_ext_stacking_from_aux_action_subc,
            cmd.pl_ext_stacking_from_aux_flag_subc,
            "t",
            cmd.test_apps_shared_ext_action,
            cmd.test_apps_shared_ext_flag,
            "v", 'vk',
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
            "help", "m", "nt",
            cmd.pl_ext_stacking_from_aux_action,
            cmd.pl_ext_stacking_from_aux_flag,
            cmd.python_plugin_action,
            cmd.python_plugin_flag,
            "t", "v", 'vk',
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
            "help", "m", "nt",
            cmd.pl_ext_stacking_from_aux_action_subc,
            cmd.pl_ext_stacking_from_aux_flag_subc,
            cmd.python_plugin_action_subc,
            cmd.python_plugin_flag_subc,
            "t", "v", 'vk',
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
            "help", "m", "nt",
            cmd.opt_taking_value,
            "t", "v", 'vk'
        )

        assert help.aux_exts == None
        assert help.pl_exts == None
        assert help.app_exts == True

        out = cmd.run()
        r = origen.app.root.joinpath('example/commands/extensions')
        assert "Could not find implementation for app extension 'None'" in out
        assert f"From root '{r}', searched:" in out
        assert f"plugin.python_plugin.plugin_test_args.py" in out
        assert f"plugin{os.sep}python_plugin.plugin_test_args.py" in out
        assert f"plugin{os.sep}python_plugin{os.sep}plugin_test_args.py" in out

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

class TestReservedOpts(CLICommon):
    cmd = CLIShared.app_sub_cmd(
        "reserved_opt_error_gen",
        help = "Generate error messages when reserved opts are used",
        opts=[
            CmdOpt("conflicting_help", help="Conflicting Opt"),
            CmdOpt("conflicting_target", help="Conflicting Opt"),
            CmdOpt("non_conflicting", help="Non-Conflicting Opt", ln="non_conflicting", sn="n"),
        ],
        subcmds=[
            Cmd(
                "single_conflicting_opt",
                help="Generate error messages for reserved opts",
                opts=[
                    CmdOpt("non_conflicting", help="Non-Conflicting Opt"),
                    CmdOpt("conflicting_target", help="Conflicting Opt"),
                ],
            ),
            Cmd(
                "multiple_conflicting_opts",
                help="Generate error messages for reserved opts",
                opts=[
                    CmdOpt("conflicting_target", help="Conflicting Opt"),
                    CmdOpt("conflicting_mode", help="Conflicting Opt"),
                    CmdOpt("not_conflicting", help="Non-Conflicting Opt", sn='n'),
                    CmdOpt("conflicting_no_targets", help="Conflicting Opt"),
                    CmdOpt("conflicting_target_again", help="Conflicting Opt"),
                    CmdOpt("conflicting_v", help="Conflicting Opt"),
                    CmdOpt("conflicting_vk", help="Conflicting Opt"),
                    CmdOpt("conflicting_help", help="Conflicting Opt", ln_aliases=["help_alias"], sn_aliases=["g", "i"]),
                ],
                subcmds=[
                    Cmd(
                        "subc",
                        help="Generate error messages for reserved opts - subc",
                        opts=[
                            CmdOpt("conflicting_help", help="Conflicting Opt"),
                        ],
                        subcmds=[
                            Cmd(
                                "subc",
                                help="Generate error messages for reserved opts - subc - subc",
                                opts=[
                                    CmdOpt("conflicting_help", help="Conflicting Opt", ln_aliases=["help2", "help3"]),
                                    CmdOpt("conflicting_v", help="Conflicting Opt", sn="c"),
                                ],
                            ),
                        ]
                    )
                ]
            )
        ],
        with_env={"ORIGEN_APP_TEST_RESERVED_OPT_ERRORS": "1"},
    )

    @classmethod
    def setup_class(cls):
        if not hasattr(cls, "base_cmd_help"):
            cls.base_cmd_help = cls.cmd.get_help_msg()
        if not hasattr(cls, "ext_cmd_help"):
            cls.ext_cmd_help = cls.ext_cmd.get_help_msg()

    @classmethod
    def teardown_class(cls):
        delattr(cls, "base_cmd_help")
        delattr(cls, "ext_cmd_help")

    def test_opts_are_added_with_respect_to_errors(self):
        help = self.base_cmd_help
        help.assert_args(None)
        help.assert_opts(
            self.cmd.conflicting_help,
            self.cmd.conflicting_target,
            "h", "m",
            self.cmd.non_conflicting,
            "nt", "t", "v", "vk"
        )
        help.assert_subcmds("help", self.cmd.multiple_conflicting_opts, self.cmd.single_conflicting_opt)

        cmd = self.cmd.single_conflicting_opt
        help = cmd.get_help_msg()
        help.assert_opts(
            cmd.conflicting_target,
            "h", "m", "nt",
            cmd.non_conflicting,
            "t", "v", "vk"
        )

        cmd = self.cmd.multiple_conflicting_opts
        help = cmd.get_help_msg()
        help.assert_opts(
            cmd.conflicting_help,
            cmd.conflicting_mode,
            cmd.conflicting_no_targets,
            cmd.conflicting_target,
            cmd.conflicting_target_again,
            cmd.conflicting_v,
            cmd.conflicting_vk,
            "h", "m",
            cmd.not_conflicting,
            "nt", "t", "v", "vk"
        )

        cmd = cmd.subc
        help = cmd.get_help_msg()
        help.assert_opts(
            cmd.conflicting_help,
            "h", "m", "nt", "t", "v", "vk"
        )

        cmd = cmd.subc
        help = cmd.get_help_msg()
        help.assert_opts(
            cmd.conflicting_v,
            cmd.conflicting_help,
            "h", "m", "nt", "t", "v", "vk"
        )

    errors = [
        (cmd, cmd.conflicting_help, [("sn", "h")]),
        (cmd, cmd.conflicting_target, [("sn", "t"), ("ln", "target")]),
        (cmd.multiple_conflicting_opts, cmd.multiple_conflicting_opts.conflicting_help, [("ln", "help"), ("sn", "h")]),
        (cmd.multiple_conflicting_opts, cmd.multiple_conflicting_opts.conflicting_vk, [("lna", "verbosity_keywords"), ("ln", "vk")]),
        (cmd.multiple_conflicting_opts, cmd.multiple_conflicting_opts.conflicting_v, [("ln", "verbosity"), ("sn", "v")]),
        (cmd.multiple_conflicting_opts, cmd.multiple_conflicting_opts.conflicting_no_targets, [("lna", "no_targets"), ("lna", "no_target")]),
        (cmd.multiple_conflicting_opts, cmd.multiple_conflicting_opts.conflicting_target_again, [("lna", "targets"), ("ln", "target"), ("sn", "t")]),
        (cmd.multiple_conflicting_opts, cmd.multiple_conflicting_opts.conflicting_mode, [("lna", "mode")]),
        (cmd.multiple_conflicting_opts, cmd.multiple_conflicting_opts.conflicting_target, [("sn", "t")]),
        (cmd.multiple_conflicting_opts.subc, cmd.multiple_conflicting_opts.subc.conflicting_help, [("sn", "h")]),
        (cmd.multiple_conflicting_opts.subc.subc, cmd.multiple_conflicting_opts.subc.subc.conflicting_v, [("sna", "v"), ("ln", "verbosity")]),
        (cmd.multiple_conflicting_opts.subc.subc, cmd.multiple_conflicting_opts.subc.subc.conflicting_help, [("lna", "help")]),
        (cmd.single_conflicting_opt, cmd.single_conflicting_opt.conflicting_target, [("ln", "target")]),
    ]
    @pytest.mark.parametrize(
        "cmd,opt,type,name",
        [(o[0], o[1], inner[0], inner[1]) for o in errors for inner in o[2]],
        ids=[f"{o[0].name}-{o[1].name}-{inner[0]}-{inner[1]}" for o in errors for inner in o[2]],
    )
    def test_error_msg_using_reserved_opts(self, cmd, opt, type, name):
        if type == "sn":
            assert cmd.reserved_opt_sn_conflict_msg(opt, name) in self.base_cmd_help.text
        elif type == "sna":
            assert cmd.reserved_opt_sna_conflict_msg(opt, name) in self.base_cmd_help.text
        elif type == "ln":
            assert cmd.reserved_opt_ln_conflict_msg(opt, name) in self.base_cmd_help.text
        elif type == "lna":
            assert cmd.reserved_opt_lna_conflict_msg(opt, name) in self.base_cmd_help.text
        else:
            raise RuntimeError(f"Unknown type {type}")

    def test_opts_are_still_available_under_non_reserved_names(self):
        cmd = self.cmd
        out = cmd.run("--conflicting_help", "--conflicting_target", "--non_conflicting", "-n")
        cmd.assert_args(
            out,
            (cmd.conflicting_help, 1),
            (cmd.conflicting_target, 1),
            (cmd.non_conflicting, 2)
        )

        cmd = self.cmd.single_conflicting_opt
        out = cmd.run("--non_conflicting", "--conflicting_target")
        cmd.assert_args(
            out,
            (cmd.conflicting_target, 1),
            (cmd.non_conflicting, 1)
        )

        cmd = self.cmd.multiple_conflicting_opts
        out = cmd.run(
            "--conflicting_target",
            "--conflicting_mode",
            "-n",
            "--conflicting_no_targets",
            "--conflicting_target_again",
            "--conflicting_v",
            "--conflicting_vk",
            "--conflicting_help", "-g", "-i", "--help_alias",
        )
        cmd.assert_args(
            out,
            (cmd.conflicting_target, 1),
            (cmd.conflicting_mode, 1),
            (cmd.not_conflicting, 1),
            (cmd.conflicting_no_targets, 1),
            (cmd.conflicting_target_again, 1),
            (cmd.conflicting_v, 1),
            (cmd.conflicting_vk, 1),
            (cmd.conflicting_help, 4),
        )

        cmd = cmd.subc
        out = cmd.run("--conflicting_help")
        cmd.assert_args(out, (cmd.conflicting_help, 1))

        cmd = cmd.subc
        out = cmd.run(
            "--conflicting_help", "--help2", "--help3",
            "-c",
        )
        cmd.assert_args(
            out,
            (cmd.conflicting_help, 3),
            (cmd.conflicting_v, 1)
        )

    ext_error_msgs_env = {"ORIGEN_APP_EXT_TEST_RESERVED_OPT_ERRORS": "1", "origen_bypass_config_lookup": "1"}
    ext_cmd = origen.helpers.regressions.cli.CLI.in_app_cmds.eval.extend(
        CmdExtOpt.from_src(
            "example",
            SrcTypes.APP,
            CmdExtOpt(
                "conflicting_target",
                help="Conflicting Core Extension"
            ),
            CmdExtOpt(
                "conflicting_no_target",
                help="Conflicting Core Extension",
                sn="n",
            ),
            CmdExtOpt(
                "conflicting_mode",
                help="Conflicting Core Extension",
                ln="mode_conflict",
                sn_aliases=["m"]
            ),
            CmdExtOpt(
                "conflicting_help",
                help="Conflicting Core Extension",
                ln="help_conflict",
                ln_aliases=["help1"]
            ),
            CmdExtOpt(
                "conflicting_v",
                help="Conflicting Core Extension",
                sn_aliases=["w"]
            ),
            CmdExtOpt(
                "conflicting_vk",
                help="Conflicting Core Extension"
            ),
        ),
        with_env=ext_error_msgs_env,
        from_configs=CLIShared.configs.suppress_plugin_collecting_config,
    )

    def test_ext_opts_are_added_with_respect_to_errors(self):
        cmd = self.ext_cmd
        help = self.ext_cmd_help
        help.assert_args(cmd.code)
        help.assert_opts(
            cmd.conflicting_target,
            cmd.conflicting_v,
            cmd.conflicting_vk,
            "h",
            cmd.conflicting_help,
            "m",
            cmd.conflicting_mode,
            cmd.conflicting_no_target,
            "nt", "t", "v", "vk"
        )
        help.assert_subcmds(None)

    ext_errors = [
        (ext_cmd, ext_cmd.conflicting_target, [("ln", "target"), ("sn", "t")]),
        (ext_cmd, ext_cmd.conflicting_no_target, [("lna", "no_target"), ("lna", "no_targets")]),
        (ext_cmd, ext_cmd.conflicting_help, [("sn", "h"), ("lna", "help")]),
        (ext_cmd, ext_cmd.conflicting_mode, [("lna", "mode")]),
        (ext_cmd, ext_cmd.conflicting_v, [("ln", "verbosity"), ("sn", "v")]),
        (ext_cmd, ext_cmd.conflicting_vk, [("lna", "verbosity_keywords"), ("lna", "vk")]),
    ]
    @pytest.mark.parametrize(
        "cmd,opt,type,name",
        [(o[0], o[1], inner[0], inner[1]) for o in ext_errors for inner in o[2]],
        ids=[f"{o[0].name}-{o[1].name}-{inner[0]}-{inner[1]}" for o in ext_errors for inner in o[2]],
    )
    def test_ext_error_msg_using_reserved_opts(self, cmd, opt, type, name):
        if type == "sn":
            assert cmd.reserved_opt_sn_conflict_msg(opt, name) in self.ext_cmd_help.text
        elif type == "sna":
            assert cmd.reserved_opt_sna_conflict_msg(opt, name) in self.ext_cmd_help.text
        elif type == "ln":
            assert cmd.reserved_opt_ln_conflict_msg(opt, name) in self.ext_cmd_help.text
        elif type == "lna":
            assert cmd.reserved_opt_lna_conflict_msg(opt, name) in self.ext_cmd_help.text
        else:
            raise RuntimeError(f"Unknown type {type}")

    def test_ext_opts_are_still_available_under_non_reserved_names(self):
        cmd = self.ext_cmd
        out = cmd.run(
            "from test_apps_shared_test_helpers.aux_cmds import run; run()",
            "--conflicting_target",
            "-n",
            "--mode_conflict", "-m",
            "--help_conflict", "--help1",
            "--conflicting_v", "-w",
            "--conflicting_vk",
        )
        self.Cmd.assert_args(
            cmd,
            out,
            (cmd.conflicting_target, 1),
            (cmd.conflicting_no_target, 1),
            (cmd.conflicting_mode, 2),
            (cmd.conflicting_help, 2),
            (cmd.conflicting_v, 2),
            (cmd.conflicting_vk, 1),
        )

# from tests.proc_funcs import target_proc_funcs
# from origen.helpers.env import in_new_origen_proc

@pytest.mark.skip
class TestTarget(CLICommon):
    def test_loading_targets_set_by_app(self):
        retn = in_new_origen_proc(mod=target_proc_funcs)
        assert retn['target_pre_load'] == None
        assert retn['tester_pre_load'] == []
        assert retn['dut_pre_load'] == None
        assert retn['first_load_done_pre_load'] == False
        assert retn['target_post_load'] == [self.targets.falcon]
        assert retn['tester_post_load'] == [self.targets.uflex]
        assert retn['dut_post_load'] == [self.targets.eagle]
        assert retn['first_load_done_post_load'] == True

    def test_getting_the_current_target(self):
        fail

@pytest.mark.skip
class TestTargetOpts(CLICommon):
    def test_target_can_be_set(self):
        targets = self.show_per_cmd_targets(targets=self.targets.eagle)
        assert targets == [self.targets.eagle]

        targets = self.show_per_cmd_targets(targets=[self.targets.hawk, self.targets.uflex])
        assert targets == [self.targets.hawk, self.targets.uflex]

    def test_no_target_can_be_used_per_command(self):
        targets = self.show_per_cmd_targets()
        assert len(targets) != 0

        targets = self.show_per_cmd_targets(targets=False)
        assert len(targets) == 0

@pytest.mark.skip
class TestModeOpts():
    def test_():
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
            cmd.opt_taking_value,
            "v",
            'vk'
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

@pytest.mark.skip
class TestTargetCmd(CLICommon):
    def test_fail():
        fail

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
