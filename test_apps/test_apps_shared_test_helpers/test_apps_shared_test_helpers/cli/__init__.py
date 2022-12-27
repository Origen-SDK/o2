# FOR_PR clean up

from origen.helpers.regressions import cli
from origen.helpers import calling_filename
from pathlib import Path, PurePosixPath
import re

# from origen.helpers.regressions.cli import CLI

# def run(**args):
#     print(f"Arg Keys: {list(args.keys())}")
#     if len(args) > 0:
#         for n, arg in args.items():
#             print(f"Arg: {n} ({arg.__class__}): {arg}")
#     else:
#         print("No args or opts given!")

def apply_ext_output_args(mod):
    # import inspect
    from origen.boot import before_cmd, after_cmd, clean_up
    from .ext_helpers import do_action
    # print(inspect.stack()[1][0])
    # mod = inspect.getmodule(inspect.stack()[1][0])
    n = get_ext_name()

    def before(**args):
        print(before_cmd_ext_args_str(args, ext_name=n))
        do_action(args.get(f"{n}_action", None), "Before")
    mod.before = before
    before_cmd(mod.before)

    def after(**args):
        print(after_cmd_ext_args_str(args, ext_name=n))
        do_action(args.get(f"{n}_action", None), "After")
    mod.after = after
    after_cmd(mod.after)

    def clean(**args):
        print(clean_up_ext_args_str(args, ext_name=n))
        do_action(args.get(f"{n}_action", None), "CleanUp")
    mod.clean = clean
    clean_up(mod.clean)

def get_ext_name(frame=None):
        split = re.split(r"/plugin\.|/plugin/|/aux_ns\.|/aux_ns/|/core\.|/core/", str(PurePosixPath(calling_filename(frame or 3))))
        if len(split) > 2:
            ext_name = split[-2]
        else:
            ext_name = split[0]
        ext_name = Path(ext_name)
        if ext_name.stem == "extensions":
            ext_name = ext_name.parent.parent
        return ext_name.stem

def output_args(preface, args):
    if preface is None:
        preface = "(CMD)"
    retn = []
    retn.append(f"All Keys: {preface}: {list(args.keys())}")
    if len(args) > 0:
        for n, arg in args.items():
            retn.append(f"Arg: {preface}: {n} ({arg.__class__}): {arg}")
    else:
        retn.append(f"Arg: {preface}: No extension args or opts given!")
    return '\n'.join(retn)

def before_cmd_ext_args_str(ext_args, ext_name=None, frame=None):
    if ext_name is None:
        ext_name = get_ext_name(frame=frame)
    return output_args(f"(Ext) ({ext_name}) (Before Cmd)", ext_args)
    # retn = []
    # retn << f"Ext Arg Keys ({ext_name}) (Before Cmd): {list(ext_args.keys())}"
    # if len(ext_args) > 0:
    #     for n, arg in ext_args.items():
    #         retn << f"Ext Arg ({ext_name}) (Before Cmd): {n} ({arg.__class__}): {arg}"
    # else:
    #     retn << f"({ext_name}) (Before Cmd): No extension args or opts given!"
    # return retn

def after_cmd_ext_args_str(ext_args, ext_name=None):
    if ext_name is None:
        ext_name = get_ext_name()
    return output_args(f"(Ext) ({ext_name}) (After Cmd)", ext_args)

def clean_up_ext_args_str(ext_args, ext_name=None):
    if ext_name is None:
        ext_name = get_ext_name()
    return output_args(f"(Ext) ({ext_name}) (CleanUp Cmd)", ext_args)

# TODO shouldn't be needed anymore
class PrintExtArgs:
    def __init_subclass__(cls) -> None:
        cls.apply_ext_methods()
        return super().__init_subclass__()

    @classmethod
    def apply_ext_methods(cls):
        # print(cls)
        # fail
        from origen.boot import before_cmd, after_cmd, clean_up
        before_cmd(cls.print_args_before)
        after_cmd(cls.print_args_after)
        clean_up(cls.print_args_clean)

    @classmethod
    def print_args_before(cls, **args):
        print(before_cmd_ext_args_str(args))

    @classmethod
    def print_args_after(cls, **args):
        print(after_cmd_ext_args_str(args))

    @classmethod
    def print_args_clean(cls, **args):
        print(clean_up_ext_args_str(args))


# from .. import root
aux_cmds_dir = Path(__file__).parent.parent.joinpath("aux_cmds")

