import origen, pytest
from .shared import CLICommon

class T_Exec(CLICommon):
    _cmd= origen.helpers.regressions.cli.CLI.global_cmds.exec

    def test_help_msg(self, cmd):
        help = cmd.get_help_msg()
        help.assert_summary(cmd.help)
        help.assert_args(cmd.command, cmd.args["args"])
        help.assert_opts("h", "v", "vk")
        help.assert_not_extendable()

    def test_without_any_args(self, cmd):
        out = cmd.gen_error()
        self.assert_v(out, None)
        self.assert_args_required_msg(out, cmd.command)

    def test_error_on_usage(self, cmd):
        out = cmd.gen_error(return_full=True, pre_cmd_opts=["-vv", "--vk", "vk_arg", "--blah"])
        self.assert_v(out["stdout"], 2, "vk_arg")
        self.assert_invalid_ln_msg(out["stderr"], "blah")

    def test_verbosity_in_errors(self, cmd):
        out = cmd.gen_error(return_full=True, pre_cmd_opts=["-vv", "--vk", "vk_arg", "--verbose"])
        print(out["stdout"])
        self.assert_v(out["stdout"], 3, ["vk_arg"])
        self.assert_args_required_msg(out["stderr"], cmd.command)

        out = cmd.gen_error(return_full=True, pre_cmd_opts=["-vv", "--vk", "vk0", "--verbose", "--vk", "vk1", "--blah"])
        self.assert_v(out["stdout"], 3, ["vk0", "vk1"])
        self.assert_invalid_ln_msg(out["stderr"], "blah")

        out = cmd.gen_error(return_full=True, pre_cmd_opts=["-b", "-vv", "--vk", "vk_arg"])
        self.assert_v(out["stdout"], 2, ["vk_arg"])
        self.assert_invalid_sn_msg(out["stderr"], "b")

    def test_args_post_cmd_is_part_of_exec_args(self, cmd):
        out = cmd.run("echo", "-h", "-v", "-v")
        assert out == "-h -v -v\n"
        self.assert_v(out, None)

        out = cmd.run("echo", "-v", "--help")
        assert out == "-v --help\n"
        self.assert_v(out, None)

        # -- is needed to interpret long arg names as values and not options
        out = cmd.run("echo", "--", "--vk", "hi", "bye", "-v")
        assert out == "--vk hi bye -v\n"
        self.assert_v(out, None)

    def test_args_pre_cmd_are_applied_to_origen(self, cmd):
        out = cmd.run("echo", "echoing...", "--", "--vk", "hi", "bye", "-v", pre_cmd_opts=["-vvv", "--vk", "vk0,vk1"])
        self.assert_v(out, 3, ["vk0", "vk1"])
        # Remove the closing coloration from the previous line
        assert out.split("\n")[-2].replace("\x1b[0m", '') == "echoing... --vk hi bye -v"
