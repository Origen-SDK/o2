import enum
from pathlib import Path
from origen.helpers.env import run_cli_cmd
from .help_msg import HelpMsg

class CmdArgOpt:
    def to_vn(self):
        return self.value_name or self.name.upper()

class CmdArg(CmdArgOpt):
    def __init__(
        self,
        name,
        help=None,
        multi=False,
        required=False,
        value_name=None,
        use_delimiter=None
    ):
        self.name = name
        self.help = help
        self.multi = multi
        self.required = required
        self.value_name = value_name
        self.use_delimiter = use_delimiter
        self.is_ext = False
        self.is_opt = False
        self.is_arg = True

class CmdOpt(CmdArgOpt):
    def __init__(
        self,
        name,
        help=None,
        takes_value=False,
        multi=False,
        required=False,
        sn=None,
        ln=None,
        sn_aliases=None,
        ln_aliases=None,
        value_name=None,
        hidden = False,
        use_delimiter=None,
        full_name=None,
        access_with_full_name=False,
    ):
        self.name = name
        self.help = help
        self.takes_value = takes_value
        self.multi = multi
        self.required = required
        self.sn = sn
        self.ln = ln
        self.sn_aliases = sn_aliases
        self.ln_aliases = ln_aliases
        self.value_name = value_name
        self.hidden = hidden
        self.use_delimiter = use_delimiter
        self.full_name = full_name
        self.access_with_full_name = access_with_full_name
        self.is_ext = False
        self.is_opt = True
        self.is_arg = False

    def to_ln(self):
        if self.access_with_full_name:
            return self.full_name
        else:
            return self.ln or self.name

    def ln_to_cli(self):
        return f"--{self.to_ln()}"

    def sn_to_cli(self):
        return f"-{self.sn}"

    def sna_to_cli(self, a=0):
        return f"-{self.sn_aliases[a]}"

    def to_cli(self):
        if self.sn:
            return self.sn_to_cli()
        else:
            return self.ln_to_cli()

class SrcTypes(enum.Enum):
    CORE = enum.auto()
    APP = enum.auto()
    PLUGIN = enum.auto()
    AUX = enum.auto()

    def __str__(self) -> str:
        if self == self.CORE:
            return "origen"
        elif self == self.APP:
            return "app"
        elif self == self.PLUGIN:
            return "plugin"
        elif self == self.AUX:
            return "aux"

    def displayed(self, src_name=None):
        if self == SrcTypes.APP:
            return "the App"
        elif self == SrcTypes.PLUGIN:
            return f"plugin '{src_name}'"
        elif self == SrcTypes.AUX:
            return f"aux namespace '{src_name}'"

class CmdExtOpt(CmdOpt):
    @classmethod
    def from_src(cls, src_name, src_type, *args):
        for a in args:
            a.src_name = src_name
            a.src_type = src_type
            if src_type == SrcTypes.APP:
                src_n = ""
            else:
                src_n = f".{src_name}"
            a.full_name = f"ext_opt.{src_type}{src_n}.{a.name}"
        return args

    def __init__(self, *args, src_name=None, src_type=None, **kwargs):
        CmdOpt.__init__(self, *args, **kwargs)
        self.is_ext = True
        self.src_name = src_name
        self.src_type = src_type
    
    @property
    def provided_by_app(self):
        return self.src_type == SrcTypes.APP

    @property
    def provided_by(self):
        if self.provided_by_app:
            return "the App"
        else:
            return self.src_name

    @property
    def displayed(self):
        return self.src_type.displayed(self.src_name)

class CmdDemo:
    def __init__(self, name, args=None, expected_output=None) -> None:
        self.name = name
        self.args = args
        self.expected_output = [expected_output] if isinstance(expected_output, str) else expected_output
        self.parent = None

    def copy(self):
        return self.__class__(
            self.name,
            list(self.args),
            (self.expected_output) if self.expected_output else None,
        )

    def run(self, add_args=None, **kwargs):
        return self.parent.run(*(self.args + (add_args or [])), **kwargs)

    def gen_error(self, add_args=None, **kwargs):
        return self.parent.gen_error(*(self.args + (add_args or [])), **kwargs)

    def assert_present(self, in_str):
        for e in self.expected_output:
            assert e in in_str