class Cmd(cli.cmd.Cmd):
    # def __init__(self, *args, **kwargs):
    #     cli.cmd.Cmd.__init__(self, *args, **kwargs)

    def assert_args(self, output, *vals):
        ext_args = {}
        args = []
        exp_ext_vals = {}
        cmd_args = []
        cmd_arg = None

        for i, v in enumerate(vals):
            opts = v[1] if isinstance(v[1], dict) else {}
            opts = v[2] if len(v) > 2 else {}

            if isinstance(v[0], CmdExtOpt):
                ext = ext_args.setdefault(v[0].src_name, {})
                if opts.get("Before", True):
                    before = ext.setdefault("Before Cmd", [])
                    if not (("Before" in opts and opts["Before"] is None) or v[1] is None):
                        before.append(v[0].name)
                if opts.get("After", True):
                    after = ext.setdefault("After Cmd", [])
                    if not (("After" in opts and opts["After"] is None) or v[1] is None):
                        after.append(v[0].name)
                if opts.get("CleanUp", True):
                    clean_up = ext.setdefault("CleanUp Cmd", [])
                    if not (("CleanUp" in opts and opts["CleanUp"] is None) or v[1] is None):
                        clean_up.append(v[0].name)
                    # ext_args.setdefault(v[0].src_name, []).append()
            else:
                args.append(v[0])
                cmd_arg = v[0]
            expected = v[0].to_assert_str(v[1], **opts)
            if isinstance(expected, str):
                expected = [expected]

            if isinstance(v[0], CmdExtOpt):
                vals = exp_ext_vals.setdefault(v[0].src_name, [(v[0], None)])
                # if expected == {"Before": {}, "After": {}, "CleanUp": {}}:
                if not (v[1] is None and ("Before" not in opts and "After" not in opts and "CleanUp" not in opts)):
                    if vals[0][1] is None:
                        exp_ext_vals[v[0].src_name] = []
                        vals = exp_ext_vals[v[0].src_name]
                    vals.append((v[0], expected))
            else:
                if v[1] is not None:
                    cmd_args.append(expected)
            # for e in expected:
            #     print(f"expecting: {e}")
            #     assert e in output
        if len(cmd_args) == 0:
            cmd_arg.to_assert_str(None)
        else:
            for exp in cmd_args:
                for e in exp:
                    print(f"expecting: {e}")
                    assert e in output
        for ns, opt in exp_ext_vals.items():
            if len(opt) == 1 and opt[0][1] is None:
                for e in opt[0][0].to_assert_str(None):
                    print(f"expecting: {e}")
                    assert e in output
            else:
                for exp in opt:
                    for e in exp[1]:
                        print(f"expecting: {e}")
                        assert e in output

        actual = self.parse_arg_keys(output)
        assert len(actual) == len(args)
        # assert actual == args
        actual = self.parse_ext_keys(output)
        print(actual)
        print(ext_args)
        assert actual == ext_args

    @classmethod
    def parse_arg_keys(cls, cmd_output):
        return eval(cmd_output.split("All Keys: (CMD):", 1)[1].split("\n")[0])

    @classmethod
    def parse_ext_keys(cls, cmd_output):
        arg_lines = cmd_output.split("All Keys: (Ext) ")
        retn = {}
        for a in arg_lines[1:]:
            a = a.split("\n")[0]
            n, keys = a.split(":", 1)
            n, phase = n.split(") (")
            retn.setdefault(n[1:], {})[phase[0:-1]] = eval(keys)
        return retn

class CmdArgOpt(cli.cmd.CmdArgOpt):
    def to_assert_str(self, vals, **opts):
        if vals is None:
            return f"Arg: (CMD): {self.name}: No extension args or opts given!"
        elif self.multi:
            c = list
            if self.use_delimiter:
                vals = [x for v in vals for x in v.split(',')]
        elif isinstance(vals, int):
            c = int
        else:
            c = str
        return f"Arg: (CMD): {self.name} ({c}): {vals}"
    
    def assert_present(self, vals, in_str, **opts):
        for e in self.to_assert_str(vals, **opts):
            assert e in in_str

class CmdArg(cli.cmd.CmdArg, CmdArgOpt):
    pass

class CmdOpt(cli.cmd.CmdOpt, CmdArgOpt):
    pass

