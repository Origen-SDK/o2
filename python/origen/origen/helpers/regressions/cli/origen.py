import origen
from .command import CmdOpt, Cmd, CmdArg, CmdDemo, CmdExtOpt

def help_subcmd():
    return Cmd("help", help="Print this message or the help of the given subcommand(s)")

class _CommonNames:
    aux_cmds = "auxillary_commands"
    pls = "plugins"
    pl = "plugin"
    build = "build"
    creds = "credentials"
    new = "new"
    eval = "eval"
    fmt = "fmt"
    i = "interactive"

    @classmethod
    def eval_cmd(cls, add_opts=None):
        return Cmd(
            cls.eval,
            help="Evaluates statements in an Origen context",
            args=[
                CmdArg("code", "Statements to evaluate", multi=True, required=True)
            ],
            opts=add_opts,
            demos=[
                CmdDemo(
                    "minimal",
                    args=["print( 'hello from eval cmd!' )"],
                    expected_output="hello from eval cmd"
                ),
                CmdDemo(
                    "multi_statement_single_arg",
                    args=["h = 'hi!'; print( origen.version ); print( h ); print( h )"],
                    expected_output=f"{origen.version}\nhi!\nhi!"
                ),
                CmdDemo(
                    "multi_statement_multi_args",
                    args=[
                        "h = 'hello!'",
                        "print( origen.version )",
                        "print( h )",
                        "print( h )"
                    ],
                    expected_output=f"{origen.version}\nhello!\nhello!"
                ),
                CmdDemo(
                    "gen_name_error",
                    args=["print( missing )"],
                    expected_output=["Traceback (most recent call last):", "NameError: name 'missing' is not defined"]
                )
            ]
        )

    @classmethod
    def interactive_cmd(cls, add_opts=None):
        return Cmd(
            cls.i,
            help="Start an Origen console to interact with the DUT",
            aliases=['i'],
        )

# Use this to mimic:
#  @classmethod
#  @property
# Available in Python 3.9+
class CoreCommandsProperties(type):
    @property
    def all_names(cls):
        return [cmd.name for cmd in cls.cmds]

    @property
    def all_names_add_help(cls):
        return cls.all_names + ["help"]

class CoreCommands(metaclass=CoreCommandsProperties):
    # Use this to mimic:
    #  @classmethod
    #  @property
    # Available in Python 3.9+
    def __getattr__(self, name=None):
        if name == "all_names_add_help":
            return self.__class__.all_names + ["help"]
        elif name == "all_names":
            return [cmd.name for cmd in self.__class__.cmds]
        else:
            self.__getattribute__(name)

class GlobalCommands(CoreCommands):
    class Names:
        eval = _CommonNames.eval
        aux_cmds = _CommonNames.aux_cmds
        pls = _CommonNames.pls
        pl = _CommonNames.pl

        proj = "proj"
        new = _CommonNames.new
        creds = _CommonNames.creds
        i = _CommonNames.i
        fmt = _CommonNames.fmt
        build = _CommonNames.build
    names = Names()

    eval = _CommonNames.eval_cmd()
    aux_cmds = Cmd(names.aux_cmds, help="Interface with auxillary commands")
    pls = Cmd(names.pls)
    pl = Cmd(names.pl)
    proj = Cmd(names.proj)
    new = Cmd(names.new)
    creds = Cmd(names.creds)
    i = _CommonNames.interactive_cmd()
    fmt = Cmd(names.fmt)
    build = Cmd(names.build)

    commands = [
        proj, new, creds, eval, i,
        pls, pl, aux_cmds, fmt, build
    ]
    cmds = commands

    origen = Cmd("")

class InAppOpts:
    targets = CmdOpt(
        "targets",
        help="Override the targets currently set by the workspace for this command",
        takes_value=True,
        multi=True,
        use_delimiter=True,
        ln="targets",
        ln_aliases=["target"],
        sn="t",
    )
    no_targets = CmdOpt(
        "no_targets",
        help="Clear any targets currently set by the workspace for this command",
        takes_value=False,
        ln_aliases=["no_target"],
    )
    mode = CmdOpt(
        "mode",
        help="Override the default mode currently set by the workspace for this command",
        takes_value=True,
        multi=False,
        ln="mode",
    )

    @classmethod
    def all(cls):
        return [cls.targets, cls.no_targets, cls.mode]

    @classmethod
    def standard_opts(self):
        return [CoreOpts.help, self.mode, self.no_targets, self.targets, CoreOpts.verbosity, CoreOpts.vk ]

