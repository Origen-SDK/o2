import pytest, pathlib

from tests.shared import PythonAppCommon
from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg, CmdExtOpt

Cmd = CLIShared.Cmd

class CLICommon(CLIShared, PythonAppCommon):
    class AppCmds:
        def __init__(self):
            self.warmup_cmd = CLIShared.app_sub_cmd(
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
            self.nested_cmds = CLIShared.app_sub_cmd(
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
            self.disabling_app_opts = CLIShared.app_sub_cmd(
                "disabling_app_opts",
                help="Test disabling standard app opts",
                subcmds=[
                    Cmd(
                        "disable_targets_opt",
                        help="Disable the targets and no-targets opt",
                        subcmds=[
                            Cmd("disable_subc", help="Disables inherited from parent"),
                            Cmd("override_subc", help="Overrides disable inherited from parent"),
                        ]
                    ),
                    Cmd(
                        "disable_mode_opt",
                        help="Disable the mode opt",
                        subcmds=[
                            Cmd("disable_subc",help="Disables inherited from parent"),
                            Cmd("override_subc", help="Overrides disable inherited from parent"),
                        ]
                    ),
                    Cmd(
                        "disable_app_opts",
                        help="Disable all app opts",
                        subcmds=[
                            Cmd("disable_subc",help="Disables inherited from parent"),
                            Cmd("override_subc", help="Overrides disable inherited from parent"),
                        ]
                    )
                ]
            )
    app_cmds = AppCmds()
    app_commands = app_cmds

    _no_config_run_opts = {
        "with_configs": CLIShared.configs.suppress_plugin_collecting_config,
        "bypass_config_lookup": True
    }

    @pytest.fixture
    def no_config_run_opts(self):
        return self._no_config_run_opts

    @classmethod
    def no_config_run_opts_plus_config(cls, add_configs):
        return {
            "with_configs": [
                CLIShared.configs.suppress_plugin_collecting_config,
                *([add_configs] if isinstance(add_configs, str) or isinstance(add_configs, pathlib.Path) else add_configs)
            ],
            "bypass_config_lookup": True
        }