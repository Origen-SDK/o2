import pytest, shutil, os
from origen.helpers.env import run_cli_cmd
from origen.helpers.regressions import cli
# from origen.helpers.regressions.cli import cmd
# from origen.helpers.regressions.cli import CLI
# from origen.helpers.regressions.cli import Cmd as CmdBase
# from origen.helpers.regressions.cli import CmdArgOpt as CmdArgOptBase
from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg

# class Cmd(cli.cmd.Cmd):
#     # def __init__(self, name, help=None, args=None, opts=None):
#     #     self.name = name
#     #     self.help = help
#     #     self.args = dict([[arg.name, arg] for arg in (args or [])])
#     #     self.opts = dict([[opt.name, opt] for opt in (opts or [])])
#     def __init__(self, *args, **kwargs):
#         cli.cmd.Cmd.__init__(self, *args, **kwargs)

#     # def get_help_msg_str(self):
#     #     return CLICommon.run_test_args_cmd_help(self.name)

#     # def get_help_msg(self):
#     #     return CLICommon.HelpMsg(self.get_help_msg_str())

#     # def run(self, *args):
#     #     return CLICommon.run_test_args_cmd(self.name, *args)

#     # def gen_error(self, *args):
#     #     return CLICommon.run_test_args_cmd(self.name, *args, expect_fail=True)["stderr"]

#     # def __getattr__(self, name: str):
#     #     if hasattr(self, 'args') and (name in self.args):
#     #         return self.args[name]
#     #     elif hasattr(self, 'opts') and (name in self.opts):
#     #         return self.opts[name]
#     #     return object.__getattribute__(self, name)

#     @classmethod
#     def parse_arg_keys(cls, cmd_output):
#         return eval(cmd_output.split("Arg Keys: ", 1)[1].split("\n")[0])

# class CmdArgOpt(cli.cmd.CmdArgOpt):
#     def to_assert_str(self, vals):
#         if self.multi:
#             c = list
#         elif isinstance(vals, int):
#             c = int
#         else:
#             c = str
#         return f"Arg: {self.name} ({c}): {vals}"

# class CmdArg(cli.cmd.CmdArg, CmdArgOpt):
#     pass

# class CmdOpt(cli.cmd.CmdOpt, CmdArgOpt):
#     pass

class CLICommon(CLIShared):
    # cmdn__cmd_testers = "cmd_testers"
    # cmdn__test_args = "test_arguments"
    # cmdn__error_cases = "error_cases"

    # cmd_base = [cli.CLI.common_names.aux_cmds, cmdn__cmd_testers]
    # arg_cmd_base = [*cmd_base, "test_arguments"]
    # verbosity_cmd = [*cmd_base, "display_cc_verbosity"]

    # Custom message from testing args/opts.
    no_args_or_opts_msg = "No args or opts given!"

    # cmd_testers_cmd = cli.CLI.aux_sub_cmd(cmdn__cmd_testers)

    # @classmethod
    # def cmd_testers_sub_cmd(cls, *args, cmd_path=None, **kwargs):
    #     return cls.aux_sub_cmd(
    #         cmd_path=[cls.cmdn__cmd_testers, *(cmd_path or [])],
    #         *args,
    #         **kwargs
    #     )

    # @classmethod
    # def test_args_sub_cmd(cls, *args, cmd_path=None, **kwargs):
    #     return cls.aux_sub_cmd(
    #         cmd_path=[cls.cmdn__cmd_testers, cls.cmdn__test_args, *(cmd_path or [])],
    #         *args,
    #         **kwargs
    #     )

    # @classmethod
    # @property
    # def err_cases_cmd(cls):
    #     return cls.cmd_testers_sub_cmd(cls.cmdn__error_cases)

    # @classmethod
    # def run_test_args_cmd(cls, cmd, *args, expect_fail=False):
    #     return run_cli_cmd(
    #         [*cls.arg_cmd_base, cmd, *args],
    #         expect_fail=expect_fail,
    #         return_details=expect_fail
    #     )

    # @classmethod
    # def run_test_args_cmd_help(cls, cmd):
    #     return run_cli_cmd([*cls.arg_cmd_base, cmd, "-h"])

    # @classmethod
    # def run_display_verbosity_cmd(cls, *args):
    #     return run_cli_cmd(cls.verbosity_cmd + list(args))

    # @staticmethod
    # def assert_arg(arg, sn=None, ln=None, value_name=None, help=None, short_aliases=None, long_aliases=None):
    #     if sn is not False:
    #         assert arg["short_name"] == sn
    #     if ln is not False:
    #         assert arg["long_name"] == ln
    #     if value_name is not False:
    #         if value_name is None:
    #             assert arg["value_name"] is None
    #             assert arg["multiple_values"] is None
    #         else:
    #             assert arg["value_name"] == value_name[0]
    #             assert arg["multiple_values"] == value_name[1]
    #     if help is not False:
    #         assert arg["help"] == help
    #     if short_aliases is not False:
    #         assert arg['short_aliases'] == short_aliases
    #     if long_aliases is not False:
    #         assert arg['long_aliases'] == long_aliases

    # @classmethod
    # def assert_help_arg(cls, arg):
    #     return cls.assert_arg(arg, 'h', "help", None, "Print help information")
    
    # @classmethod
    # def assert_verbose_arg(cls, arg):
    #     return cls.assert_arg(arg, 'v', None, None, "Terminal verbosity level e.g. -v, -vv, -vvv")
    
    # @classmethod
    # def assert_vk_arg(cls, arg):
    #     return cls.assert_arg(arg, 'k', None, ("verbosity_keywords", True), "Keywords for verbose listeners")

    @pytest.fixture
    def with_cli_aux_cmds(self):
        shutil.copy(self.dummy_config, self.cli_config)
        shutil.copy(self.cli_aux_cmds_toml, self.cli_dir)
        shutil.copytree(self.cli_aux_cmds_impl, self.cli_dir.joinpath("aux_cmds_from_cli_dir"), dirs_exist_ok=True)
        yield
        os.remove(self.cli_config)
        os.remove(self.cli_dir.joinpath("aux_cmds_from_cli_dir.toml"))
        shutil.rmtree(self.cli_dir.joinpath("aux_cmds_from_cli_dir"))

    @classmethod
    def parse_subcmd_help_dialogue(cls, msg_str):
        return CLICommon.HelpMsg(msg_str)


# class AuxCmd():
#     ...