class CmdExtOpt(cli.cmd.CmdExtOpt, CmdArgOpt):
    def to_assert_str(self, vals, **opts):
        if isinstance(vals, dict):
            opts = vals 
        preface = f"Arg: (Ext) ({self.src_name})"

        retn = []
        before_val = opts["Before"] if "Before" in opts else vals
        after_val = opts["After"] if "After" in opts else vals
        cleanup_val = opts["CleanUp"] if "CleanUp" in opts else vals
        if not before_val is False:
            if before_val is None:
                retn.append(f"{preface} (Before Cmd):{CmdArgOpt.to_assert_str(self, before_val).split(':', 3)[3]}")
            else:
                retn.append(f"{preface} (Before Cmd):{CmdArgOpt.to_assert_str(self, before_val).split(':', 2)[2]}")
        if not after_val is False:
            if after_val is None:
                retn.append(f"{preface} (After Cmd):{CmdArgOpt.to_assert_str(self, after_val).split(':', 3)[3]}")
            else:
                retn.append(f"{preface} (After Cmd):{CmdArgOpt.to_assert_str(self, after_val).split(':', 2)[2]}")
        if not cleanup_val is False:
            if cleanup_val is None:
                retn.append(f"{preface} (CleanUp Cmd):{CmdArgOpt.to_assert_str(self, cleanup_val).split(':', 3)[3]}")
            else:
                retn.append(f"{preface} (CleanUp Cmd):{CmdArgOpt.to_assert_str(self, cleanup_val).split(':', 2)[2]}")
        return retn

