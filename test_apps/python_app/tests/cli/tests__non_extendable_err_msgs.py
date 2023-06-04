import pytest
from cli.shared import CLICommon
from .core_cmds.target import TargetCLI

class T_NonExtendableErrMsgs(TargetCLI):
    with_env = {"ORIGEN_EXT_TARGET_CMD": "1"}
    _cmd = CLICommon.in_app_commands.target.extend(
        [],
        with_env=with_env,
        from_configs=["config/enumerated_plugins.toml"],
    )
    # TODO CLI have with_env extension apply to subcommands?
    view_cmd = CLICommon.in_app_commands.target.view.extend(
        [],
        with_env=with_env,
        from_configs=["config/enumerated_plugins.toml"],
    )

    @pytest.fixture
    def cmd(self):
        return self._cmd

    err_srcs = [CLICommon.plugins.tas, CLICommon.plugins.python_plugin, CLICommon.app_cmds]

    @classmethod
    def assert_non_ext_errors(cls, out):
        errors = "\n".join(cls.extract_logged_errors(out))
        cls.assert_ext_non_ext_cmd_msg(errors, cls._cmd.view, cls.err_srcs)
        cls.assert_ext_non_ext_cmd_msg(errors, cls._cmd, cls.err_srcs)

    def test_err_msg(self, cmd):
        help = cmd.get_help_msg()
        help.assert_opts(cmd.full_paths, "h", "v", "vk")
        self.assert_non_ext_errors(help.text)

    def test_err_msg_when_run(self, cmd, eagle):
        out = cmd.run()
        self.assert_non_ext_errors(out)
        self.assert_out(out, eagle)

    def test_err_msg_subc(self):
        cmd = self.view_cmd
        help = cmd.get_help_msg()
        help.assert_opts(cmd.full_paths, "h", "v", "vk")
        self.assert_non_ext_errors(help.text)

    def test_subc_err_msg_when_run(self, eagle):
        out = self.view_cmd.run()
        self.assert_non_ext_errors(out)
        self.assert_out(out, eagle)
