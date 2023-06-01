from origen.helpers.regressions import cli
from origen.helpers import calling_filename
from pathlib import Path, PurePosixPath
import pytest, re

from .cmd_models import Cmd, CmdArg, CmdOpt, CmdExtOpt
from .cmd_models.auxs import Aux
from .cmd_models.exts import ExtensionDrivers
from .cmd_models.plugins import Plugins
from .error_cases import ErrorCases

from .asertions import AssertionHelpers

develop_origen = "develop_origen"
def develop_origen_cmd():
    return Cmd(
        develop_origen,
        help="Commands to assist with Origen core development",
        aliases=["origen"],
        subcmds=[
            Cmd("build"),
            Cmd("fmt"),
        ]
    )

cli.GlobalCommands.Names.develop_origen = develop_origen
cli.GlobalCommands.develop_origen = develop_origen_cmd()
cli.GlobalCommands.commands.insert(2, cli.GlobalCommands.develop_origen)

cli.InAppCommands.Names.develop_origen = develop_origen
cli.InAppCommands.develop_origen = develop_origen_cmd()
cli.InAppCommands.commands.insert(4, cli.InAppCommands.develop_origen)

def apply_ext_output_args(mod):
    from origen.boot import before_cmd, after_cmd, clean_up
    from .ext_helpers import do_action
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
        split = re.split(r"/plugin\.|/plugin/|/aux_ns\.|/aux_ns/|/core\.|/core/|/app\.|/app/", str(PurePosixPath(calling_filename(frame or 3))))
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
        retn.append(f"Arg: {preface}: No args or opts given!")
    return '\n'.join(retn)

def before_cmd_ext_args_str(ext_args, ext_name=None, frame=None):
    if ext_name is None:
        ext_name = get_ext_name(frame=frame)
    return output_args(f"(Ext) ({ext_name}) (Before Cmd)", ext_args)

def after_cmd_ext_args_str(ext_args, ext_name=None):
    if ext_name is None:
        ext_name = get_ext_name()
    return output_args(f"(Ext) ({ext_name}) (After Cmd)", ext_args)

def clean_up_ext_args_str(ext_args, ext_name=None):
    if ext_name is None:
        ext_name = get_ext_name()
    return output_args(f"(Ext) ({ext_name}) (CleanUp Cmd)", ext_args)

class Configs:
    configs_dir = Path(__file__).parent.joinpath("configs")
    suppress_plugin_collecting_config = configs_dir.joinpath("suppress_plugin_collecting.toml")
    no_plugins_no_aux_cmds_config = configs_dir.joinpath("no_plugins_no_aux_cmds.toml")
    empty_config = configs_dir.joinpath("empty.toml")

class CLIShared(cli.CLI, AssertionHelpers):
    Cmd = Cmd
    error_messages = ErrorCases()
    na = "no_action"

    @pytest.fixture
    def cmd(self):
        return self._cmd

    @pytest.fixture
    def cached_help(self):
        return self.get_cached_help()

    def get_cached_help(self):
        if not hasattr(self, "_cached_help"):
            self._cached_help = self._cmd.get_help_msg()
        return self._cached_help

    @classmethod
    def add_no_pl_aux_cfg(cls, cmd):
        return cmd.extend([], from_configs=cls.configs.no_plugins_no_aux_cmds_config, with_env={"origen_bypass_config_lookup": "1"})

    pln__python_plugin = "python_plugin"

    cmd_shortcuts__default_plugins = {
        "plugin_says_hi": (pln__python_plugin, "plugin_says_hi"),
        "echo": (pln__python_plugin, "echo"),
    }

    plugins = Plugins()
    aux = Aux()
    exts = ExtensionDrivers()
    exts.init_conflicts(plugins, aux)

    python_plugin = plugins.python_plugin
    cmd_testers = aux.namespaces.cmd_testers
    cmd_testers_cmd = cmd_testers.cmd_testers

    configs = Configs()

    project_dir = Path(__file__).parent.parent.parent.parent.parent
    cli_dir = project_dir.joinpath("rust/origen/target/debug")
    test_apps_dir = project_dir.joinpath("test_apps")
    plugins_dir = test_apps_dir # Currently the same but may change if test_apps dir is re-organized