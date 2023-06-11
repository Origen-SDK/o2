import pytest
from .shared import CLICommon

class Common(CLICommon):
    pass

class T_ArgBuilding(Common):
    @pytest.fixture
    def sv(self):
        return "single_arg_value"

    @pytest.fixture
    def rv(self):
        return "required"

    @pytest.fixture
    def m0(self):
        return "m0"

    @pytest.fixture
    def m1(self):
        return "m1"

    @pytest.fixture
    def m2(self):
        return "m2"

    def test_no_args_available(self):
        cmd = self.cmd_testers.test_args.no_args_or_opts
        help = cmd.get_help_msg()
        help.assert_args(None)
        help.assert_bare_opts()

        out = cmd.run()
        cmd.assert_args(out, None)

    class TestSingleOptionArg(Common):
        _cmd = Common.cmd_testers.test_args.optional_arg

        def test_help_msg(self, cmd):
            help = cmd.get_help_msg()
            help.assert_args(cmd.single_val)
            help.assert_bare_opts

        def test_no_args(self, cmd):
            # Try with no args
            out = cmd.run()
            cmd.assert_args(out, None)

        def test_single_arg_given(self, cmd, sv):
            # Try with single arg given
            out = cmd.run(sv)
            cmd.assert_args(out, (cmd.single_val, sv))

        def test_error_on_two_args_given(self, cmd, sv):
            # Two args should generate error
            another_sv = f"another_{sv}"
            out = cmd.gen_error(sv, another_sv)
            assert self.err_msgs.too_many_args(another_sv) in out

    class TestSingleRequiredArg(Common):
        _cmd = Common.cmd_testers.test_args.required_arg

        def test_help_msg(self, cmd):
            help = cmd.get_help_msg()
            help.assert_args(cmd.required_val, cmd.optional_val)
            help.assert_bare_opts()

        def test_error_on_missing_required_arg(self, cmd):
            # No args should generate error
            out = cmd.gen_error()
            assert self.err_msgs.missing_required_arg(cmd.required_val) in out

        def test_req_arg_given(self, cmd, rv):
            # Single required arg
            out = cmd.run(rv)
            cmd.assert_args(out, (cmd.required_val, rv))

        def test_req_and_optional_arg_given(self, cmd, rv):
            # Required and optional arg
            ov = "optional"
            out = cmd.run(rv, ov)
            cmd.assert_args(out, (cmd.required_val, rv), (cmd.optional_val, ov))

    class TestMultiArg(Common):
        _cmd = Common.cmd_testers.test_args.multi_arg

        def test_help_msg(self, cmd):
            help = cmd.get_help_msg()
            help.assert_args(cmd.multi_arg)
            help.assert_bare_opts()

        def test_no_args(self, cmd):
            # Try with no args
            out = cmd.run()
            cmd.assert_args(out, None)

        def test_one_arg(self, cmd, m0):
            # Try with one arg
            out = cmd.run(m0)
            cmd.assert_args(out, (cmd.multi_arg, [m0]))

        def test_three_args(self, cmd, m0, m1, m2):
            # Try with three args
            out = cmd.run(m0, m1, m2)
            cmd.assert_args(out, (cmd.multi_arg, [m0, m1, m2]))

        def test_delimiter_is_not_default(self, cmd, m0, m1, m2):
            # No delimiter by default, so this should all look like a single value
            out = cmd.run(f"{m0},{m1},{m2}")
            cmd.multi_arg.to_assert_str([f"{m0},{m1},{m2}"]) in out

    class TestDelimitedMultiArg(Common):
        _cmd = Common.cmd_testers.test_args.delim_multi_arg

        def test_help_msg(sel, cmd):
            help = cmd.get_help_msg()
            help.assert_args(cmd.delim_m_arg)
            help.assert_bare_opts()

        def test_no_args(self, cmd):
            # Try with no args
            out = cmd.run()
            cmd.assert_args(out, None)

        def test_one_arg(self, cmd, m0):
            # Try with one arg
            out = cmd.run(m0)
            cmd.assert_args(out, (cmd.delim_m_arg, [m0]))

        def test_three_non_delimited_args(self, cmd, m0, m1, m2):
            # Try with three args
            out = cmd.run(m0, m1, m2)
            cmd.assert_args(out, (cmd.delim_m_arg, [m0, m1, m2]))

        def test_three_delimited_args(self, cmd, m0, m1, m2):
            # Use delimiter to split up values
            out = cmd.run(f"{m0},{m1},{m2}")
            cmd.assert_args(out, (cmd.delim_m_arg, [m0, m1, m2]))

    class TestArgsWithValueNames(Common):
        _cmd = Common.cmd_testers.test_args.args_with_value_names

        def test_help_msg(self, cmd):
            help = cmd.get_help_msg()
            help.assert_args(cmd.s_arg, cmd.m_arg)
            help.assert_bare_opts()

        def test_value_names(self, cmd):
            sv ="single_val"
            m0 = "m0_val"
            m1 = "m1_val"
            out = cmd.run(sv, m0, m1)
            cmd.assert_args(out, (cmd.s_arg, sv), (cmd.m_arg, [m0, m1]))

    class TestCombinedSingleAndMultiArgs(Common):
        _cmd = Common.cmd_testers.test_args.single_and_multi_arg

        def test_help_msg(self, cmd):
            help = cmd.get_help_msg()
            help.assert_args(cmd.single_val, cmd.multi_arg)
            help.assert_bare_opts()

        def test_no_args(self, cmd):
            # Try no args
            out = cmd.run()
            cmd.assert_args(out, None)

        def test_single__arg(self, cmd, sv):
            # Try single arg only
            out = cmd.run(sv)
            cmd.assert_args(out, (cmd.single_val, sv))

        def test_two_args(self, cmd, sv, m0):
            # Try two args
            out = cmd.run(sv, m0)
            cmd.assert_args(out, (cmd.single_val, sv), (cmd.multi_arg, [m0]))

        def test_three_args(self, cmd, sv, m0, m1):
            # Try three args
            out = cmd.run(sv, m0, m1)
            cmd.assert_args(out, (cmd.single_val, sv), (cmd.multi_arg, [m0, m1]))