class ExtensionDrivers:
    exts_workout_cfg = aux_cmds_dir.joinpath("exts_workout_cfg.toml")
    exts_workout_toml = aux_cmds_dir.joinpath("exts_workout.toml")
    pl_ext_stacking_from_aux_cfg = aux_cmds_dir.joinpath("pl_ext_stacking_from_aux_cfg.toml")
    pl_ext_stacking_from_aux_toml = aux_cmds_dir.joinpath("pl_ext_stacking_from_aux.toml")
    core_cmd_exts_cfg = aux_cmds_dir.joinpath("core_cmd_exts_cfg.toml")
    core_cmd_exts_toml = aux_cmds_dir.joinpath("core_cmd_exts.toml")
    exts = {
        "plugin.python_plugin.plugin_test_args": {
            "exts": CmdExtOpt.from_src(
                "exts_workout",
                cli.cmd.SrcTypes.AUX,
                CmdExtOpt(
                    "flag_extension",
                    help="Single flag extension",
                    sn="f",
                    ln="flag_ext",
                ),
                CmdExtOpt(
                    "single_val_opt",
                    takes_value=True,
                    sn="s",
                    help="Extended opt taking a single value",
                ),
                CmdExtOpt(
                    "multi_val_opt",
                    ln="multi",
                    sn_aliases=["m"],
                    ln_aliases=["multi_non_delim"],
                    multi=True,
                    value_name="MULTI_VAL",
                    help="Extended opt taking a multiple, non-delimited values",
                ),
                CmdExtOpt(
                    "multi_val_delim_opt",
                    ln_aliases=["multi_delim"],
                    multi=True,
                    use_delimiter=True,
                    help="Extended opt taking a multiple, delimited values",
                ),
                CmdExtOpt(
                    "exts_workout_action",
                    takes_value=True,
                    required=True,
                    multi=True,
                    help="Additional actions for testing purposes",
                ),
                CmdExtOpt(
                    "hidden_opt",
                    hidden=True,
                    help="Hidden extended opt",
                ),
            ),
            "toml": exts_workout_toml,
        },
        "plugin.python_plugin.plugin_test_args.subc": {
            "exts": CmdExtOpt.from_src(
                "exts_workout",
                cli.cmd.SrcTypes.AUX,
                CmdExtOpt(
                    "exts_workout_action",
                    multi=True,
                    help="Action for the extended opt",
                    ln="action",
                ),
            ),
            "toml": exts_workout_toml,
        },
        "plugin.python_plugin.plugin_test_ext_stacking": {
            "exts": [
                *CmdExtOpt.from_src(
                    "exts_workout",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "ext_action",
                        multi=True,
                        help="Action for the extended opt",
                        ln="action",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "pl_ext_stacking_from_aux",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "pl_ext_stacking_action",
                        multi=True,
                        help="Action from pl_ext_stacking aux cmds",
                    ),
                    CmdExtOpt(
                        "pl_ext_stacking_flag",
                        help="Flag from pl_ext_stacking aux cmds",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "python_plugin_the_second",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "pl_the_2nd_ext_action",
                        help="Action from pl_the_2nd plugin",
                        multi=True,
                    ),
                    CmdExtOpt(
                        "pl_the_2nd_ext_flag",
                        help="Flag from pl_the_2nd plugin",
                    ),
                )
            ]
        },
        "plugin.python_plugin.plugin_test_ext_stacking.subc": {
            "exts": [
                *CmdExtOpt.from_src(
                    "exts_workout",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "ext_action_subc",
                        multi=True,
                        help="Action for the extended opt subc",
                        ln="action",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "pl_ext_stacking_from_aux",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "pl_ext_stacking_action_subc",
                        multi=True,
                        help="Action from pl_ext_stacking aux cmds subc",
                    ),
                    CmdExtOpt(
                        "pl_ext_stacking_flag_subc",
                        help="Flag from pl_ext_stacking aux cmds subc",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "python_plugin_the_second",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "pl_the_2nd_ext_action_subc",
                        help="Action from pl_the_2nd plugin subc",
                        multi=True,
                    ),
                    CmdExtOpt(
                        "pl_the_2nd_ext_flag_subc",
                        help="Flag from pl_the_2nd plugin subc",
                    ),
                )
            ]
        },
        "aux.dummy_cmds.dummy_cmd": {
            "exts": [
                *CmdExtOpt.from_src(
                    "exts_workout",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "exts_workout_action",
                        multi=True,
                        help="Action for the extended opt",
                        ln="action",
                    ),
                    CmdExtOpt(
                        "exts_workout_flag",
                        help="Flag for the extended opt",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "pl_ext_stacking_from_aux",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "pl_ext_stacking_action",
                        multi=True,
                        help="Action from pl_ext_stacking aux cmds",
                    ),
                    CmdExtOpt(
                        "pl_ext_stacking_from_aux_flag",
                        help="Flag from pl_ext_stacking aux cmds",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "python_plugin",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "python_plugin_action",
                        help="Action from python_plugin",
                        multi=True,
                    ),
                    CmdExtOpt(
                        "python_plugin_flag",
                        help="Flag from python_plugin",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "python_plugin_the_second",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "python_plugin_the_second_action",
                        help="Action from pl_the_2nd plugin",
                        multi=True,
                    ),
                    CmdExtOpt(
                        "python_plugin_the_second_flag",
                        help="Flag from pl_the_2nd plugin",
                    ),
                ),
            ],
        },
        "aux.dummy_cmds.dummy_cmd.subc": {
            "exts": [
                *CmdExtOpt.from_src(
                    "exts_workout",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "exts_workout_action",
                        multi=True,
                        help="Action for the extended opt subc",
                        ln="action",
                    ),
                    CmdExtOpt(
                        "exts_workout_flag_subc",
                        help="Flag for the extended opt subc",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "pl_ext_stacking_from_aux",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "pl_ext_stacking_from_aux_action_subc",
                        multi=True,
                        help="Action from pl_ext_stacking aux cmds subc",
                    ),
                    CmdExtOpt(
                        "pl_ext_stacking_from_aux_flag_subc",
                        help="Flag from pl_ext_stacking aux cmds subc",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "python_plugin",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "python_plugin_action_subc",
                        help="Action from python_plugin subc",
                        multi=True,
                    ),
                    CmdExtOpt(
                        "python_plugin_flag_subc",
                        help="Flag from python_plugin subc",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "python_plugin_the_second",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "python_plugin_the_second_action_subc",
                        help="Action from pl_the_2nd plugin subc",
                        multi=True,
                    ),
                    CmdExtOpt(
                        "python_plugin_the_second_flag_subc",
                        help="Flag from pl_the_2nd plugin subc",
                    ),
                ),
            ]
        },
        "generic_core_ext": {
            "exts": [
                *CmdExtOpt.from_src(
                    "pl_ext_cmds",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "pl_ext_cmds_generic_ext",
                        help="Generic ext from pl_ext_cmds plugin",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "core_cmd_exts",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "core_cmd_exts_generic_core_ext",
                        help="Generic core ext from aux commands",
                    ),
                ),
            ]
        }
    }

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
    # python_plugin = PythonPlugin()
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