class InAppCommands(CoreCommands):
    in_app_opts = InAppOpts()

    @classmethod
    def standard_opts(self):
        return self.in_app_opts.standard_opts()

    class Names:
        app = "app"
        aux_cmds = _CommonNames.aux_cmds
        build = _CommonNames.build
        compile = "compile"
        creds = _CommonNames.creds
        env = "env"
        eval = _CommonNames.eval
        exec = "exec"
        fmt = _CommonNames.fmt
        generate = "generate"
        i = _CommonNames.i
        mailer = "mailer"
        mode = "mode"
        new = _CommonNames.new
        pl = _CommonNames.pl
        pls = _CommonNames.pls
        save_ref = "save_ref"
        target = "target"
        web = "web"
    names = Names()

    app = Cmd(names.app, subcmds=[Cmd("commands")])
    aux_cmds = Cmd(names.aux_cmds)
    build = Cmd(names.build)
    compile = Cmd(names.compile)
    creds = Cmd(names.creds)
    env = Cmd(names.env)
    eval = _CommonNames.eval_cmd(add_opts=in_app_opts.all())
    exec = Cmd(names.exec)
    fmt = Cmd(names.fmt)
    generate = Cmd(names.generate)
    i = _CommonNames.interactive_cmd(add_opts=in_app_opts.all())
    mailer = Cmd(names.mailer)
    mode = Cmd(names.mode)
    new = Cmd(names.new)
    pl = Cmd(names.pl)
    pls = Cmd(names.pls)
    save_ref = Cmd(names.save_ref)
    target = Cmd(names.target)
    web = Cmd(names.web)

    commands = [
        app, aux_cmds, build, compile, creds, env, eval, exec, fmt, generate, i, mailer, mode, new, pl, pls, save_ref, target, web
    ]
    cmds = commands

    origen = Cmd(None)

class CoreOpts:
    help = CmdOpt('help', "Print help information", sn="h", ln="help")
    verbosity = CmdOpt('verbosity', "Terminal verbosity level e.g. -v, -vv, -vvv", ln="verbosity", sn="v")
    vk= CmdOpt("verbosity_keywords", "Keywords for verbose listeners", value_name= "verbosity_keywords", takes_value=True, multi=True, ln_aliases=["vk"])

class CoreErrorMessages:
    @classmethod
    def too_many_args(cls, val):
        return f"Found argument '{val}' which wasn't expected, or isn't valid in this context"

    @classmethod
    def unknown_opt_msg(cls, opt, ln=True):
        if ln:
            prefix = "--"
        else:
            prefix = "-"

        if isinstance(opt, CmdOpt):
            n = f"{prefix}{opt.name}"
        else:
            n = f"{prefix}{opt}"

        return f"Found argument '{n}' which wasn't expected, or isn't valid in this context"

    @classmethod
    def missing_required_arg(cls, *vals):
        mapped_vals = []
        for v in vals:
            if isinstance(v, CmdOpt):
                mapped_vals.append(f"{v.ln_to_cli()} <{v.to_vn()}>")
            else:
                mapped_vals.append(f"<{v.to_vn()}>")
        return "The following required arguments were not provided:" + "\n    " + "    \n".join(mapped_vals)

    @classmethod
    def conflict_msg(cls, cmd, opt, conflict, conflict_type):
        if conflict_type in ['long name', 'long name alias']:
            hyphens = "--"
        else:
            hyphens = "-"

        if isinstance(opt, CmdExtOpt):
            return f"Option '{opt.name}' extended from '{opt.provided_by}' for command '{cmd.full_name}' tried to use reserved option {conflict_type} '{conflict}' and will not be available as '{hyphens}{conflict}'"
        else:
            return f"Option '{opt.name}' from command '{cmd.full_name}' tried to use reserved option {conflict_type} '{conflict}' and will not be available as '{hyphens}{conflict}'"

    @classmethod
    def reserved_opt_ln_conflict_msg(cls, cmd, opt, conflict):
        return cls.conflict_msg(cmd, opt, conflict, "long name")

    @classmethod
    def reserved_opt_sn_conflict_msg(cls, cmd, opt, conflict):
        return cls.conflict_msg(cmd, opt, conflict, "short name")

    @classmethod
    def reserved_opt_lna_conflict_msg(cls, cmd, opt, conflict):
        return cls.conflict_msg(cmd, opt, conflict, "long name alias")

    @classmethod
    def reserved_opt_sna_conflict_msg(cls, cmd, opt, conflict):
        return cls.conflict_msg(cmd, opt, conflict, "short name alias")
