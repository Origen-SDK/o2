from origen.helpers.regressions import cli
from . import Cmd, CmdArg, CmdOpt

class PlExtCmds(cli.CLI):
    def __init__(self):
        self.name = "pl_ext_cmds"

class PythonPlugin(cli.CLI):
    Cmd = Cmd

    def __init__(self):
        self.name = "python_plugin"
        self.python_plugin = self.pl_cmd(
            self.name
        )
        self.echo = self.pl_sub_cmd(
            self.name,
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
        )
        self.plugin_says_hi = self.pl_sub_cmd(
            self.name,
            "plugin_says_hi",
            help="Say 'hi' from the python plugin",
            opts=[
                CmdOpt(
                    name="times",
                    help="Number of times for the python plugin to say",
                    value_name="TIMES",
                    ln="times",
                    sn="t"
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
        )
        self.plugin_test_args = self.pl_sub_cmd(
            self.name,
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
                    ]
                )
            ]
        )
        self.plugin_test_ext_stacking = self.pl_sub_cmd(
            self.name,
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

    @property
    def base_cmd(self):
        return self.python_plugin

    @property
    def ordered_subcmds(self):
        return [
            self.echo,
            "help",
            self.plugin_says_hi,
            self.plugin_test_args,
            self.plugin_test_ext_stacking
        ]

class PythonPluginNoCmds(cli.CLI):
    def __init__(self):
        self.name = "python_plugin_no_cmds"
        self.python_plugin_no_cmds = self.pl_cmd(
            self.name
        )

    @property
    def base_cmd(self):
        return self.python_plugin_no_cmds

class PythonPluginTheSecond(cli.CLI):
    def __init__(self):
        self.name = "python_plugin_the_second"
        self.python_plugin_the_second = self.pl_cmd(
            self.name
        )

    @property
    def base_cmd(self):
        return self.python_plugin_the_second

class TestAppsSharedTestHelpers(cli.CLI):
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
            super
