from ..shared import CLICommon


class T_Web(CLICommon):
    _cmd = CLICommon.in_app_cmds.web

    def test_help_msg(self, cmd, no_config_run_opts):
        help = cmd.get_help_msg(run_opts=no_config_run_opts)
        help.assert_summary(cmd.help)
        assert set(help.subcmd_names) == {"build", "clean", "help", "serve", "view"}

    def test_build_help_msg(self, cmd, no_config_run_opts):
        help = cmd.build.get_help_msg(run_opts=no_config_run_opts)
        help.assert_summary(cmd.build.help)
        option_names = {opt["long_name"] for opt in help.opts}
        assert {
            "archive",
            "as-release",
            "clean",
            "no-api",
            "release",
            "release-with-warnings",
            "sphinx-args",
            "view",
        }.issubset(option_names)

    def test_serve_help_msg(self, cmd, no_config_run_opts):
        help = cmd.serve.get_help_msg(run_opts=no_config_run_opts)
        help.assert_summary(cmd.serve.help)
        option_names = {opt["long_name"] for opt in help.opts}
        assert {"host", "port", "open", "fast", "sphinx-args"}.issubset(option_names)
