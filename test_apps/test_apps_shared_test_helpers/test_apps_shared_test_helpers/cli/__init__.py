from origen.helpers.regressions import cli
from origen.helpers import calling_filename
from pathlib import Path, PurePosixPath
import re

from .cmd_models import Cmd, CmdArg, CmdOpt, CmdExtOpt
from .cmd_models.aux import Aux
from .cmd_models.exts import ExtensionDrivers
from .cmd_models.plugins import Plugins

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

    python_plugin = plugins.python_plugin
    cmd_testers = aux.namespaces.cmd_testers
    cmd_testers_cmd = cmd_testers.cmd_testers
