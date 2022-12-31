import pytest
from .shared import CLICommon

class Common(CLICommon):
    cmdn__no_args_or_opts = "no_args_or_opts"
    cmdn__optional_arg = "optional_arg"
    cmdn__required_arg = "required_arg"
    cmdn__multi_arg = "multi_arg"
    cmdn__delim_multi_arg = "delim_multi_arg"
    cmdn__single_and_multi_arg = "single_and_multi_arg"
    cmdn__args_with_value_names = "args_with_value_names"

    @pytest.fixture
    def cmd(self):
        return getattr(self.cmd_testers.test_args, self.cmdn)

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
        # cmd = self.cmds[self.cmdn__no_args_or_opts]
        cmd = self.cmd_testers.test_args.no_args_or_opts
        help = cmd.get_help_msg()
        # # help = self.parse_subcmd_help_dialogue(self.run_opt_cmd_help(self.cmd__sv_opt_arg))
        # assert len(help.args) == 0
        # assert len(help.opts) == 3
        help.assert_num_args(0)
        help.assert_num_opts(3)

        help.assert_help_opt_at(0)
        help.assert_vk_opt_at(1)
        help.assert_v_opt_at(2)
        # self.assert_help_arg(help['opts'][0])
        # self.assert_vk_arg(help['opts'][1])
        # self.assert_verbose_arg(help['opts'][2])
    
        # out = self.run_test_args_cmd(self.cmd__sv_opt_arg)
        out = cmd.run()
        assert self.no_args_or_opts_msg in out

    class TestSingleOptionArg(Common):
        cmdn = Common.cmdn__optional_arg

        def test_help_msg(self, cmd):
            help = cmd.get_help_msg()
            help.assert_num_args(1)
            help.assert_num_opts(3)
            help.assert_arg_at(0, cmd.single_val)

        def test_no_args(self, cmd):
            # Try with no args
            out = cmd.run()
            assert self.no_args_or_opts_msg in out

        def test_single_arg_given(self, cmd, sv):
            # Try with single arg given
            out = cmd.run(sv)
            assert cmd.single_val.to_assert_str(sv) in out
            # self.assert_arg_present(cmd.optional_arg, sv, str) in out

        def test_error_on_two_args_given(self, cmd, sv):
            # Two args should generate error
            another_sv = f"another_{sv}"
            out = cmd.gen_error(sv, another_sv)
            assert self.err_msgs.too_many_args(another_sv) in out

    class TestSingleRequiredArg(Common):
        cmdn = Common.cmdn__required_arg

        def test_help_msg(self, cmd):
            help = cmd.get_help_msg()
            help.assert_num_args(2)
            help.assert_num_opts(3)
            help.assert_arg_at(0, cmd.required_val)
            help.assert_arg_at(1, cmd.optional_val)

        def test_error_on_missing_required_arg(self, cmd):
            # No args should generate error
            out = cmd.gen_error()
            assert self.err_msgs.missing_required_arg(cmd.required_val) in out

        def test_req_arg_given(self, cmd, rv):
            # Single required arg
            out = cmd.run(rv)
            assert cmd.required_val.to_assert_str(rv) in out

        def test_req_and_optional_arg_given(self, cmd, rv):
            # Required and optional arg
            ov = "optional"
            out = cmd.run(rv, ov)
            assert cmd.required_val.to_assert_str(rv) in out
            assert cmd.optional_val.to_assert_str(ov) in out

    class TestMultiArg(Common):
        cmdn = Common.cmdn__multi_arg

        def test_help_msg(self, cmd):
            help = cmd.get_help_msg()
            help.assert_num_args(1)
            help.assert_num_opts(3)
            help.assert_arg_at(0, cmd.multi_arg)

        def test_no_args(self, cmd):
            # Try with no args
            out = cmd.run()
            assert self.no_args_or_opts_msg in out

        def test_one_arg(self, cmd, m0):
            # Try with one arg
            out = cmd.run(m0)
            assert cmd.multi_arg.to_assert_str([m0]) in out

        def test_three_args(self, cmd, m0, m1, m2):
            # Try with three args
            out = cmd.run(m0, m1, m2)
            cmd.multi_arg.to_assert_str([m0, m1, m2]) in out

        def test_delimiter_is_not_default(self, cmd, m0, m1, m2):
            # No delimiter by default, so this should all look like a single value
            out = cmd.run(f"{m0},{m1},{m2}")
            assert cmd.multi_arg.to_assert_str([f"{m0},{m1},{m2}"]) in out

    class TestDelimitedMultiArg(Common):
        cmdn = Common.cmdn__delim_multi_arg

        def test_help_msg(sel, cmd):
            help = cmd.get_help_msg()
            help.assert_num_args(1)
            help.assert_num_opts(3)
            help.assert_arg_at(0, cmd.delim_m_arg)

        def test_no_args(self, cmd):
            # Try with no args
            out = cmd.run()
            assert self.no_args_or_opts_msg in out

        def test_one_arg(self, cmd, m0):
            # Try with one arg
            out = cmd.run(m0)
            assert cmd.delim_m_arg.to_assert_str([m0]) in out

        def test_three_non_delimited_args(self, cmd, m0, m1, m2):
            # Try with three args
            out = cmd.run(m0, m1, m2)
            assert cmd.delim_m_arg.to_assert_str([m0, m1, m2]) in out

        def test_three_delimited_args(self, cmd, m0, m1, m2):
            # Use delimiter to split up values
            out = cmd.run(f"{m0},{m1},{m2}")
            assert cmd.delim_m_arg.to_assert_str([m0, m1, m2]) in out

    class TestArgsWithValueNames(Common):
        cmdn = Common.cmdn__args_with_value_names

        def test_help_msg(self, cmd):
            help = cmd.get_help_msg()
            help.assert_num_args(2)
            help.assert_num_opts(3)
            help.assert_arg_at(0, cmd.s_arg)
            help.assert_arg_at(1, cmd.m_arg)

        def test_value_names(self, cmd):
            sv ="single_val"
            m0 = "m0_val"
            m1 = "m1_val"
            out = cmd.run(sv, m0, m1)
            assert cmd
            assert cmd.s_arg.to_assert_str(sv) in out
            assert cmd.m_arg.to_assert_str([m0, m1]) in out

    class TestCombinedSingleAndMultiArgs(Common):
        cmdn = Common.cmdn__single_and_multi_arg

        def test_help_msg(self, cmd):
            help = cmd.get_help_msg()
            help.assert_num_args(2)
            help.assert_num_opts(3)
            help.assert_arg_at(0, cmd.single_val)
            help.assert_arg_at(1, cmd.multi_arg)

        def test_no_args(self, cmd):
            # Try no args
            out = cmd.run()
            assert self.no_args_or_opts_msg in out

        def test_single__arg(self, cmd, sv):
            # Try single arg only
            out = cmd.run(sv)
            assert cmd.single_val.to_assert_str(sv) in out

        def test_two_args(self, cmd, sv, m0):
            # Try two args
            out = cmd.run(sv, m0)
            assert cmd.single_val.to_assert_str(sv) in out
            assert cmd.multi_arg.to_assert_str([m0]) in out

        def test_three_args(self, cmd, sv, m0, m1):
            # Try three args
            out = cmd.run(sv, m0, m1)
            assert cmd.single_val.to_assert_str(sv) in out
            assert cmd.multi_arg.to_assert_str([m0, m1]) in out
