"""Functional coverage for top-level command shortcuts.

Clap 3's experimental ``Command::replace`` expanded a shortcut token into a
nested command path. Clap 4 removed that API, so the CLI normalizes argv itself
before parsing. The rest of the suite only asserts that shortcuts are *listed*
in help output, which would not notice the expansion breaking, so these tests
exercise it end to end.
"""

from .shared import CLICommon


class TestCmdShortcuts(CLICommon):
    def test_plugin_shortcut_matches_the_full_command_path(self):
        shortcut = self.run_cli_cmd(["plugin_says_hi"])
        full = self.run_cli_cmd(["plugin", "python_plugin", "plugin_says_hi"])
        assert "Hi from the python plugin!" in shortcut
        assert shortcut == full

    def test_app_shortcut_matches_the_full_command_path(self):
        shortcut = self.run_cli_cmd(["y", "-h"])
        full = self.run_cli_cmd([*CLICommon.app_sub_cmd_path, "playground", "-h"])
        assert shortcut == full

    def test_aux_shortcut_matches_the_full_command_path(self):
        shortcut = self.run_cli_cmd(["dummy_cmd", "-h"])
        full = self.run_cli_cmd(
            ["auxillary_commands", "dummy_cmds", "dummy_cmd", "-h"]
        )
        assert shortcut == full

    def test_shortcut_expands_after_a_valueless_global_flag(self):
        # 'origen -v <full command path>' has always worked, so the shortcut
        # form has to accept the same leading flags.
        assert "Hi from the python plugin!" in self.run_cli_cmd(["-v", "plugin_says_hi"])
        assert "Hi from the python plugin!" in self.run_cli_cmd(["-vv", "plugin_says_hi"])

    def test_shortcut_is_not_expanded_when_it_is_an_option_value(self):
        # '-t' consumes the next token as a target name. Expanding it there
        # would turn a value into a command.
        out = self.gen_error(["-t", "plugin_says_hi"])
        assert "Hi from the python plugin!" not in out


class TestAppCommandArgDefinitions(CLICommon):
    def test_app_command_with_positional_args_builds(self):
        # Regression: clap 4 asserts on positional layouts that clap 3 silently
        # accepted, which crashed the CLI instead of reporting a bad definition.
        help = self.run_cli_cmd(["y", "-h"])
        assert "ERROR" not in help
        assert "panicked" not in help
        assert "<MYARG>" in help