class AuxCmdsFromCliDir(cli.CLI):
    Cmd = Cmd
    
    def __init__(self):
        self.name = "aux_cmds_from_cli_dir"
        self.aux_cmds_from_cli_dir = self.aux_cmd(
            self.name,
            help="Aux Commands from the Origen CLI directory",
            subcmds = [
                Cmd("cli_dir_says_hi")
            ],
        )
        # self.cli_dir_says_hi = self.aux_sub_cmd(
        #     self.name,
        #     "cli_dir_says_hi",
        # )
    
    @property
    def base_cmd(self):
        return self.aux_cmds_from_cli_dir

class AddAuxCmds(cli.CLI):
    Cmd = Cmd
    
    def __init__(self):
        self.name = "add_aux_cmd"
        self.add_aux_cmd = self.aux_cmd(
            self.name,
            help=None,
        )

    @property
    def base_cmd(self):
        return self.add_aux_cmd

class CmdTesters(cli.CLI):
    Cmd = Cmd
    # cfg_toml

    def __init__(self):
        self.name = "cmd_testers"
        self.cmd_testers = self.aux_cmd(
            self.name,
            help="Commands to assist in testing aux commands when no app is present",
            subcmds=[
                Cmd(
                    "error_cases",
                    help="Commands to test error messages and improper command configuration",
                    subcmds=[
                        Cmd(
                            "missing_impl_dir",
                            subcmds=[
                                Cmd("missing_impl_dir_subc")
                            ]
                        ),
                        Cmd("missing_impl_file"),
                        Cmd("test_missing_run_function"),
                        Cmd("test_exception_in_run"),
                    ]
                ),
                Cmd("python_no_app_tests", help="Test commands for python-no-app workspace"),
                Cmd(
                    "test_arguments",
                    help="Test various argument and option schemes from commands",
                    subcmds=[
                        Cmd("display_verbosity_opts"),
                        Cmd(
                            "no_args_or_opts",
                            help="Command taking no arguments or options"
                        ),
                        Cmd(
                            "optional_arg",
                            help="Command taking a single, optional argument",
                            args=[CmdArg("single_val", "Single value")],
                        ),
                        Cmd(
                            "required_arg",
                            help="Command taking a required and optional arg",
                            args=[
                                CmdArg("required_val", "Single required value", required=True),
                                CmdArg("optional_val", "Single optional value")
                            ],
                        ),
                        Cmd(
                            "multi_arg",
                            help="Command taking a multi-arg",
                            args=[
                                CmdArg("multi_arg", "Multi-arg value", True)
                            ],
                        ),
                        Cmd(
                            "delim_multi_arg",
                            help="Command taking a delimited multi-arg",
                            args=[
                                CmdArg("delim_m_arg", "Delimited Multi-arg value ('multiple' implied)", True)
                            ],
                        ),
                        Cmd(
                            "single_and_multi_arg",
                            help="Command taking a single and multi-arg",
                            args=[
                                CmdArg("single_val", "Single value"),
                                CmdArg("multi_arg", "Multi-arg value", True)
                            ],
                        ),
                        Cmd(
                            "args_with_value_names",
                            help="Single and multi arg with value custom value names",
                            args=[
                                CmdArg("s_arg", "Single value arg with custom value name", value_name="Single Arg Val"),
                                CmdArg("m_arg", "Multi value arg with custom value name", True, value_name="Multi Arg Val")
                            ],
                        ),
                        Cmd(
                            "single_value_optional_opt",
                            help="Command taking optional, single option",
                            opts=[
                                CmdOpt(
                                    name="implicit_single_val",
                                    help='Implicit non-required single value',
                                    takes_value=True,
                                    required=False,
                                ),
                                CmdOpt(
                                    name="explicit_single_val",
                                    help='Explicit non-required single value',
                                    takes_value=True,
                                    required=False,
                                ),
                            ]
                        ),
                        Cmd(
                            "single_value_required_opt",
                            help="Command with single-value optional and required options",
                            opts=[
                                CmdOpt(
                                    name="non_req_val",
                                    help="Non-required single value",
                                    takes_value=True,
                                ),
                                CmdOpt(
                                    name="req_val",
                                    help="Required single value",
                                    takes_value=True,
                                    required=True,
                                ),
                            ]
                        ),
                        Cmd(
                            "multi_opts",
                            help="Command with multi-value optional and required options",
                            opts=[
                                CmdOpt(
                                    name="m_opt",
                                    help="Opt with multiple values",
                                    multi=True,
                                ),
                                CmdOpt(
                                    name="im_m_opt",
                                    help="Opt accepting multiple values were 'takes value' is implied",
                                    multi=True,
                                ),
                                CmdOpt(
                                    name="req_m_opt",
                                    help="Required opt accepting multiple values",
                                    multi=True,
                                    required=True,
                                ),
                                CmdOpt(
                                    name="d_m_opt",
                                    help="Delimited multi opt",
                                    multi=True,
                                ),
                                CmdOpt(
                                    name="d_im_m_opt",
                                    help="Delimited opt where 'multi' and 'takes value' is implied",
                                    multi=True,
                                ),
                            ]
                        ),
                        Cmd(
                            "flag_opts",
                            help="Command with flag-style options only",
                            opts=[
                                CmdOpt(
                                    name="im_f_opt",
                                    help="Stackable flag opt with 'takes value=false' implied",
                                ),
                                CmdOpt(
                                    name="ex_f_opt",
                                    help="Stackable flag opt with 'takes value=false' set",
                                ),
                            ]
                        ),
                        Cmd(
                            "opts_with_value_names",
                            help="Command with single/multi-opts with custom value names",
                            opts=[
                                CmdOpt(
                                    name="s_opt_nv_im_tv",
                                    help="Single opt with value name, implying 'takes_value'=true",
                                    value_name="s_val_impl",
                                ),
                                CmdOpt(
                                    name="s_opt_nv_ex_tv",
                                    help="Single opt with value name and explicit 'takes_value'=true",
                                    value_name="s_val_expl",
                                    takes_value=True,
                                ),
                                CmdOpt(
                                    name="m_opt_named_val",
                                    help="Multi-opt with value name",
                                    value_name="m_val",
                                    multi=True,
                                ),
                                CmdOpt(
                                    name="s_opt_ln_nv",
                                    help="Single opt with long name and value name",
                                    value_name="ln_nv",
                                ),
                            ]
                        ),
                        Cmd(
                            "opts_with_aliases",
                            help="Command with option aliasing, custom long, and short names",
                            opts=[
                                CmdOpt(
                                    name="single_opt",
                                    help="Single opt with long/short name",
                                    takes_value=True,
                                    ln="s_opt",
                                    sn="s"
                                ),
                                CmdOpt(
                                    name="multi_opt",
                                    help="Multi-opt with long/short name",
                                    takes_value=True,
                                    multi=True,
                                    ln="m_opt",
                                    sn="m"
                                ),
                                CmdOpt(
                                    name="occurrence_counter",
                                    help="Flag opt with long/short name",
                                    ln="cnt",
                                    sn="o",
                                ),
                                CmdOpt(
                                    name="flag_opt_short_name",
                                    help="Flag opt with short name only",
                                    sn="f"
                                ),
                                CmdOpt(
                                    name="flag_opt_long_name",
                                    help="Flag opt with long name only",
                                    ln="ln_f_opt"
                                ),
                                CmdOpt(
                                    name="flag_opt_dupl_ln_sn",
                                    help="Flag opt with ln matching another's sn",
                                    ln="f"
                                ),
                                CmdOpt(
                                    name="fo_sn_aliases",
                                    help="Flag opt with short aliases",
                                    sn_aliases=['a', 'b']
                                ),
                                CmdOpt(
                                    name="fo_sn_and_aliases",
                                    help="Flag opt with short name and short aliases",
                                    sn="c",
                                    sn_aliases=['d', 'e']
                                ),
                                CmdOpt(
                                    name="fo_ln_aliases",
                                    help="Flag opt with long aliases",
                                    ln_aliases=['fa', 'fb']
                                ),
                                CmdOpt(
                                    name="fo_ln_and_aliases",
                                    help="Flag opt with long name and long aliases",
                                    ln="fc",
                                    ln_aliases=['fd', 'fe']
                                ),
                                CmdOpt(
                                    name="fo_sn_ln_aliases",
                                    help="Flag opt with long and short aliases",
                                    ln_aliases=['sn_ln_1', 'sn_ln_2'],
                                    sn_aliases=['z'],
                                ),
                            ]
                        ),
                        Cmd(
                            "hidden_opt",
                            help="Command with a hidden opt",
                            opts=[
                                CmdOpt(
                                    name="hidden_opt",
                                    help="Hidden opt",
                                    hidden=True,
                                ),
                                CmdOpt(
                                    # name="non_hidden_opt",
                                    name="visible_opt",
                                    help="Visible, non-hidden, opt",
                                ),
                            ]
                        ),
                    ]
                ),
                Cmd("test_current_command", help="Tests origen.current_command"),
                Cmd(
                    "test_nested_level_1",
                    help="Tests origen.current_command L1",
                    subcmds=[
                        Cmd(
                            "test_nested_level_2",
                            help="Tests origen.current_command L2",
                            subcmds=[
                                Cmd("test_nested_level_3_a", help="Tests origen.current_command L3a"),
                                Cmd("test_nested_level_3_b", help="Tests origen.current_command L3b"),
                            ]
                        )
                    ]
                ),
            ]
        )
        # self.test_arguments = self.aux_sub_cmd(
        #     self.name,
        #     "test_arguments",
        # )
        # self.error_cases = self.aux_sub_cmd(
        #     self.name,
        #     "error_cases"
        # )
        # self.display_cc_verbosity = self.aux_sub_cmd(
        #     self.name,
        #     "display_cc_verbosity"
        # )

    @property
    def base_cmd(self):
        return self.cmd_testers

    @property
    def test_args(self):
        return self.base_cmd.test_arguments

    @property
    def display_v(self):
        return self.test_args.display_verbosity_opts

    @property
    def error_cases(self):
        return self.base_cmd.error_cases

    @property
    def subc_l1(self):
        return self.base_cmd.test_nested_level_1

    @property
    def subc_l2(self):
        return self.subc_l1.test_nested_level_2

    @property
    def subc_l3_a(self):
        return self.subc_l2.test_nested_level_3_a

    @property
    def subc_l3_b(self):
        return self.subc_l2.test_nested_level_3_b


