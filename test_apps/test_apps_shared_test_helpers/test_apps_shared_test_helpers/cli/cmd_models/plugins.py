import origen
from origen.helpers.regressions import cli
from . import Cmd, CmdArg, CmdOpt, SrcBase

class Plugin(SrcBase):
    @property
    def src_type(self):
        return cli.SrcTypes.PLUGIN

class PlExtCmds(Plugin):
    def __init__(self):
        self.name = "pl_ext_cmds"
        self.pl_ext_cmds = self.pl_cmd(
            self.name,
        )

class PythonPlugin(Plugin):
    Cmd = Cmd

    def __init__(self):
        self.name = "python_plugin"
        subcmds = [
            Cmd(
                "do_actions",
                help="Perform the given actions",
                args=[CmdArg(
                    name="actions",
                    help="Actions to perform",
                    use_delimiter=True,
                )],
            ),
            Cmd(
                "echo",
                help="Echos the input",
                args=[CmdArg(
                    name="input",
                    help="Input to echo",
                    use_delimiter=True,
                )],
                opts=[CmdOpt(
                    name="repeat",
                    help="Echo again (repeat)",
                    ln="repeat",
                    sn="r",
                )],
            ),
            Cmd(
                "plugin_says_hi",
                help="Say 'hi' from the python plugin",
                opts=[
                    CmdOpt(
                        name="times",
                        help="Number of times for the python plugin to say",
                        value_name="TIMES",
                        ln="times",
                        sn="x"
                    ),
                    CmdOpt(
                        name="loudly",
                        help="LOUDLY say hi",
                        ln="loudly",
                        sn="l"
                    ),
                    CmdOpt(
                        name="to",
                        help="Specify who should be greeted",
                        multi=True,
                    )
                ]
            ),
            Cmd(
                "plugin_test_args",
                help="Test command for a plugin",
                args=[
                    CmdArg(
                        name="single_arg",
                        help="Single Arg",
                    ),
                    CmdArg(
                        name="multi_arg",
                        help="Multi Arg",
                        multi=True,
                    ),
                ],
                opts=[
                    CmdOpt(
                        name="opt_taking_value",
                        help="Opt taking a single value",
                        ln="opt",
                    ),
                    CmdOpt(
                        name="flag_opt",
                        help="Flag Opt",
                        ln="flag",
                    ),
                    CmdOpt(
                        name="sn_only",
                        help="Opt with short name only",
                        sn="n",
                    ),
                    CmdOpt(
                        name="opt_with_aliases",
                        help="Opt with aliases",
                        ln_aliases=["alias", "opt_alias"],
                        sn_aliases=["a", "b"],
                    )
                ],
                subcmds=[
                    Cmd(
                        "subc",
                        help="Test Subcommand for plugin_test_args",
                        args=[
                            CmdArg(
                                name="single_arg",
                                help="Single Arg For Subcommand",
                            ),
                        ],
                        opts=[
                            CmdOpt(
                                name="flag_opt",
                                help="Flag Opt For Subcommand",
                            ),
                            CmdOpt(
                                name="subc_sn_only",
                                help="Opt with short name only for subc",
                                sn="n",
                            ),
                            CmdOpt(
                                name="subc_opt_with_aliases",
                                help="Opt with aliases for subc",
                                ln="subc_opt",
                                ln_aliases=["subc_alias", "subc_opt_alias"],
                                sn_aliases=["a", "b"]
                            ),
                        ]
                    )
                ]
            ),
            Cmd(
                "plugin_test_ext_stacking",
                help="Test ext stacking for plugin command",
                args=[
                    CmdArg(
                        name="single_arg",
                        help="Single Arg",
                    ),
                ],
                opts=[
                    CmdOpt(
                        name="flag_opt",
                        help="Flag Opt",
                    ),
                ],
                subcmds=[
                    Cmd(
                        "subc",
                        help="Test Subcommand for ext stacking",
                        args=[
                            CmdArg(
                                name="single_arg",
                                help="Single Arg",
                            ),
                        ],
                        opts=[
                            CmdOpt(
                                name="flag_opt",
                                help="Flag Opt",
                            ),
                        ],
                    )
                ]
            )
        ]
        if origen.app:
            subcmds.insert(0, Cmd(
                "disabling_app_opts_from_pl",
                help="Test disabling standard app opts from plugin commands"
            ))

        self.python_plugin = self.pl_cmd(
            self.name,
            subcmds=subcmds
        )

        self.disabling_app_opts_from_pl = self.pl_sub_cmd(
            self.name,
            "disabling_app_opts_from_pl",
            help="Test disabling standard app opts from plugin commands",
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
            ],
        )
        self.intra_cmd_conflicts = self.pl_sub_cmd(
            self.name,
            "intra_cmd_conflicts",
            help="PL cmd with conflicting args and opts within the cmd",
            with_env={"ORIGEN_PL_INTRA_CMD_CONFLICTS": "1"},
            **self.intra_cmd_conflicts_args_opts_subcs(),
        )
    
    @property
    def intra_cmd_conflicts_list(self):
        if not hasattr(self, "_intra_cmd_conflicts_list"):
            self._intra_cmd_conflicts_list = self.get_intra_cmd_conflicts_list(self.intra_cmd_conflicts)
        return self._intra_cmd_conflicts_list

    @classmethod
    def intra_cmd_conflicts_args_opts_subcs(cls):
        return {
            "args": [
                CmdArg(
                    name="arg0",
                    help="Arg 0",
                ),
                CmdArg(
                    name="arg1",
                    help="Arg 1",
                ),
                CmdArg(
                    name="arg2",
                    help="Arg 2",
                ),
            ],
            "opts": [
                CmdOpt(
                    name="opt",
                    help="Opt 0",
                    ln_aliases=["opt0"]
                ),
                CmdOpt(
                    name="arg_clash",
                    help="Arg-Opt clash in ln/lna (okay)",
                    ln="arg0",
                    ln_aliases=["arg1"],
                ),
                CmdOpt(
                    name="reserved_prefix_in_ln_lna",
                    help="Reserved prefix in ln and lna",
                    ln_aliases=["ext_opt_lna"],
                ),
                CmdOpt(
                    name="intra_opt_conflicts",
                    help="Various intra-opt conflicts",
                    ln="intra_opt_cons",
                    sn="c",
                    ln_aliases=["intra_opt_conflicts", "intra_opt_cons2"],
                    sn_aliases=["a", "b", "e"]
                ),
                CmdOpt(
                    name="inter_opt_conflicts",
                    help="Various inter-opt conflicts",
                    sn_aliases=["d"],
                ),
                CmdOpt(
                    name="opt0",
                    help="Inferred long name clash",
                ),
            ],
            "subcmds": [
                Cmd(
                    "conflicts_subc",
                    help="Subcommand with conflicts",
                    args=[
                        CmdArg(
                            name="arg0",
                            help="Arg 0",
                        ),
                        CmdArg(
                            name="sub_arg_1",
                            help="Subc Arg 1",
                        ),
                    ],
                    opts=[
                        CmdOpt(
                            name="opt",
                            help="Opt 0",
                            ln_aliases=["subc_opt"],
                        ),
                        CmdOpt(
                            name="intra_subc_conflicts",
                            help="Intra-opt conflicts for subc",
                            sn="r",
                            ln="intra_subc_conflicts",
                        ),
                        CmdOpt(
                            name="intra_subc_lna_iln_conflict",
                            help="Intra-opt iln conflict",
                        ),
                        CmdOpt(
                            name="inter_subc_conflicts",
                            help="Inter-opt conflicts for subc",
                        ),
                    ]
                )
            ]
        }
    
    @classmethod
    def get_intra_cmd_conflicts_list(self, base_cmd):
        return [
            ["duplicate", base_cmd.arg0, 0],
            ["duplicate", base_cmd.arg1, 2],
            ["reserved_prefix_arg_name", "ext_opt.arg"],
            ["duplicate", base_cmd.opt, 0],
            ["reserved_prefix_opt_name", "ext_opt.opt", None],
            ["reserved_prefix_ln", base_cmd.reserved_prefix_in_ln_lna,"ext_opt.ln"],
            ["reserved_prefix_lna", base_cmd.reserved_prefix_in_ln_lna, "ext_opt.lna"],
            ["inter_ext_sna_sn", base_cmd.intra_opt_conflicts, "c"],
            ["repeated_sna", base_cmd.intra_opt_conflicts, "b", 1],
            ["inter_ext_sna_sn", base_cmd.intra_opt_conflicts, "c"],
            ["repeated_sna", base_cmd.intra_opt_conflicts, "e", 5],
            ["inter_ext_lna_ln", base_cmd.intra_opt_conflicts, "intra_opt_cons"],
            ["repeated_lna", base_cmd.intra_opt_conflicts, "intra_opt_cons2", 2],
            ["arg_opt_name_conflict", base_cmd.arg0, 0],
            [base_cmd.conflicts_subc, "duplicate", base_cmd.conflicts_subc.sub_arg_1, 1],
            [base_cmd.conflicts_subc, "reserved_prefix_arg_name", "ext_opt.subc_arg"],
            [base_cmd.conflicts_subc, "reserved_prefix_opt_name", "ext_opt.subc_opt", None],
            [base_cmd.conflicts_subc, "reserved_prefix_lna", base_cmd.conflicts_subc.opt, "ext_opt.subc_opt_lna"],
            [base_cmd.conflicts_subc, "inter_ext_sna_sn", base_cmd.conflicts_subc.intra_subc_conflicts, "r"],
            [base_cmd.conflicts_subc, "inter_ext_lna_ln", base_cmd.conflicts_subc.intra_subc_conflicts, "intra_subc_conflicts"],
            [base_cmd.conflicts_subc, "inter_ext_lna_iln", base_cmd.conflicts_subc.intra_subc_lna_iln_conflict],
            [base_cmd.conflicts_subc, "duplicate", base_cmd.conflicts_subc.intra_subc_conflicts, 2],
            ["ln", "lna", base_cmd.inter_opt_conflicts, base_cmd.intra_opt_conflicts, "intra_opt_conflicts"],
            ["sn", "sna", base_cmd.inter_opt_conflicts, base_cmd.intra_opt_conflicts, "a"],
            ["lna", "ln", base_cmd.inter_opt_conflicts, base_cmd.intra_opt_conflicts, "intra_opt_cons"],
            ["lna", "lna", base_cmd.inter_opt_conflicts, base_cmd.reserved_prefix_in_ln_lna, "ext_opt_lna"],
            ["lna", "iln", base_cmd.inter_opt_conflicts, base_cmd.reserved_prefix_in_ln_lna, "reserved_prefix_in_ln_lna"],
            ["sna", "sna", base_cmd.inter_opt_conflicts, base_cmd.intra_opt_conflicts, "b"],
            ["sna", "sn", base_cmd.inter_opt_conflicts, base_cmd.intra_opt_conflicts, "c"],
            ["iln", "lna", "opt0", base_cmd.opt],
            ["intra_cmd_not_placed", "opt0"],
            [base_cmd.conflicts_subc, "ln", "iln", base_cmd.conflicts_subc.inter_subc_conflicts, base_cmd.conflicts_subc.opt, "opt"],
            [base_cmd.conflicts_subc, "lna", "ln", base_cmd.conflicts_subc.inter_subc_conflicts, base_cmd.conflicts_subc.intra_subc_conflicts, "intra_subc_conflicts"],
            [base_cmd.conflicts_subc, "sna", "sn", base_cmd.conflicts_subc.inter_subc_conflicts, base_cmd.conflicts_subc.intra_subc_conflicts, "r"],
        ]

    @property
    def base_cmd(self):
        return self.python_plugin

    @property
    def ordered_subcmds(self):
        return [
            self.do_actions,
            self.echo,
            "help",
            self.plugin_says_hi,
            self.plugin_test_args,
            self.plugin_test_ext_stacking,
        ]

