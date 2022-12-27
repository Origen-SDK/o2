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
        use_delimiter=None
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

    def to_ln(self):
        return self.ln or self.name

    def ln_to_cli(self):
        return f"--{self.to_ln()}"

    def sn_to_cli(self):
        return f"-{self.sn}"

class SrcTypes(enum.Enum):
    CORE = enum.auto()
    APP = enum.auto()
    PLUGIN = enum.auto()
    AUX = enum.auto()

class CmdExtOpt(CmdOpt):
    @classmethod
    def from_src(cls, src_name, src_type, *args):
        for a in args:
            a.src_name = src_name
            a.src_type = src_type
        return args

    def __init__(self, *args, src_name=None, src_type=None, **kwargs):
        CmdOpt.__init__(self, *args, **kwargs)
        self.src_name = src_name
        self.src_type = src_type

class CmdDemo:
    def __init__(self, name, args=None, expected_output=None) -> None:
        self.name = name
        self.args = args
        self.expected_output = expected_output
        self.parent = None

    def run(self, add_args=None, **kwargs):
        return self.parent.run(*(self.args + (add_args or [])), **kwargs)

    def assert_present(self, in_str):
        assert self.expected_output in in_str

class Cmd:
    def __init__(self, name, cmd_path=None, help=None, args=None, opts=None, subcmds = None, use_configs = None, with_env=None, demos=None, global_demos=None, app_demos=None, parent=None):
        self.name = name
        self.cmd_path = cmd_path or []
        self.help = help
        self.args = dict([[arg.name, arg] for arg in (args or [])])
        self.opts = dict([[opt.name, opt] for opt in (opts or [])])
        self.subcmds = dict([[subcmd.name, subcmd] for subcmd in (subcmds or [])])
        self.exts = None
        self.with_env = with_env
        self.parent = parent
        if use_configs:
            if not isinstance(use_configs, list):
                use_configs = [use_configs]
            self.use_configs = [Path(c) for c in use_configs]
        else:
            self.use_configs = None
        for subcmd in self.subcmds.values():
            subcmd.parent = self
            subcmd.cmd_path = self.cmd_path + [self.name]
            subcmd.use_configs = [*self.use_configs] if self.use_configs else None

        if self.parent is None:
            self.update_cmd_paths()

        self.demos = dict([[d.name, d] for d in (demos or [])])
        for d in self.demos.values(): d.parent = self
        self.global_demos = dict([[d.name, d] for d in (global_demos or [])])
        for d in self.global_demos.values(): d.parent = self
        self.app_demos = dict([[d.name, d] for d in (app_demos or [])])
        for d in self.app_demos.values(): d.parent = self

    def update_cmd_paths(self):
        for subcmd in self.subcmds.values():
            subcmd.cmd_path = self.cmd_path + [self.name]
            subcmd.update_cmd_paths()

    def extend(self, exts, with_env=None, from_configs=None):
        dup = self.__class__(
            self.name,
            self.cmd_path,
            self.help,
            self.args.values(),
            self.opts.values(),
            self.subcmds.values(),
            [*self.use_configs] if self.use_configs else None,
            self.with_env.copy() if self.with_env else None,
            self.demos.values(),
            self.global_demos.values(),
            self.app_demos.values(),
        )
        dup.exts = dict([[ext.name, ext] for ext in (exts or [])])
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

    def get_help_msg_str(self, with_configs=None):
        return self.run("-h", with_configs=with_configs)

    def get_help_msg(self, with_configs=None):
        return HelpMsg(self.get_help_msg_str(with_configs=with_configs))

    def run(self, *args, with_env=None, with_configs=None, expect_fail=False):
        return run_cli_cmd(
            [*self.cmd_path, *([self.name] if self.name else []), *args],
            with_env=with_env or self.with_env,
            with_configs=self._with_configs_(with_configs),
            expect_fail=expect_fail,
            return_details=expect_fail,
        )

    def gen_error(self, *args, with_configs=None, return_stdout=False, return_full=False):
        out = self.run(
            *args,
            with_configs=with_configs,
            expect_fail=True,
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
        elif hasattr(self, 'exts') and (name in self.exts):
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
        print(self.demos)
        print(self.global_demos)
        if name in self.global_demos:
            return self.global_demos[name]
        elif name in self.demos:
            return self.demos[name]
