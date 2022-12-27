import pytest
from .shared import CLICommon, CmdArg, CmdOpt
Cmd = CLICommon.Cmd

class Common(CLICommon):
    pln = "python_plugin"
    pln_2nd = "python_plugin_the_second"
    pln_no_cmds = "python_plugin_no_cmds"

    pln__cmdn__hi = "plugin_says_hi"
    pln__cmdn__echo = "echo"

    # plugin_subcmds = {
    #     pln: {
    #         pln__cmdn__hi: Cmd(
    #             pln__cmdn__hi,
    #             ["plugin", pln],
    #             help="Say 'hi' from the python plugin",
    #             opts=[
    #                 CmdOpt(
    #                     name="times",
    #                     help="Number of times for the python plugin to say",
    #                     value_name="TIMES",
    #                     ln="times",
    #                     sn="t"
    #                 ),
    #                 CmdOpt(
    #                     name="loudly",
    #                     help="LOUDLY say hi",
    #                     ln="loudly",
    #                     sn="l"
    #                 ),
    #                 CmdOpt(
    #                     name="to",
    #                     help="Specify who should be greeted",
    #                     multi=True,
    #                 )
    #             ]
    #         ),
    #         pln__cmdn__echo: Cmd(
    #             pln__cmdn__echo,
    #             ["plugin", pln],
    #             help="Echos the input",
    #             args=[
    #                 CmdArg(
    #                     name="input",
    #                     help="Input to echo",
    #                     multi=True,
    #                 )
    #             ],
    #             opts=[
    #                 CmdOpt(
    #                     name="repeat",
    #                     help="Echo again (repeat)",
    #                     ln="repeat",
    #                     sn="r"
    #                 )
    #             ]
    #         ),
    #     },
    #     pln_no_cmds: {},
    #     pln_2nd: {},
    # }

    # pl_cmd = Cmd("plugin")

class T_LoadingPluginCmds(Common):
    def test_plugin_cmds_are_added(self):
        # TODO is ordering off here?
        # help = self.pl_cmd.get_help_msg()
        help = self.global_cmds.pl.get_help_msg()
        # help.assert_opt_at(0, e_opt)
        # assert [pl["name"] for pl in help.subcmds] == [
        #     "help",
        #     self.python_plugin.name,
        #     self.plugins.python_plugin_no_cmds.name,
        #     self.plugins.python_plugin_the_second.name
        # ]
        help.assert_subcmds(
            "help",
            self.python_plugin.base_cmd,
            self.plugins.python_plugin_no_cmds.base_cmd,
            self.plugins.python_plugin_the_second.base_cmd
        )

    # def test_plugin_help_msg(self):
    #     fail

    # def test_no_cmds_present_only_has_help_subcmd(self):
    #     help = self.cmds["python_plugin_no_cmds"].get_help_msg()
    #     assert len(help.args) == 0
    #     assert len(help.opts) == 0
    #     assert list(help.subcmds.keys()) == ["help"]

    class Test_PythonPluginCMDs(Common):
        @pytest.fixture
        def root_cmd(self):
            return self.python_plugin.base_cmd
            # return Cmd(self.pln, ["plugin"])

        @pytest.fixture
        def hi_cmd(self):
            return self.python_plugin.plugin_says_hi
            # return self.plugin_subcmds[self.pln][self.pln__cmdn__hi]

        @pytest.fixture
        def echo_cmd(self):
            return self.python_plugin.echo
            # return self.plugin_subcmds[self.pln][self.pln__cmdn__echo]

        @classmethod
        def hi_msg(cls, to=None):
            return f"Hi{(' ' + ','.join(to)) if to else ''} from the python plugin!"

        @classmethod
        def hi_preface(cls, t=1):
            return f"Saying hi {t} time(s)..."

        @classmethod
        def echo_msg(cls, *input):
            return f"Echoing '" + ','.join(input) + "' from python_plugin"

        def test_help_msg(self, root_cmd):
            help = root_cmd.get_help_msg()
            print(help.opts)
            assert len(help.opts) == 3
            help.assert_help_opt_at(0)
            help.assert_vk_opt_at(1)
            help.assert_v_opt_at(2)

            # assert len(help.args) == 0
            help.assert_args(None)
            help.assert_subcmds(*self.python_plugin.ordered_subcmds)
            # assert len(help.subcmds) == 3
            # assert list(help.subcmds.keys()) == [self.pln__cmdn__echo, "help", self.pln__cmdn__hi]

        def test_hi_help_cmd(self, hi_cmd):
            help = hi_cmd.get_help_msg()
            assert len(help.opts) == 6
            help.assert_help_opt_at(0)
            help.assert_vk_opt_at(1)
            help.assert_opt_at(2, hi_cmd.loudly)
            help.assert_opt_at(3, hi_cmd.times)
            help.assert_opt_at(4, hi_cmd.to)
            help.assert_v_opt_at(5)

        def test_py_plugin_says_hi(self, hi_cmd):
            out = hi_cmd.run()
            assert self.hi_preface() in out
            assert out.count(self.hi_msg()) == 1

        def test_py_plugin_says_hi_3_times(self, hi_cmd):
            out = hi_cmd.run(hi_cmd.times.sn_to_cli(), "3")
            assert self.hi_preface(3) in out
            assert out.count(self.hi_msg()) == 3

        def test_py_plugin_says_hi_loudy_to(self, hi_cmd):
            to = ["Scooby", "Shaggy"]
            out = hi_cmd.run("--to", *to, "--loudly")
            assert self.hi_preface() in out
            assert out.count(self.hi_msg(to).upper()) == 1

        def test_py_plugin_echo(self, echo_cmd):
            s = "hello"
            out = echo_cmd.run(s)
            print(out)
            assert out.count(self.echo_msg(s)) == 1

        def test_py_plugin_echo_multi(self, echo_cmd):
            s = ["hello", "there"]
            out = echo_cmd.run(*s)
            assert out.count(self.echo_msg(*s)) == 1

            s = ["hello", "there", "repeated"]
            out = echo_cmd.run(*s, "-r")
            assert out.count(self.echo_msg(*s)) == 2

        def test_py_plugin_echo_delimited(self, echo_cmd):
            s = ["hello", "there", "delimited"]
            out = echo_cmd.run(','.join(s), "--repeat")
            print(out)
            assert out.count(self.echo_msg(*s)) == 2

        @pytest.mark.skip
        def test_nested_cmds(self):
            fail

    # class Test_PythonluginThe2ndCMDs(Common):
    #     pass

    class Test_PythonPluginNoCMDs(Common):
        @pytest.fixture
        def root_cmd(self):
            # return Cmd(self.pln_no_cmds, ["plugin"])
            return self.plugins.python_plugin_no_cmds.base_cmd

        def test_no_cmds_present_only_has_help_subcmd(self, root_cmd):
            # help = self.cmds["python_plugin_no_cmds"].get_help_msg()
            help = root_cmd.get_help_msg()
            help.assert_args(None)
            assert len(help.opts) == 3
            help.assert_subcmds(None)