class PythonPluginNoCmds(Plugin):
    def __init__(self):
        self.name = "python_plugin_no_cmds"
        self.python_plugin_no_cmds = self.pl_cmd(
            self.name
        )

    @property
    def base_cmd(self):
        return self.python_plugin_no_cmds

class PythonPluginTheSecond(Plugin):
    def __init__(self):
        self.name = "python_plugin_the_second"
        self.python_plugin_the_second = self.pl_cmd(
            self.name
        )

    @property
    def base_cmd(self):
        return self.python_plugin_the_second

class TestAppsSharedTestHelpers(Plugin):
    def __init__(self):
        self.name = "test_apps_shared_test_helpers"
        self.test_apps_shared_test_helpers = self.pl_cmd(
            self.name
        )

    @property
    def base_cmd(self):
        return self.test_apps_shared_test_helpers

class Plugins:
    def __init__(self):
        self.plugins = {
            "pl_ext_cmds": PlExtCmds(),
            "python_plugin": PythonPlugin(),
            "python_plugin_no_cmds": PythonPluginNoCmds(),
            "python_plugin_the_second": PythonPluginTheSecond(),
            "test_apps_shared_test_helpers": TestAppsSharedTestHelpers()
        }

    @property
    def tas(self):
        return self.test_apps_shared_test_helpers

    @property
    def py_pl(self):
        return self.python_plugin

    @property
    def python_no_app_collected_pl_names(self):
        return list(self.plugins.keys())

    @property
    def python_no_app_config_pl_names(self):
        return [
            'python_plugin',
            'python_plugin_the_second',
            'python_plugin_no_cmds'
        ]

    def __getattr__(self, name):
        if name in self.plugins:
            return self.plugins[name]
        else:
            return object.__getattribute__(self, name)
