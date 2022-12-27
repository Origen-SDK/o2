import pytest, pathlib
import subprocess
import os
import origen
from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg

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
    @classmethod
    @property
    def cmd_shortcuts__app(cls):
        return {
            'arg_opt_warmup': 'arg_opt_warmup',
            "examples": "examples",
            "playground": "playground",
            "y": "playground",
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

class TestPluginCommandsAreAdded:
    ...

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

@pytest.mark.skip
class TestExtendingFromAppCommands:
    def test_extending_global_cmds(self):
        fail

    def test_extending_plugin_cmds(self):
        fail

    def test_extending_aux_cmds(self):
        fail
