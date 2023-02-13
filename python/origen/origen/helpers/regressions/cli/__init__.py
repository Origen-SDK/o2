'''Regression test helpers for testing/checking Origen CLI integration'''

from .command import Cmd, CmdArg, CmdOpt
from .help_msg import HelpMsg
from .origen import _CommonNames, GlobalCommands, InAppCommands, CoreOpts, CoreErrorMessages
from .... import origen as o

cmd = command

class CLIProperties(type):
    @property
    def app_sub_cmd_path(cls):
        cmd = cls.in_app_cmds.app
        return [cmd.name, cmd.commands.name]

class CLI(metaclass=CLIProperties):
    HelpMsg = HelpMsg
    Cmd = Cmd
    CmdOpt = CmdOpt
    CmdArg = CmdArg

    opts = CoreOpts()
    core_opts = opts

    global_commands = GlobalCommands()
    global_cmds = global_commands
    global_core_cmds = global_commands
    global_core_commands = global_commands

    in_app_commands = InAppCommands()
    in_app_cmds = in_app_commands
    in_app_core_cmds = in_app_commands
    in_app_core_commands = in_app_commands

    common_names = _CommonNames()

    @classmethod
    def pl_cmd(cls, plugin, *args, **kwargs):
        return cls.Cmd(plugin, cmd_path=[cls.common_names.pl], *args, **kwargs)

    @classmethod
    def pl_sub_cmd(cls, plugin, name, *args, cmd_path=None, **kwargs):
        return cls.Cmd(name, cmd_path=[cls.common_names.pl, plugin, *(cmd_path or [])], *args, **kwargs)

    @classmethod
    def aux_cmd(cls, namespace, *args, from_config=None, **kwargs):
        return cls.Cmd(namespace, cmd_path=[cls.common_names.aux_cmds], use_configs=from_config, *args, **kwargs)

    @classmethod
    def aux_sub_cmd(cls, namespace, name, *args, cmd_path=None, from_config=None, **kwargs):
        return cls.Cmd(name, cmd_path=[cls.common_names.aux_cmds, namespace, *(cmd_path or [])], use_configs=from_config, *args, **kwargs)

    @classmethod
    def app_sub_cmd(cls, *args, cmd_path=None, **kwargs):
        return cls.Cmd(cmd_path=[*cls.app_sub_cmd_path, *(cmd_path or [])], *args, **kwargs)

    @classmethod
    def eval(cls, *code, **kwargs):
        if o.app:
            cmd = InAppCommands.eval
        else:
            cmd = GlobalCommands.eval
        return cmd.run(*code, **kwargs)

    if o.in_app_context:
        cmds = in_app_cmds
    else:
        cmds = global_cmds


    error_messages = CoreErrorMessages()
    err_msgs = error_messages
