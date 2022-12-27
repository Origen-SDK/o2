import origen
from .command import CmdOpt, Cmd, CmdArg, CmdDemo

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

class CoreCommands:
    @classmethod
    @property
    def all_names(cls):
        return [cmd.name for cmd in cls.cmds]

    @classmethod
    @property
    def all_names_add_help(cls):
        return cls.all_names + ["help"]

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

    eval = Cmd(
        names.eval,
        help="Evaluates statements in an Origen context",
        args=[
            CmdArg("code", "Statements to evaluate", multi=True, required=True)
        ],
        demos=[CmdDemo(
            "minimal",
            args=["h = 'hi!'; print( origen.version ); print( h )", "print( h )"],
            expected_output=f"{origen.version}\nhi!\nhi!"
        )]
    )
    aux_cmds = Cmd(names.aux_cmds, help="Interface with auxillary commands")
    pls = Cmd(names.pls)
    pl = Cmd(names.pl)
    proj = Cmd(names.proj)
    new = Cmd(names.new)
    creds = Cmd(names.creds)
    i = Cmd(names.i)
    fmt = Cmd(names.fmt)
    build = Cmd(names.build)

    commands = [
        proj, new, creds, eval, i,
        pls, pl, aux_cmds, fmt, build
    ]
    cmds = commands

    origen = Cmd("")

class InAppCommands(CoreCommands):
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
    eval = Cmd(names.eval)
    exec = Cmd(names.exec)
    fmt = Cmd(names.fmt)
    generate = Cmd(names.generate)
    i = Cmd(names.i)
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
    verbosity = CmdOpt('verbosity', "Terminal verbosity level e.g. -v, -vv, -vvv", sn="v")
    vk= CmdOpt("verbosity_keywords", "Keywords for verbose listeners", value_name= "verbosity_keywords", takes_value=True, multi=True, sn="k")

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