class DummyCmds(cli.CLI):
    Cmd = Cmd
    cfg_toml = aux_cmds_dir.joinpath("dummy_cmds_cfg.toml")

    def __init__(self):
        self.name = "dummy_cmds"
        self.dummy_cmd = self.aux_sub_cmd(
            self.name,
            "dummy_cmd",
            help="Dummy Aux Command",
            args=[
                CmdArg(
                    name="action_arg",
                    help="Dummy Aux Action",
                    multi=True,
                ),
            ],
            subcmds=[
                Cmd(
                    "subc",
                    help="Dummy Aux Subcommand",
                    args=[
                        CmdArg(
                            name="action_arg",
                            help="Dummy Aux Subc Action",
                            multi=True,
                        ),
                    ],
                    opts=[
                        CmdOpt(
                            name="flag_opt",
                            help="Dummy Aux Subc Flag",
                        ),
                    ],
                )
            ],
            from_config=self.cfg_toml
        )

class PythonNoAppAuxCmds(cli.CLI):
    Cmd = Cmd

    def __init__(self):
        self.name = "python_no_app_aux_cmds"
        self.python_no_app_aux_cmds = self.aux_sub_cmd(
            self.name,
            "python_no_app_aux_cmds"
        )
    
    @property
    def base_cmd(self):
        return self.python_no_app_aux_cmds

class AuxNamespaces:
    # dummy_cmds = DummyCmds()

    def __init__(self) -> None:
        self.dummy_cmds = DummyCmds()
        self.cmd_testers = CmdTesters()
        self.python_no_app_aux_cmds = PythonNoAppAuxCmds()
        self.aux_cmds_from_cli_dir = AuxCmdsFromCliDir()
        self.add_aux_cmd = AddAuxCmds()

class Aux:
    namespaces = AuxNamespaces()

    @classmethod
    @property
    def ns(cls):
        return cls.namespaces

class CLIShared(cli.CLI):
    Cmd = Cmd

    pln__python_plugin = "python_plugin"

    cmd_shortcuts__default_plugins = {
        "plugin_says_hi": (pln__python_plugin, "plugin_says_hi"),
        "echo": (pln__python_plugin, "echo"),
    }

    exts = ExtensionDrivers()
    plugins = Plugins()
    aux = Aux()

    @classmethod
    @property
    def python_plugin(cls):
        return cls.plugins.python_plugin

    @classmethod
    @property
    def cmd_testers(cls):
        return cls.aux.namespaces.cmd_testers

    @classmethod
    @property
    def cmd_testers_cmd(cls):
        return cls.cmd_testers.cmd_testers
