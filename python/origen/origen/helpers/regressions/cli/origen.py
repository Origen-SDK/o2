import origen
from .command import CmdOpt, Cmd, CmdArg, CmdArgOpt, CmdDemo, CmdExtOpt
from ._origen import to_std_opt

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
    exec = "exec"
    fmt = "fmt"
    i = "interactive"
    v = "-v"

    @classmethod
    def aux_cmds_cmd(cls, add_opts=None):
        return Cmd(cls.aux_cmds, help="Interface with auxillary commands")

    @classmethod
    def eval_cmd(cls, add_opts=None):
        return Cmd(
            cls.eval,
            help="Evaluates statements in an Origen context",
            args=[
                CmdArg("code", "Statements to evaluate", multi=True, required=True)
            ],
            opts=(add_opts or []) + [
                CmdOpt(
                    "scripts",
                    help="Evaluate from script files",
                    ln="scripts",
                    sn="s",
                    ln_aliases=["files"],
                    sn_aliases=["f"],
                    multi=True,
                    required=False,
                )
            ],
            h_opt_idx=0,
            v_opt_idx=2,
            vk_opt_idx=3,
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
    def exec_cmd(cls, add_opts=None):
        return Cmd(
            cls.exec,
            help="Execute a command within your Origen/Python environment (e.g. origen exec pytest)",
            args=[
                CmdArg("command", "The command to be run", required=True),
                CmdArg("args", "Arguments to be passed to the command", multi=True, required=False)
            ],
            opts=add_opts,
            prefix_opts=True,
        )

    @classmethod
    def creds_cmd(cls, add_opts=None):
        return Cmd(
            cls.creds,
            help="Set or clear user credentials",
            subcmds=[
                Cmd(
                    "set",
                    help="Set the current user's password",
                    opts=(add_opts or []) + [
                        CmdOpt(
                            "all",
                            help="Set the password for all datasets",
                            ln="all",
                            sn="a",
                            takes_value=False,
                            required=False,
                        ),
                        CmdOpt(
                            "datasets",
                            help="Specify the dataset to set the password for",
                            ln="datasets",
                            sn="d",
                            multi=True,
                            required=False,
                        ),
                    ]
                ),
                Cmd(
                    "clear",
                    help="Clear the user's password",
                    opts=(add_opts or []) + [
                        CmdOpt(
                            "all",
                            help="Clear the password for all datasets",
                            ln="all",
                            sn="a",
                            takes_value=False,
                            required=False,
                        ),
                        CmdOpt(
                            "datasets",
                            help="Specify the dataset to clear the password for",
                            ln="datasets",
                            sn="d",
                            multi=True,
                            required=False,
                        ),
                    ]
                )
            ]
        )

    @classmethod
    def interactive_cmd(cls, add_opts=None):
        return Cmd(
            cls.i,
            help="Start an Origen console to interact with the DUT",
            aliases=['i'],
            opts=add_opts,
        )

    @classmethod
    def pl_cmd(cls, add_opts=None):
        return Cmd(
            cls.pl,
            help="Access added commands from individual plugins",
            aliases=["pl"],
            opts=add_opts,
        )

    @classmethod
    def pls_cmd(cls, add_opts=None):
        return Cmd(
            cls.pls,
            help="Interface with the Origen plugin manager",
            aliases=["pls", "plss"],
            opts=add_opts,
            subcmds=[
                Cmd(
                    "list",
                    help="List the available plugins",
                    aliases=["ls"],
                )
            ]
        )

    @classmethod
    def v_cmd(cls):
        return Cmd(cls.v)

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
        exec = _CommonNames.exec
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
    exec = _CommonNames.exec_cmd()
    aux_cmds = _CommonNames.aux_cmds_cmd()
    pls = _CommonNames.pls_cmd()
    pl = _CommonNames.pl_cmd()
    # proj = Cmd(names.proj)
    new = Cmd(
        names.new,
        help="Create a new origen environment (e.g., app, workspace)",
    )
    creds = _CommonNames.creds_cmd()
    i = _CommonNames.interactive_cmd()
    # fmt = Cmd(names.fmt)
    # build = Cmd(names.build)
    v = _CommonNames.v_cmd()

    commands = [
        # proj, fmt, build
        creds, eval, exec, i, new,
        pls, pl, aux_cmds,
   ]
    cmds = commands

    origen = Cmd("")


class InAppOpts:

    @classmethod
    def all(cls):
        return [cls.targets, cls.no_targets, cls.mode]

    @classmethod
    def standard_opts(self):
        return [CoreOpts.help, self.mode, self.no_targets, self.targets, CoreOpts.verbosity, CoreOpts.vk ]

    mode = to_std_opt("m")
    no_targets = to_std_opt("nt")
    targets = to_std_opt("t")
    to_std_opt = to_std_opt

class InAppCommands(CoreCommands):
    in_app_opts = InAppOpts()

    @classmethod
    def standard_opts(self):
        return self.in_app_opts.standard_opts()

    class Names:
        app = "app"
        aux_cmds = _CommonNames.aux_cmds
        # build = _CommonNames.build
        # compile = "compile"
        creds = _CommonNames.creds
        env = "env"
        eval = _CommonNames.eval
        exec = _CommonNames.exec
        # fmt = _CommonNames.fmt
        generate = "generate"
        i = _CommonNames.i
        # mailer = "mailer"
        # mode = "mode"
        # new = _CommonNames.new
        pl = _CommonNames.pl
        pls = _CommonNames.pls
        # save_ref = "save_ref"
        target = "target"
        # web = "web"
    names = Names()

    class _TargetCmd_:
        @classmethod
        def full_path_opt(cls):
            return CmdOpt(
                "full_paths",
                "Display targets' full paths",
                ln="full-paths",
                sn="f",
                ln_aliases=["full_paths"]
            )
        
        @classmethod
        def targets_arg(cls, help):
            return CmdArg(
                "targets",
                help=help,
                multi=True,
                required=True,
                use_delimiter=True
            )

    app = Cmd(
        names.app,
        help="Manage and interface with the application",
        subcmds=[
            # Cmd(
            #     "checkin",
            #     help="Check in the given pathspecs",
            # ),
            Cmd(
                "commands",
                help="Interface with commands added by the application",
                aliases=["cmds"],
            ),
            # Cmd(
            #     "init",
            #     help="Initialize the application's revision control",
            # ),
            # Cmd(
            #     "package",
            #     help="Build the app into publishable package (e.g., a 'python wheel')",
            # ),
            # Cmd(
            #     "publish",
            #     help="Publish (release) the app",
            # ),
            # Cmd(
            #     "run_publish_checks",
            #     help="Run production-ready and publish-ready checks",
            # ),
            # Cmd(
            #     "status",
            #     help="Show any local changes",
            # ),
        ],
        help_subc_idx=2,
        extendable=False,
    )
    aux_cmds = _CommonNames.aux_cmds_cmd()
    creds = _CommonNames.creds_cmd(add_opts=in_app_opts.all())
    env = Cmd(
        names.env,
        help="Manage your application's Origen/Python environment (dependencies, etc.)",
        subcmds=[
            Cmd(
                "setup",
                help="Setup your application's Python environment for the first time in a new workspace, this will install dependencies per the poetry.lock file",
            ),
            Cmd(
                "update",
                help="Update your application's Python dependencies according to the latest pyproject.toml file",
            ),
        ],
        help_subc_idx=0,
        extendable=False
    )
    eval = _CommonNames.eval_cmd(add_opts=in_app_opts.all())
    exec = _CommonNames.exec_cmd()
    # fmt = Cmd(names.fmt)
    generate = Cmd(
        names.generate,
        help="Generate patterns or test programs",
        args=[CmdArg("files", help="The name of the file(s) to be generated", multi=True, required=True)],
        opts=["m", "nt", "o", "r", "t"],
        h_opt_idx=0,
        vk_opt_idx=7,
        v_opt_idx=6,
    )
    i = _CommonNames.interactive_cmd(add_opts=in_app_opts.all())
    # mailer = Cmd(names.mailer)
    # mode = Cmd(names.mode)
    # new = Cmd(names.new)
    pl = _CommonNames.pl_cmd()
    pls = _CommonNames.pls_cmd()
    # save_ref = Cmd(names.save_ref)
    target = Cmd(
        names.target,
        help="Set/view the default target",
        opts=[_TargetCmd_.full_path_opt()],
        subcmds=[
            Cmd(
                "add",
                help="Activates the given target(s)",
                args=[_TargetCmd_.targets_arg("Targets to be activated")],
                opts=[_TargetCmd_.full_path_opt()],
                aliases=["a"],
            ),
            Cmd(
                "clear",
                help="Deactivates any and all current targets",
                aliases=["c"],
            ),
            Cmd(
                "default",
                help="Activates the default target(s) while deactivating all others",
                opts=[_TargetCmd_.full_path_opt()],
                aliases=["d"],
            ),
            Cmd(
                "remove",
                help="Deactivates the given target(s)",
                args=[_TargetCmd_.targets_arg("Targets to be deactivated")],
                opts=[_TargetCmd_.full_path_opt()],
                aliases=["r"],
            ),
            Cmd(
                "set",
                help="Activates the given target(s) while deactivating all others",
                args=[_TargetCmd_.targets_arg("Targets to be set")],
                opts=[_TargetCmd_.full_path_opt()],
                aliases=["s"],
            ),
            Cmd(
                "view",
                help="Views the currently activated target(s)",
                opts=[_TargetCmd_.full_path_opt()],
                aliases=["v"],
            ),
        ],
        aliases=["w"],
    )
    # web = Cmd(names.web)
    v = _CommonNames.v_cmd()

    commands = [
        # app, aux_cmds, build, compile, creds, env, eval, exec, fmt, generate, i, mailer, mode, new, pl, pls, save_ref, target, web
        app, aux_cmds, creds, env, eval, exec, generate, i, pl, pls, target
    ]
    cmds = commands

    origen = Cmd(None)

class CoreOpts:
    help = CmdOpt('help', "Print help information", sn="h", ln="help")
    verbosity = CmdOpt('verbosity', "Terminal verbosity level e.g. -v, -vv, -vvv", ln="verbose", ln_aliases=["verbosity"], sn="v")
    vk = CmdOpt("verbosity_keywords", "Keywords for verbose listeners", value_name= "verbosity_keywords", takes_value=True, use_delimiter=True, ln_aliases=["vk"])

class CoreErrorMessages:
    @classmethod
    def _invalid_arg_msg(cls, val):
        return f"Found argument '{val}' which wasn't expected, or isn't valid in this context"

    @classmethod
    def _missing_arg_val_msg(cls, arg, type, value_name=None):
        if type == "ln":
            prefix = '--'
        elif type == "sn":
            prefix = '-'
        elif type == "arg":
            prefix = ''
        else:
            raise RuntimeError(f"Cannot generate missing ar val msg for arg type '{type}")
        if isinstance(arg, CmdArgOpt):
            if value_name is None:
                value_name = arg.to_vn()
            arg = arg.name
        return f"The argument '{prefix}{arg} <{value_name or arg}>' requires a value but none was supplied"

    @classmethod
    def missing_arg_val_msg(cls, arg, value_name=None):
        return cls._missing_arg_val_msg(arg, "arg", value_name=value_name)

    @classmethod
    def missing_ln_val_msg(cls, arg, value_name=None):
        return cls._missing_arg_val_msg(arg, "ln", value_name=value_name)

    @classmethod
    def missing_sn_val_msg(cls, arg, value_name=None):
        return cls._missing_arg_val_msg(arg, "sn", value_name=value_name)

    @classmethod
    def too_many_args(cls, val):
        return cls._invalid_arg_msg(val)

    @classmethod
    def unknown_arg_msg(cls, arg):
        return cls._invalid_arg_msg(arg)

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

        return cls._invalid_arg_msg(n)

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
    def invalid_subc_msg(cls, subc):
        return f"The subcommand '{subc}' wasn't recognized\n\nUSAGE:\n"

    @classmethod
    def cmd_building_err_prefix(cls, cmd):
        return f"When processing command '{cmd.full_name}':"

    @classmethod
    def conflict_msg(cls, cmd, opt, conflict, conflict_type):
        if conflict_type in ['long name', 'long name alias']:
            hyphens = "--"
        else:
            hyphens = "-"

        if isinstance(opt, CmdExtOpt):
            return f"{cls.cmd_building_err_prefix(cmd)} Option '{opt.name}' extended from {opt.provided_by} tried to use reserved option {conflict_type} '{conflict}' and will not be available as '{hyphens}{conflict}'"
        else:
            return f"{cls.cmd_building_err_prefix(cmd)} Option '{opt.name}' tried to use reserved option {conflict_type} '{conflict}' and will not be available as '{hyphens}{conflict}'"

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