class Cmd:
    def __init__(
            self,
            name,
            cmd_path=None,
            help=None,
            args=None,
            opts=None,
            subcmds=None,
            use_configs=None,
            with_env=None,
            demos=None,
            global_demos=None,
            app_demos=None,
            parent=None,
            aliases=None,
            src_type=None,
            prefix_opts=False,
            extendable=True,
            h_opt_idx=None,
            v_opt_idx=None,
            vk_opt_idx=None,
            help_subc_idx=None,
        ):
        from ._origen import to_std_opt
        self.name = name
        self.cmd_path = cmd_path or []
        self.help = help
        self.args = dict([[arg.name, arg] for arg in (args or [])])
        opts = [(to_std_opt(o) if isinstance(o, str) else o) for o in (opts or [])]
        self.opts = dict([[opt.name, opt] for opt in (opts or [])])
        self.subcmds = dict([[subcmd.name, subcmd] for subcmd in (subcmds or [])])
        self.aliases = aliases
        self.exts = None
        self.with_env = with_env
        self.parent = parent
        self.src_type = src_type
        self.prefix_opts = prefix_opts
        if use_configs:
            if not isinstance(use_configs, list):
                use_configs = [use_configs]
            self.use_configs = [Path(c) for c in use_configs]
        else:
            self.use_configs = None
        for subcmd in self.subcmds.values():
            subcmd.parent = self
            subcmd.cmd_path = self.cmd_path + [self.name]

        if self.parent is None:
            self.update_subc()

        self.demos = dict([[d.name, d] for d in (demos or [])])
        for d in self.demos.values(): d.parent = self
        self.global_demos = dict([[d.name, d] for d in (global_demos or [])])
        for d in self.global_demos.values(): d.parent = self
        self.app_demos = dict([[d.name, d] for d in (app_demos or [])])
        for d in self.app_demos.values(): d.parent = self

        self.extendable = extendable
        self.h_opt_idx = h_opt_idx
        self.v_opt_idx = v_opt_idx
        self.vk_opt_idx = vk_opt_idx
        self.help_subc_idx = help_subc_idx

    def replace_subcmds(self, *subcmds):
        self.subcmds = dict([[subcmd.name, subcmd] for subcmd in (subcmds or [])])
        for subcmd in self.subcmds.values():
            subcmd.parent = self
            subcmd.cmd_path = self.cmd_path + [self.name]

        if self.parent is None:
            self.update_subc()

    def update_subc(self):
        for subcmd in self.subcmds.values():
            subcmd.cmd_path = self.cmd_path + [self.name]
            subcmd.use_configs = [
                *(self.use_configs or []),
                *(subcmd.use_configs or [])
            ]
            subcmd.with_env = {
                **(self.with_env or {}),
                **(subcmd.with_env or {})
            }
            subcmd.update_subc()

    def extend(self, exts, with_env=None, from_configs=None):
        dup = self.__class__(
            self.name,
            self.cmd_path,
            self.help,
            self.args.values(),
            self.opts.values(),
            self.subcmds.values(),
            [*self.use_configs] if self.use_configs else None,
            dict(self.with_env) if self.with_env else None,
            [d.copy() for d in self.demos.values()],
            [d.copy() for d in self.global_demos.values()],
            [d.copy() for d in self.app_demos.values()],
            self.parent,
            self.aliases,
            self.src_type,
            self.prefix_opts,
            self.extendable,
            self.h_opt_idx,
            self.v_opt_idx,
            self.vk_opt_idx,
            self.help_subc_idx,
        )
        dup.exts = dict(self.exts) if self.exts else {}
        dup.exts.update(dict([[ext.name, ext] for ext in (exts or [])]))
        if from_configs:
            if isinstance(from_configs, str):
                from_configs = [from_configs]
            elif isinstance(from_configs, Path):
                from_configs = [from_configs]

            if dup.use_configs:
                dup.use_configs += from_configs
            else:
                dup.use_configs = from_configs
        if with_env:
            if dup.with_env:
                dup.with_env.update(with_env)
            else:
                dup.with_env = with_env
        return dup

    def _with_configs_(self, with_configs):
        if self.use_configs:
            if isinstance(with_configs, str):
                with_configs = self.use_configs + [with_configs]
            elif isinstance(with_configs, Path):
                with_configs = self.use_configs + [with_configs]
            else:
                with_configs = self.use_configs + (with_configs or [])
        return with_configs

    def get_help_msg_str(self, with_configs=None, run_opts=None, opts_pre_cmd=None):
        return self.run("help" if self.prefix_opts else "-h", with_configs=with_configs, run_opts=run_opts, pre_cmd_opts=(opts_pre_cmd or self.prefix_opts))

    def get_help_msg(self, with_configs=None, bypass_config_lookup=None, run_opts=None, opts_pre_cmd=None):
        return HelpMsg(self.get_help_msg_str(with_configs=with_configs, run_opts=run_opts, opts_pre_cmd=opts_pre_cmd))

    def run(self, *args, with_env=None, with_configs=None, expect_fail=False, run_opts=None, pre_cmd_opts=None):
        run_opts = dict(run_opts) if run_opts else {}
        a = [(a.to_cli() if isinstance(a, CmdOpt) else a) for a in args]
        if pre_cmd_opts is None or pre_cmd_opts is False:
            pre_cmd_opts = []
        elif pre_cmd_opts is True:
            pre_cmd_opts = a
            a = []

        return run_cli_cmd(
            [*pre_cmd_opts, *self.cmd_path, *([self.name] if self.name else []), *a],
            with_env=run_opts.pop("with_env", None) or with_env or self.with_env,
            with_configs=run_opts.pop("with_configs", None) or self._with_configs_(with_configs),
            expect_fail=run_opts.pop("expect_fail", None) or expect_fail,
            return_details=run_opts.pop("return_details", None) or expect_fail,
            **(run_opts or {}),
        )

    def gen_error(self, *args, with_configs=None, return_stdout=False, return_full=False, run_opts=None, pre_cmd_opts=None):
        out = self.run(
            *args,
            with_configs=with_configs,
            expect_fail=True,
            run_opts=run_opts,
            pre_cmd_opts=pre_cmd_opts,
        )
        if return_full:
            return out
        if return_stdout:
            return out["stdout"]
        else:
            return out["stderr"]

    def __getattr__(self, name: str):
        if hasattr(self, 'args') and (name in self.args):
            return self.args[name]
        elif hasattr(self, 'opts') and (name in self.opts):
            return self.opts[name]
        elif hasattr(self, 'subcmds') and (name in self.subcmds):
            return self.subcmds[name]
        elif hasattr(self, 'exts') and (name in (self.exts or [])):
            return self.exts[name]
        return object.__getattribute__(self, name)

    @property
    def num_args(self):
        return len(self.args)

    @property
    def visible_opts(self):
        return {n:opt for (n, opt) in self.opts.items() if not opt.hidden}

    @property
    def num_opts(self):
        return len(self.visible_opts) + 3 # 3 for standard opts (-h, -v, --vk)
    
    def global_demo(self, name):
        if name in self.global_demos:
            return self.global_demos[name]
        elif name in self.demos:
            return self.demos[name]

    @property
    def is_app_cmd(self):
        return (self.cmd_path[0] == "app") if len(self.cmd_path) > 0 else False

    @property
    def is_pl_cmd(self):
        return (self.cmd_path[0] == "plugin") if len(self.cmd_path) > 0 else False

    @property
    def is_aux_cmd(self):
        return (self.cmd_path[0] == "aux") if len(self.cmd_path) > 0 else False

    @property
    def is_core_cmd(self):
        return not (self.is_app_cmd or self.is_pl_cmd or self.is_aux_cmd)

    @property
    def full_name(self):
        if self.is_app_cmd:
            cp = ('.' + '.'.join(self.cmd_path[2:])) if len(self.cmd_path) > 2 else ''
            return f"app{cp}.{self.name}"
        elif self.is_core_cmd:
            return '.'.join(["origen", *self.cmd_path, self.name])
        else:
            return f"{'.'.join(self.cmd_path)}.{self.name}"

    def reserved_opt_ln_conflict_msg(self, opt, name):
        from .origen import CoreErrorMessages
        return CoreErrorMessages.reserved_opt_ln_conflict_msg(self, opt, name)

    def reserved_opt_lna_conflict_msg(self, opt, name):
        from .origen import CoreErrorMessages
        return CoreErrorMessages.reserved_opt_lna_conflict_msg(self, opt, name)

    def reserved_opt_sn_conflict_msg(self, opt, name):
        from .origen import CoreErrorMessages
        return CoreErrorMessages.reserved_opt_sn_conflict_msg(self, opt, name)

    def reserved_opt_sna_conflict_msg(self, opt, name):
        from .origen import CoreErrorMessages
        return CoreErrorMessages.reserved_opt_sna_conflict_msg(self, opt, name)
