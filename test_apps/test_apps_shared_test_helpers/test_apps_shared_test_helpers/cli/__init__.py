from origen.helpers.regressions import cli
from origen.helpers import calling_filename
from pathlib import Path, PurePosixPath
import pytest, re

from .cmd_models import Cmd, CmdArg, CmdOpt, CmdExtOpt
from .cmd_models.auxs import Aux
from .cmd_models.exts import ExtensionDrivers
from .cmd_models.plugins import Plugins

from .asertions import AssertionHelpers

develop_origen = "develop_origen"
def develop_origen_cmd():
    return Cmd(
        develop_origen,
        help="Commands to assist with Origen core development",
        aliases=["origen"],
        subcmds=[
            Cmd("build"),
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

class CLIShared(cli.CLI, AssertionHelpers):
    Cmd = Cmd
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

    # FOR_PR see about moving this into Origen
    @classmethod
    def to_conflict_msg(cls, cmd, conflict):
        if not isinstance(conflict[0], str):
            cmd = conflict[0]
            conflict = conflict[1:]

        type = conflict[0]
        def tname(t, cap=False):
            if t in ["lna", "repeated_lna"]:
                n = "long name alias"
            elif t == "ln":
                n = "long name"
            elif t == "iln":
                n = "inferred long name"
            elif t in ["sna", "repeated_sna"]:
                n = "short name alias"
            elif t == "sn":
                n = "short name"
            else:
                raise RuntimeError(f"Cannot get conflict name from conflict type {t}")
            if cap:
                n = n.capitalize()
            return n

        prefix = f"When processing command '{cmd.full_name}':"
        if type in ["lna", "ln", "sna", "sn", "iln"]:
            with_type = conflict[1]
            offender_opt = conflict[2]
            with_opt = conflict[3]
            if type == "iln":
                if not isinstance(offender_opt, str):
                    c = offender_opt.name
                else:
                    c = offender_opt
            else:
                c = conflict[4]
            if with_opt is None:
                with_opt = offender_opt

            if (not isinstance(offender_opt, str)) and offender_opt.is_ext:
                if with_opt.is_ext:
                    msg = f"{tname(type, True)} '{c}' for extension option '{offender_opt.name}', from {offender_opt.displayed}, conflicts with {tname(with_type, False)} for extension '{with_opt.name}' provided by {with_opt.displayed}"
                else:
                    msg = f"{tname(type, True)} '{c}' for extension option '{offender_opt.name}', from {offender_opt.displayed}, conflicts with {tname(with_type, False)} from command option '{with_opt.name}'"
            else:
                if not isinstance(offender_opt, str):
                    offender_opt = offender_opt.name
                msg = f"{tname(type, True)} '{c}' for command option '{offender_opt}' conflicts with {tname(with_type, False)} from option '{with_opt.name}'"
        elif type in ["inter_ext_sna_sn", "inter_ext_lna_ln", "inter_ext_lna_iln"]:
            offending_opt = conflict[1]
            if type == "inter_ext_sna_sn":
                type = "sna"
                with_type = "sn"
                name = conflict[2]
            elif type == "inter_ext_lna_ln":
                type = "lna"
                with_type = "ln"
                name = conflict[2]
            elif "inter_ext_lna_iln":
                type = "lna"
                with_type = "iln"
                name = offending_opt.name
            if offending_opt.is_ext:
                msg = f"Option '{offending_opt.name}' extended from {offending_opt.displayed} specifies {tname(type, False)} '{name}' but it conflicts with the option's {tname(with_type, False)}"
            else:
                msg = f"Option '{offending_opt.name}' specifies {tname(type, False)} '{name}' but it conflicts with the option's {tname(with_type, False)}"
        elif type in ["repeated_sna", "repeated_lna"]:
            offending_opt = conflict[1]
            if offending_opt.is_ext:
                offending_src = f"extended from {conflict[1].displayed} "
            else:
                offending_src = ''
            name = conflict[2]
            index = conflict[3]
            msg = f"Option '{offending_opt.name}' {offending_src}repeats {tname(type, False)} '{name}' (first occurrence at index {index})"
        elif type == "reserved_prefix_arg_name":
            offending_arg = conflict[1]
            msg = f"Argument '{offending_arg}' uses reserved prefix 'ext_opt'. This option will not be available"
        elif type == "reserved_prefix_opt_name":
            offending_opt = conflict[1]
            offending_src = conflict[2]
            if offending_src is None:
                msg = f"Option '{offending_opt}' uses reserved prefix 'ext_opt'. This option will not be available"
            else:
                msg = f"Option '{offending_opt}' extended from {offending_src} uses reserved prefix 'ext_opt'. This option will not be available"
        elif type in ["reserved_prefix_ln", "reserved_prefix_lna"]:
            offending_opt = conflict[1]
            name = conflict[2]
            if type == "reserved_prefix_ln":
                type = "ln"
            elif type == "reserved_prefix_lna":
                type = "lna"
            if offending_opt.is_ext:
                msg = f"Option '{offending_opt.name}' extended from {offending_opt.displayed} uses reserved prefix 'ext_opt' in {tname(type, False)} '{name}' and will not be available as '--{name}'"
            else:
                msg = f"Option '{offending_opt.name}' uses reserved prefix 'ext_opt' in {tname(type, False)} '{name}' and will not be available as '--{name}'"
        elif type == "self_lna_iln":
            offending_opt = conflict[1]
            msg = f"Option '{offending_opt.name}' extended from {offending_opt.displayed} specifies long name alias '{offending_opt.name}' but it conflicts with the option's inferred long name. If this is intentional, please set this as the option's long name"
        elif type == "duplicate":
            offending_opt = conflict[1]
            index = conflict[2]
            if offending_opt.is_ext:
                msg = f"Option '{offending_opt.name}' extended from {offending_opt.displayed} is already present. Subsequent occurrences will be skipped (first occurrence at index {index})"
            elif offending_opt.is_arg:
                msg = f"Argument '{offending_opt.name}' is already present. Subsequent occurrences will be skipped (first occurrence at index {index})"
            else:
                msg = f"Option '{offending_opt.name}' is already present. Subsequent occurrences will be skipped (first occurrence at index {index})"
        elif type == "intra_cmd_not_placed":
            msg = f"Unable to place unique long name, short name, or inferred long name for command option '{conflict[1]}'. Please resolve any previous conflicts regarding this option or add/update this option's name, long name, or short name"
        elif type == "arg_opt_name_conflict":
            msg = f"Option '{conflict[1].name}' conflicts with Arg of the same name (Arg #{conflict[2]})"
        else:
            raise RuntimeError(f"Unrecognized conflict type {conflict[0]}")
        msg = f"{prefix} {msg}"
        return msg