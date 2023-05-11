import pytest, pathlib
from origen.helpers.regressions import cli

from tests.shared import PythonAppCommon
from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg, CmdExtOpt
from test_apps_shared_test_helpers.cli.cmd_models import SrcBase

Cmd = CLIShared.Cmd

class EmptyApp(SrcBase):
    def __init__(self):
        self.src_type = cli.SrcTypes.APP
        self.name = "example"
        self.app_cmds = CLIShared.in_app_cmds.app.commands.extend(
            [],
            with_env = {"origen_app_config_paths": str(PythonAppCommon.to_config_path("empty_app_cmds.toml"))},
        )

class CLICommon(CLIShared, PythonAppCommon):
    class AppCmds(SrcBase):
        def __init__(self):
            self.src_type = cli.SrcTypes.APP
            self.name = "example"
            self.app_cmds = CLIShared.in_app_cmds.app.commands.extend([])
            self.app_cmds.replace_subcmds(
                Cmd(
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
                ),
                Cmd(
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
                ),
                Cmd(
                    "examples",
                    help="Run diff-based regression tests of the pattern and program generator",
                ),
                Cmd(
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
                ),
                Cmd(
                    "playground",
                    help="This is used to test Origen's app command definition and dispatch",
                    aliases=["y"],
                ),
            )
            self.intra_cmd_conflicts = CLIShared.app_sub_cmd(
                "intra_cmd_conflicts",
                help="Test intra app cmd conflicts",
                with_env={"ORIGEN_APP_INTRA_CMD_CONFLICTS": "1"},
                **CLIShared.python_plugin.intra_cmd_conflicts_args_opts_subcs(),
            )
            self.exts = {
                "app.arg_opt_warmup": {
                    "exts": [
                        *CmdExtOpt.from_src(
                            "python_plugin",
                            cli.cmd.SrcTypes.PLUGIN,
                            CmdExtOpt(
                                "pypl_single_opt",
                                help="Single opt from PYPL",
                                takes_value=True,
                            ),
                            CmdExtOpt(
                                "pypl_multi_opt",
                                help="Multi opt from PYPL",
                                ln_aliases=["PYPL"],
                                multi=True,
                            ),
                            CmdExtOpt(
                                "pypl_hidden",
                                help="Hidden opt from PYPL",
                                hidden=True,
                                sn="p",
                                ln="pypl_h_opt"
                            ),
                        ),
                        *CmdExtOpt.from_src(
                            "test_apps_shared_test_helpers",
                            cli.cmd.SrcTypes.PLUGIN,
                            CmdExtOpt(
                                "tas_single_opt",
                                help="Single opt from TAS",
                                ln="tas_sv",
                                takes_value=True,
                            ),
                            CmdExtOpt(
                                "tas_multi_opt",
                                help="Multi opt from TAS",
                                ln="tas_multi_opt",
                                ln_aliases=["tas_multi", "TAS"],
                                sn_aliases=["a"],
                                multi=True,
                            ),
                            CmdExtOpt(
                                "tas_hidden",
                                help="Hidden opt from TAS",
                                hidden=True,
                            ),
                        ),
                        *CmdExtOpt.from_src(
                            "python_app_exts",
                            cli.cmd.SrcTypes.AUX,
                            CmdExtOpt(
                                "ec_single_opt",
                                help="Single opt from EC",
                                takes_value=True,
                            ),
                            CmdExtOpt(
                                "ec_multi_opt",
                                help="Multi opt from EC",
                                ln="ec_multi_opt",
                                ln_aliases=["ec_multi", "EC"],
                                sn="e",
                                multi=True,
                            ),
                            CmdExtOpt(
                                "ec_hidden",
                                ln="ec_h_opt",
                                help="Hidden opt from EC",
                                hidden=True,
                            ),
                        ),
                    ],
                    "env": {"ORIGEN_APP_EXT_ARG_OPT_WARMUP": "1"},
                    "cfg": CLIShared.aux.ns.python_app_aux_cmds.exts_cfg,
                },
                "app.nested_app_cmds.nested_l1": {
                    "exts": [
                        *CmdExtOpt.from_src(
                            "python_plugin",
                            cli.cmd.SrcTypes.PLUGIN,
                            CmdExtOpt(
                                "pypl_single_opt_shallow",
                                help="Single opt from PYPL",
                                sn="p",
                                takes_value=True,
                            ),
                            CmdExtOpt(
                                "pypl_flag_opt_shallow",
                                help="Flag opt from PYPL",
                            ),
                        ),
                        *CmdExtOpt.from_src(
                            "test_apps_shared_test_helpers",
                            cli.cmd.SrcTypes.PLUGIN,
                            CmdExtOpt(
                                "tas_multi_opt_shallow",
                                help="Multi opt from TAS",
                                ln_aliases=["tas_m_opt", "tas_shallow"],
                                multi=True,
                            ),
                            CmdExtOpt(
                                "tas_flag_opt_shallow",
                                help="Flag opt from TAS",
                                ln="tas_f",
                                sn="f"
                            ),
                        ),
                        *CmdExtOpt.from_src(
                            "python_app_exts",
                            cli.cmd.SrcTypes.AUX,
                            CmdExtOpt(
                                "ec_single_opt_shallow",
                                help="Single opt from EC",
                                takes_value=True,
                            ),
                            CmdExtOpt(
                                "ec_flag_opt_shallow",
                                help="Flag opt from EC",
                                ln="ec_f",
                            ),
                        ),
                    ],
                    "env": {"ORIGEN_APP_EXT_NESTED": "1"},
                    "cfg": CLIShared.aux.ns.python_app_aux_cmds.exts_cfg,
                },
                "app.nested_app_cmds.nested_l1.nested_l2_b.nested_l3_a": {
                    "exts": [
                        *CmdExtOpt.from_src(
                            "python_plugin",
                            cli.cmd.SrcTypes.PLUGIN,
                            CmdExtOpt(
                                "pypl_single_opt_deep",
                                help="Single opt from PYPL",
                                sn="q",
                                takes_value=True,
                            ),
                            CmdExtOpt(
                                "pypl_flag_opt_deep",
                                help="Flag opt from PYPL",
                                ln="py_f",
                                sn="f",
                            ),
                        ),
                        *CmdExtOpt.from_src(
                            "test_apps_shared_test_helpers",
                            cli.cmd.SrcTypes.PLUGIN,
                            CmdExtOpt(
                                "tas_multi_opt_deep",
                                help="Multi opt from TAS",
                                ln="tas_opt",
                                multi=True,
                            ),
                            CmdExtOpt(
                                "tas_flag_opt_deep",
                                help="Flag opt from TAS",
                            )
                        ),
                        *CmdExtOpt.from_src(
                            "python_app_exts",
                            cli.cmd.SrcTypes.AUX,
                            CmdExtOpt(
                                "ec_single_opt_deep",
                                help="Single opt from EC",
                                ln="ec_opt",
                                ln_aliases=["ec_deep"],
                                takes_value=True,
                            ),
                            CmdExtOpt(
                                "ec_flag_opt_deep",
                                help="Flag opt from EC",
                                ln="ec_df",
                                sn="c"
                            ),
                        ),
                    ],
                    "env": {"ORIGEN_APP_EXT_NESTED": "1"},
                    "cfg": CLIShared.aux.ns.python_app_aux_cmds.exts_cfg,
                },
            }
            self.conflict_exts = {
                "app.arg_opt_warmup": {
                    "exts": [
                        *CmdExtOpt.from_src(
                            "python_plugin",
                            cli.cmd.SrcTypes.PLUGIN,
                            CmdExtOpt(
                                "flag_opt",
                                help="Flag opt from Python Plugin",
                            ),
                            CmdExtOpt(
                                "conflicts_from_python_plugin",
                                help="Some conflicts from Python Plugin",
                                ln_aliases=["python_plugin_conflicts"],
                                sn_aliases=["a"],
                            ),
                        ),
                        *CmdExtOpt.from_src(
                            "test_apps_shared_test_helpers",
                            cli.cmd.SrcTypes.PLUGIN,
                            CmdExtOpt(
                                "flag_opt",
                                help="Conflict with flag opt from TAS",
                                access_with_full_name=True,
                            ),
                            CmdExtOpt(
                                "conflicts_from_test_apps_shared",
                                help="Some conflicts from Test Apps Shared PL",
                                ln="TAS",
                                sn_aliases=["b"],
                            ),
                        ),
                        *CmdExtOpt.from_src(
                            "ext_conflicts",
                            cli.cmd.SrcTypes.AUX,
                            CmdExtOpt(
                                "flag_opt",
                                help="Conflict with flag opt from ext_conflicts AUX",
                                access_with_full_name=True,
                            ),
                            CmdExtOpt(
                                "conflicts_from_ext_conflicts",
                                help="Some conflicts from ext_conflicts AUX",
                                ln_aliases=["EX_Conflicts"],
                                sn_aliases=["c"],
                            ),
                        ),
                    ],
                    "env": {"ORIGEN_APP_EXT_CONFLICTS_ARG_OPT_WARMUP": "1"},
                    "cfg": CLIShared.aux.aux_cmds_dir.joinpath("ext_conflicts_cfg.toml"),
                },
                "python_plugin.plugin_test_args": {
                    "exts": [
                        *CmdExtOpt.from_src(
                            "test_apps_shared_test_helpers",
                            cli.cmd.SrcTypes.PLUGIN,
                            CmdExtOpt(
                                "tas_opt",
                                help="Opt from TAS",
                                ln="tas",
                                sn="z",
                                ln_aliases=["t_opt"],
                                sn_aliases=["c"],
                            ),
                            CmdExtOpt(
                                "tas_iln",
                                help="ILN from TAS",
                            ),
                        ),
                        *CmdExtOpt.from_src(
                            "ext_conflicts",
                            cli.cmd.SrcTypes.AUX,
                            CmdExtOpt(
                                "ec_opt",
                                help="Opt from EC",
                                ln="ec",
                                sn="e",
                                ln_aliases=["e_opt"],
                                sn_aliases=["d"],
                            ),
                            CmdExtOpt(
                                "tas_iln",
                                help="More conflicts",
                                access_with_full_name=True,
                            ),
                        ),
                        *CmdExtOpt.from_src(
                            "example",
                            cli.cmd.SrcTypes.APP,
                            CmdExtOpt(
                                "app_opt",
                                help="Conflict with cmd/aux/plugin opts",
                                ln="app",
                                ln_aliases=["app_opt", "app_flag"],
                                sn_aliases=["g"],
                            ),
                            CmdExtOpt(
                                "tas_iln",
                                help="Conflict iln from App",
                                access_with_full_name=True,
                            ),
                        ),
                    ],
                    "env": {"ORIGEN_APP_PL_CMD_CONFLICTS": "1"},
                    "cfg": CLIShared.aux.aux_cmds_dir.joinpath("ext_conflicts_cfg.toml"),
                }
            }

        @property
        def arg_opt_warmup(self):
            return self.warmup_cmd

    app_cmds = AppCmds()
    app_commands = app_cmds
    empty_app = EmptyApp()

    _no_config_run_opts = {
        "with_configs": CLIShared.configs.suppress_plugin_collecting_config,
        "bypass_config_lookup": True
    }

    @classmethod
    def loaded_plugins_alpha(cls):
        return [
            cls.plugins.pl_ext_cmds,
            cls.plugins.py_pl,
            cls.plugins.python_plugin_no_cmds,
            cls.plugins.tas,
        ]

    @classmethod
    def loaded_plugins(cls):
        return [
            cls.plugins.python_plugin_no_cmds,
            cls.plugins.pl_ext_cmds,
            cls.plugins.tas,
            cls.plugins.py_pl,
        ]

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