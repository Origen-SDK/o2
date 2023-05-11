import pytest
from .shared import CLICommon

class Common(CLICommon):
    pass

class T_LoadingPluginCmds(Common):
    class Test_PythonPluginCMDs(Common):
        @pytest.fixture
        def root_cmd(self):
            return self.python_plugin.base_cmd

        @pytest.fixture
        def hi_cmd(self):
            return self.python_plugin.plugin_says_hi

        @pytest.fixture
        def echo_cmd(self):
            return self.python_plugin.echo

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
            help.assert_bare_opts()

            help.assert_args(None)
            help.assert_subcmds(*self.python_plugin.ordered_subcmds)
            help.assert_not_extendable()

        def test_help_on_no_subcmd_given(self, root_cmd):
            out = root_cmd.gen_error()
            assert out == root_cmd.get_help_msg().text

        def test_hi_help_cmd(self, hi_cmd):
            help = hi_cmd.get_help_msg()
            help.assert_opts(
                "h",
                hi_cmd.loudly, hi_cmd.to,
                "v", "vk",
                hi_cmd.times,
            )

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
            assert out.count(self.echo_msg(*s)) == 2

        @pytest.mark.skip
        def test_nested_cmds(self):
            fail

    class Test_PythonPluginNoCMDs(Common):
        @pytest.fixture
        def root_cmd(self):
            return self.plugins.python_plugin_no_cmds.base_cmd

        def test_no_cmds_in_plugin(self, root_cmd):
            help = root_cmd.get_help_msg()
            help.assert_args(None)
            help.assert_bare_opts()
            help.assert_subcmds(None)
            help.assert_not_extendable()
