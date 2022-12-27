import pytest
from .shared import CLICommon, CmdOpt

class Common(CLICommon):
    # Hard-coded here to ensure it matches the TOML
    cmdn__sv_opt_opt = "single_value_optional_opt"
    cmdn__sv_req_opt = "single_value_required_opt"
    cmdn__multi_opt = "multi_opts"
    cmdn__flag_opts = "flag_opts"
    cmdn__opt_vns = "opts_with_value_names"
    cmdn__opt_aliases = "opts_with_aliases"
    cmdn__hidden_opt = "hidden_opt"

    # cmds = {
    #     cmdn__sv_opt_opt: CLICommon.test_args_sub_cmd(
    #         cmdn__sv_opt_opt,
    #         help="Command taking optional, single option",
    #         opts=[
    #             CmdOpt(
    #                 name="implicit_single_val",
    #                 help='Implicit non-required single value',
    #                 takes_value=True,
    #                 required=False,
    #             ),
    #             CmdOpt(
    #                 name="explicit_single_val",
    #                 help='Explicit non-required single value',
    #                 takes_value=True,
    #                 required=False,
    #             ),
    #         ]
    #     ),
    #     cmdn__sv_req_opt: CLICommon.test_args_sub_cmd(
    #         cmdn__sv_req_opt,
    #         help="Command with single-value optional and required options",
    #         opts=[
    #             CmdOpt(
    #                 name="non_req_val",
    #                 help="Non-required single value",
    #                 takes_value=True,
    #             ),
    #             CmdOpt(
    #                 name="req_val",
    #                 help="Required single value",
    #                 takes_value=True,
    #                 required=True,
    #             ),
    #         ]
    #     ),
    #     cmdn__multi_opt: CLICommon.test_args_sub_cmd(
    #         cmdn__multi_opt,
    #         help="Command with multi-value optional and required options",
    #         opts=[
    #             CmdOpt(
    #                 name="m_opt",
    #                 help="Opt with multiple values",
    #                 multi=True,
    #             ),
    #             CmdOpt(
    #                 name="im_m_opt",
    #                 help="Opt accepting multiple values were 'takes value' is implied",
    #                 multi=True,
    #             ),
    #             CmdOpt(
    #                 name="req_m_opt",
    #                 help="Required opt accepting multiple values",
    #                 multi=True,
    #                 required=True,
    #             ),
    #             CmdOpt(
    #                 name="d_m_opt",
    #                 help="Delimited multi opt",
    #                 multi=True,
    #             ),
    #             CmdOpt(
    #                 name="d_im_m_opt",
    #                 help="Delimited opt where 'multi' and 'takes value' is implied",
    #                 multi=True,
    #             ),
    #         ]
    #     ),
    #     cmdn__flag_opts: CLICommon.test_args_sub_cmd(
    #         cmdn__flag_opts,
    #         help="Command with flag-style options only",
    #         opts=[
    #             CmdOpt(
    #                 name="im_f_opt",
    #                 help="Stackable flag opt with 'takes value=false' implied",
    #             ),
    #             CmdOpt(
    #                 name="ex_f_opt",
    #                 help="Stackable flag opt with 'takes value=false' set",
    #             ),
    #         ]
    #     ),
    #     cmdn__opt_vns: CLICommon.test_args_sub_cmd(
    #         cmdn__opt_vns,
    #         help="Command with single/multi-opts with custom value names",
    #         opts=[
    #             CmdOpt(
    #                 name="s_opt_nv_im_tv",
    #                 help="Single opt with value name, implying 'takes_value'=true",
    #                 value_name="s_val_impl",
    #             ),
    #             CmdOpt(
    #                 name="s_opt_nv_ex_tv",
    #                 help="Single opt with value name and explicit 'takes_value'=true",
    #                 value_name="s_val_expl",
    #                 takes_value=True,
    #             ),
    #             CmdOpt(
    #                 name="m_opt_named_val",
    #                 help="Multi-opt with value name",
    #                 value_name="m_val",
    #                 multi=True,
    #             ),
    #             CmdOpt(
    #                 name="s_opt_ln_nv",
    #                 help="Single opt with long name and value name",
    #                 value_name="ln_nv",
    #             ),
    #         ]
    #     ),
    #     cmdn__opt_aliases: CLICommon.test_args_sub_cmd(
    #         cmdn__opt_aliases,
    #         help="Command with option aliasing, custom long, and short names",
    #         opts=[
    #             CmdOpt(
    #                 name="single_opt",
    #                 help="Single opt with long/short name",
    #                 takes_value=True,
    #                 ln="s_opt",
    #                 sn="s"
    #             ),
    #             CmdOpt(
    #                 name="multi_opt",
    #                 help="Multi-opt with long/short name",
    #                 takes_value=True,
    #                 multi=True,
    #                 ln="m_opt",
    #                 sn="m"
    #             ),
    #             CmdOpt(
    #                 name="occurrence_counter",
    #                 help="Flag opt with long/short name",
    #                 ln="cnt",
    #                 sn="o",
    #             ),
    #             CmdOpt(
    #                 name="flag_opt_short_name",
    #                 help="Flag opt with short name only",
    #                 sn="f"
    #             ),
    #             CmdOpt(
    #                 name="flag_opt_long_name",
    #                 help="Flag opt with long name only",
    #                 ln="ln_f_opt"
    #             ),
    #             CmdOpt(
    #                 name="flag_opt_dupl_ln_sn",
    #                 help="Flag opt with ln matching another's sn",
    #                 ln="f"
    #             ),
    #             CmdOpt(
    #                 name="fo_sn_aliases",
    #                 help="Flag opt with short aliases",
    #                 sn_aliases=['a', 'b']
    #             ),
    #             CmdOpt(
    #                 name="fo_sn_and_aliases",
    #                 help="Flag opt with short name and short aliases",
    #                 sn="c",
    #                 sn_aliases=['d', 'e']
    #             ),
    #             CmdOpt(
    #                 name="fo_ln_aliases",
    #                 help="Flag opt with long aliases",
    #                 ln_aliases=['fa', 'fb']
    #             ),
    #             CmdOpt(
    #                 name="fo_ln_and_aliases",
    #                 help="Flag opt with long name and long aliases",
    #                 ln="fc",
    #                 ln_aliases=['fd', 'fe']
    #             ),
    #             CmdOpt(
    #                 name="fo_sn_ln_aliases",
    #                 help="Flag opt with long and short aliases",
    #                 ln_aliases=['sn_ln_1', 'sn_ln_2'],
    #                 sn_aliases=['z'],
    #             ),
    #         ]
    #     ),
    #     cmdn__hidden_opt: CLICommon.test_args_sub_cmd(
    #         cmdn__hidden_opt,
    #         help="Command with a hidden opt",
    #         opts=[
    #             CmdOpt(
    #                 name="hidden_opt",
    #                 help="Hidden opt",
    #                 hidden=True,
    #             ),
    #             CmdOpt(
    #                 # name="non_hidden_opt",
    #                 name="visible_opt",
    #                 help="Visible, non-hidden, opt",
    #             ),
    #         ]
    #     ),
    # }

    # TODO consolidate this with arg building. or remove
    @pytest.fixture
    def cmd(self):
        return getattr(self.cmd_testers.test_args, self.cmdn)

class T_OptBuilding(Common):

    class TestOptionSingleValueOpts(Common):
        cmdn = Common.cmdn__sv_opt_opt

        @pytest.fixture
        def e_opt(self, cmd):
            return cmd.explicit_single_val

        @pytest.fixture
        def i_opt(self, cmd):
            return cmd.implicit_single_val

        @pytest.fixture
        def ev(self):
            return "expl_value"

        @pytest.fixture
        def iv(self):
            return "impl_value"

        def test_help_msg(self, cmd, e_opt, i_opt):
            help = cmd.get_help_msg()
            help.assert_opt_at(0, e_opt)
            help.assert_opt_at(2, i_opt)
        
        def test_expl_sv_opt(self, cmd, e_opt, ev):
            out = cmd.run(e_opt.ln_to_cli(), ev)
            assert e_opt.to_assert_str(ev) in out
            assert cmd.parse_arg_keys(out) == [e_opt.name]
        
        def test_impl_sv_opt(self, cmd, i_opt, iv):
            out = cmd.run(i_opt.ln_to_cli(), iv)
            assert i_opt.to_assert_str(iv) in out
            assert cmd.parse_arg_keys(out) == [i_opt.name]
        
        def test_both_opts(self, cmd, e_opt, i_opt, ev, iv):
            out = cmd.run(i_opt.ln_to_cli(), iv, e_opt.ln_to_cli(), ev)
            assert e_opt.to_assert_str(ev) in out
            assert i_opt.to_assert_str(iv) in out
            assert cmd.parse_arg_keys(out) == [i_opt.name, e_opt.name]
        
        def test_both_opts_in_reverse_order(self, cmd, e_opt, i_opt, ev, iv):
            out = cmd.run(e_opt.ln_to_cli(), ev, i_opt.ln_to_cli(), iv)
            assert e_opt.to_assert_str(ev) in out
            assert i_opt.to_assert_str(iv) in out
            assert cmd.parse_arg_keys(out) == [i_opt.name, e_opt.name]

        def test_error_on_multi_opt(self, cmd, e_opt, ev):
            another_ev = f"another_{ev}"
            out = cmd.gen_error(e_opt.ln_to_cli(), ev, another_ev)
            assert self.err_msgs.too_many_args(another_ev) in out
    
    class TestRequiredOpt(Common):
        cmdn = Common.cmdn__sv_req_opt

        @pytest.fixture
        def o_opt(self, cmd):
            return cmd.non_req_val

        @pytest.fixture
        def r_opt(self, cmd):
            return cmd.req_val

        @pytest.fixture
        def ov(self):
            return "opt_val"

        @pytest.fixture
        def rv(self):
            return "req_val"

        def test_help_msg(self, cmd, o_opt, r_opt):
            help = cmd.get_help_msg()
            help.assert_opt_at(2, o_opt)
            help.assert_opt_at(3, r_opt)

        def test_req_opt_given(self, cmd, r_opt, rv):
            out = cmd.run(r_opt.ln_to_cli(), rv)
            assert r_opt.to_assert_str(rv) in out
            assert cmd.parse_arg_keys(out) == [r_opt.name]

        def test_req_and_optional_opt_given(self, cmd, o_opt, r_opt, ov, rv):
            out = cmd.run(r_opt.ln_to_cli(), rv, o_opt.ln_to_cli(), ov)
            assert r_opt.to_assert_str(rv) in out
            assert o_opt.to_assert_str(ov) in out
            assert cmd.parse_arg_keys(out) == [o_opt.name, r_opt.name]

        def test_error_on_no_opts_given(self, cmd, r_opt):
            out = cmd.gen_error()
            assert self.err_msgs.missing_required_arg(r_opt) in out

        def test_error_on_optional_opt_only(self, cmd, r_opt, o_opt, ov):
            out = cmd.gen_error(o_opt.ln_to_cli(), ov)
            assert self.err_msgs.missing_required_arg(r_opt) in out

    class TestMultiValueOpts(Common):
        cmdn = Common.cmdn__multi_opt

        @pytest.fixture
        def m_opt(self, cmd):
            return cmd.m_opt

        @pytest.fixture
        def im_m_opt(self, cmd):
            return cmd.im_m_opt

        @pytest.fixture
        def req_m_opt(self, cmd):
            return cmd.req_m_opt

        @pytest.fixture
        def d_m_opt(self, cmd):
            return cmd.d_m_opt

        @pytest.fixture
        def d_im_m_opt(self, cmd):
            return cmd.d_im_m_opt

        @pytest.fixture
        def m_opt_v(self):
            return ["m0"]

        @pytest.fixture
        def im_m_opt_v(self):
            return ["mA", "mB", "mC"]

        @pytest.fixture
        def req_m_opt_v(self):
            return ["r0", "r1"]

        @pytest.fixture
        def d_m_opt_v(self):
            return ["d0", "d1"]

        @pytest.fixture
        def d_im_m_opt_v(self):
            return ["i0", "i1", "i2"]

        @pytest.fixture
        def m_opt_v_dlim(self):
            return "m0"

        @pytest.fixture
        def im_m_opt_v_dlim(self):
            return "mA,mB,mC"

        @pytest.fixture
        def req_m_opt_v_dlim(self):
            return "r0,r1"

        @pytest.fixture
        def d_m_opt_v_dlim(self):
            return "d0,d1"

        @pytest.fixture
        def d_im_m_opt_v_dlim(self):
            return "i0,i1,i2"

        def test_help_msg(self, cmd, m_opt, im_m_opt, req_m_opt, d_m_opt, d_im_m_opt):
            help = cmd.get_help_msg()
            help.assert_opt_at(5, m_opt)
            help.assert_opt_at(3, im_m_opt)
            help.assert_opt_at(6, req_m_opt)
            help.assert_opt_at(1, d_m_opt)
            help.assert_opt_at(0, d_im_m_opt)

        def test_all_multi_val_opts_given(self, cmd,
            m_opt, im_m_opt, req_m_opt, d_m_opt, d_im_m_opt,
            m_opt_v, im_m_opt_v, req_m_opt_v, d_m_opt_v, d_im_m_opt_v
        ):
            out = cmd.run(
                m_opt.ln_to_cli(), *m_opt_v,
                im_m_opt.ln_to_cli(), *im_m_opt_v,
                req_m_opt.ln_to_cli(), *req_m_opt_v,
                d_m_opt.ln_to_cli(), *d_m_opt_v,
                d_im_m_opt.ln_to_cli(), *d_im_m_opt_v
            )
            assert m_opt.to_assert_str(m_opt_v) in out
            assert im_m_opt.to_assert_str(im_m_opt_v) in out
            assert req_m_opt.to_assert_str(req_m_opt_v) in out
            assert d_m_opt.to_assert_str(d_m_opt_v) in out
            assert d_im_m_opt.to_assert_str(d_im_m_opt_v) in out
            assert cmd.parse_arg_keys(out) == [
                m_opt.name,
                im_m_opt.name,
                req_m_opt.name,
                d_m_opt.name,
                d_im_m_opt.name
            ]

        def test_delimited_is_not_the_default(self, cmd,
            m_opt, im_m_opt, req_m_opt, d_m_opt, d_im_m_opt,
            d_m_opt_v, d_im_m_opt_v,
            m_opt_v_dlim, im_m_opt_v_dlim, req_m_opt_v_dlim, d_m_opt_v_dlim, d_im_m_opt_v_dlim
        ):
            out = cmd.run(
                m_opt.ln_to_cli(), m_opt_v_dlim,
                im_m_opt.ln_to_cli(), im_m_opt_v_dlim,
                req_m_opt.ln_to_cli(), req_m_opt_v_dlim,
                d_m_opt.ln_to_cli(), d_m_opt_v_dlim,
                d_im_m_opt.ln_to_cli(), d_im_m_opt_v_dlim
            )
            assert m_opt.to_assert_str([m_opt_v_dlim]) in out
            assert im_m_opt.to_assert_str([im_m_opt_v_dlim]) in out
            assert req_m_opt.to_assert_str([req_m_opt_v_dlim]) in out
            assert d_m_opt.to_assert_str(d_m_opt_v) in out
            assert d_im_m_opt.to_assert_str(d_im_m_opt_v) in out
            assert cmd.parse_arg_keys(out) == [
                m_opt.name,
                im_m_opt.name,
                req_m_opt.name,
                d_m_opt.name,
                d_im_m_opt.name
            ]

        def test_multiple_occurrences_stack(self, cmd, m_opt, req_m_opt, m_opt_v, im_m_opt_v, req_m_opt_v):
            out = cmd.run(
                m_opt.ln_to_cli(), *m_opt_v,
                m_opt.ln_to_cli(), *im_m_opt_v,
                req_m_opt.ln_to_cli(), *req_m_opt_v,
                req_m_opt.ln_to_cli(), *req_m_opt_v,
            )
            assert m_opt.to_assert_str([*m_opt_v, *im_m_opt_v])
            assert req_m_opt.to_assert_str([*req_m_opt_v, *req_m_opt_v])
            assert cmd.parse_arg_keys(out) == [m_opt.name, req_m_opt.name]

        def test_error_on_missing_required_opt(self, cmd, req_m_opt):
            out = cmd.gen_error()
            assert self.err_msgs.missing_required_arg(req_m_opt) in out

    class TestFlagOpts(Common):
        cmdn = Common.cmdn__flag_opts

        @pytest.fixture
        def e_opt(self, cmd):
            return cmd.ex_f_opt

        @pytest.fixture
        def i_opt(self, cmd):
            return cmd.im_f_opt

        def test_help_msg(self, cmd, e_opt, i_opt):
            help = cmd.get_help_msg()
            help.assert_opt_at(0, e_opt)
            help.assert_opt_at(2, i_opt)

        def test_no_flag_opts_given(self, cmd):
            out = cmd.run()
            assert self.no_args_or_opts_msg in out

        def test_one_flag_opt_given(self, cmd, e_opt):
            out = cmd.run(e_opt.ln_to_cli())
            assert e_opt.to_assert_str(1) in out

        def test_two_flag_opts_given(self, cmd, e_opt, i_opt):
            out = cmd.run(e_opt.ln_to_cli(), i_opt.ln_to_cli())
            assert e_opt.to_assert_str(1) in out
            assert i_opt.to_assert_str(1) in out

        def test_multiple_occurrences_stack(self, cmd, e_opt, i_opt):
            out = cmd.run(
                e_opt.ln_to_cli(),
                e_opt.ln_to_cli(),
                i_opt.ln_to_cli(),
                e_opt.ln_to_cli(),
                i_opt.ln_to_cli()
            )
            assert e_opt.to_assert_str(3) in out
            assert i_opt.to_assert_str(2) in out

        def test_error_on_flag_opt_with_value(self, cmd, i_opt):
            sv = "single_val"
            out = cmd.gen_error(i_opt.ln_to_cli(), sv)
            assert self.err_msgs.too_many_args(sv) in out

    class TestOptsWithValueNames(Common):
        cmdn = Common.cmdn__opt_vns

        @pytest.fixture
        def e_s_opt(self, cmd):
            return cmd.s_opt_nv_ex_tv

        @pytest.fixture
        def i_s_opt(self, cmd):
            return cmd.s_opt_nv_im_tv

        @pytest.fixture
        def m_opt(self, cmd):
            return cmd.m_opt_named_val

        @pytest.fixture
        def ln_s_opt(self, cmd):
            return cmd.s_opt_ln_nv

        @pytest.fixture
        def sv_e(self):
            return "ex_single_val"

        @pytest.fixture
        def sv_i(self):
            return "im_single_val"

        @pytest.fixture
        def mv(self):
            return ["mv0", "mv1"]

        def test_help_msg(self, cmd, e_s_opt, i_s_opt, m_opt, ln_s_opt):
            help = cmd.get_help_msg()
            help.assert_opt_at(5, i_s_opt)
            help.assert_opt_at(4, e_s_opt)
            help.assert_opt_at(2, m_opt)
            help.assert_opt_at(3, ln_s_opt)
        
        def test_single_value_opt_with_value_name(self, cmd, e_s_opt, i_s_opt, sv_e, sv_i):
            out = cmd.run(i_s_opt.ln_to_cli(), sv_i, e_s_opt.ln_to_cli(), sv_e)
            assert i_s_opt.to_assert_str(sv_i) in out
            assert e_s_opt.to_assert_str(sv_e) in out
            assert cmd.parse_arg_keys(out) == [i_s_opt.name, e_s_opt.name]

        def test_multi_value_opt_with_value_name(self, cmd, m_opt, mv):
            out = cmd.run(m_opt.ln_to_cli(), *mv)
            assert m_opt.to_assert_str(mv) in out
            assert cmd.parse_arg_keys(out) == [m_opt.name]

        def test_single_value_opt_with_value_and_long_names(self, cmd, ln_s_opt, sv_i):
            out = cmd.run(ln_s_opt.ln_to_cli(), sv_i)
            assert ln_s_opt.to_assert_str(sv_i) in out
            assert cmd.parse_arg_keys(out) == [ln_s_opt.name]

    class TestOptsWithAliases(Common):
        cmdn = Common.cmdn__opt_aliases

        @pytest.fixture
        def s_opt(self, cmd):
            return cmd.single_opt

        @pytest.fixture
        def m_opt(self, cmd):
            return cmd.multi_opt

        @pytest.fixture
        def oc_opt(self, cmd):
            return cmd.occurrence_counter

        @pytest.fixture
        def f_sn_opt(self, cmd):
            return cmd.flag_opt_short_name

        @pytest.fixture
        def f_ln_opt(self, cmd):
            return cmd.flag_opt_long_name

        @pytest.fixture
        def f_dupl_ln_sn_opt(self, cmd):
            return cmd.flag_opt_dupl_ln_sn

        @pytest.fixture
        def f_sn_al_opt(self, cmd):
            return cmd.fo_sn_aliases

        @pytest.fixture
        def f_sn_and_al_opt(self, cmd):
            return cmd.fo_sn_and_aliases

        @pytest.fixture
        def f_ln_al_opt(self, cmd):
            return cmd.fo_ln_aliases

        @pytest.fixture
        def f_ln_and_al_opt(self, cmd):
            return cmd.fo_ln_and_aliases

        @pytest.fixture
        def f_ln_sn_al_opt(self, cmd):
            return cmd.fo_sn_ln_aliases

        @pytest.fixture
        def tv(self):
            return "single_test_val"

        def test_help_msg(self, cmd,
            s_opt, m_opt, oc_opt, f_sn_opt, f_ln_opt,
            f_dupl_ln_sn_opt, f_ln_al_opt, f_sn_al_opt,
             f_sn_and_al_opt, f_ln_and_al_opt, f_ln_sn_al_opt
        ):
            help = cmd.get_help_msg()
            help.assert_num_args(0)
            help.assert_num_opts(14)
            help.assert_opt_at(12, s_opt)
            help.assert_opt_at(10, m_opt)
            help.assert_opt_at(11, oc_opt)
            help.assert_opt_at(2, f_sn_opt)
            help.assert_opt_at(9, f_ln_opt)
            help.assert_opt_at(1, f_dupl_ln_sn_opt)
            help.assert_opt_at(4, f_ln_al_opt)
            help.assert_opt_at(0, f_sn_and_al_opt)
            help.assert_opt_at(3, f_ln_and_al_opt)
            help.assert_opt_at(5, f_sn_al_opt)
            help.assert_opt_at(6, f_ln_sn_al_opt)

        def test_single_val_opt_as_long_name(self, cmd, s_opt, tv):
            # Try single opt long name
            out = cmd.run(s_opt.ln_to_cli(), tv)
            assert s_opt.to_assert_str(tv) in out
            assert cmd.parse_arg_keys(out) == [s_opt.name]

        def test_single_val_opt_as_short_name(self, cmd, s_opt, tv):
            # Try single opt short name
            out = cmd.run(s_opt.sn_to_cli(), tv)
            assert s_opt.to_assert_str(tv) in out
            assert cmd.parse_arg_keys(out) == [s_opt.name]

        def test_error_on_single_val_opt_name(self, cmd, s_opt, tv):
            # Try single opt opt name
            # Should fail as ln and sn are used on the CLI
            out = cmd.gen_error(f"--{s_opt.name}", tv)
            assert self.err_msgs.unknown_opt_msg(s_opt) in out

        def test_multi_val_opt_as_long_name(self, cmd, m_opt, tv):
            # Try multi opt long name
            out = cmd.run(m_opt.ln_to_cli(), tv, tv)
            assert m_opt.to_assert_str([tv, tv]) in out
            assert cmd.parse_arg_keys(out) == [m_opt.name]

        def test_multi_val_opt_as_short_name(self, cmd, m_opt, tv):
            # Try multi opt short name
            out = cmd.run(m_opt.sn_to_cli(), tv, tv, tv)
            assert m_opt.to_assert_str([tv, tv, tv]) in out
            assert cmd.parse_arg_keys(out) == [m_opt.name]

        def test_error_on_multi_val_opt_name(self, cmd, m_opt, tv):
            # Try multi opt opt name
            # Should fail as ln and sn are used on the CLI
            out = cmd.gen_error(f"--{m_opt.name}", tv)
            assert self.err_msgs.unknown_opt_msg(m_opt) in out

        def test_multi_val_opt_ln_sn_stacking(self, cmd, m_opt, tv):
            # Try multi opt stacking
            out = cmd.run(
                m_opt.sn_to_cli(), tv, tv,
                m_opt.ln_to_cli(), tv, tv,
                m_opt.sn_to_cli(), tv
            )
            assert m_opt.to_assert_str([tv, tv, tv, tv, tv]) in out
            assert cmd.parse_arg_keys(out) == [m_opt.name]

        def test_flag_opt_long_name(self, cmd, oc_opt):
            # Try occurrence counter long name
            out = cmd.run(oc_opt.ln_to_cli())
            assert oc_opt.to_assert_str(1) in out
            assert cmd.parse_arg_keys(out) == [oc_opt.name]

        def test_flag_opt_short_name(self, cmd, oc_opt):
            # Try occurrence counter short name
            out = cmd.run(oc_opt.sn_to_cli())
            assert oc_opt.to_assert_str(1) in out

        def test_flag_opt_ln_sn_stacking(self, cmd, oc_opt):
            # Try occurrence counter with both short and long name
            out = cmd.run(oc_opt.sn_to_cli(), oc_opt.ln_to_cli(), oc_opt.sn_to_cli(), oc_opt.ln_to_cli())
            assert oc_opt.to_assert_str(4) in out

        def test_error_on_flag_opt_name(self, cmd, oc_opt):
            # Try occurrence counter opt name
            out = cmd.gen_error(f"--{oc_opt.name}")
            assert self.err_msgs.unknown_opt_msg(oc_opt) in out

        def test_flag_opt_with_short_name_only(self, cmd, f_sn_opt):
            # Try flag opt short only
            out = cmd.run(f_sn_opt.sn_to_cli())
            assert f_sn_opt.to_assert_str(1) in out

        def test_error_on_short_name_only_opt_name(self, cmd, f_sn_opt):
            # Try flag opt short only opt name
            # Should generate an error
            out = cmd.gen_error(f"--{f_sn_opt.name}")
            assert self.err_msgs.unknown_opt_msg(f_sn_opt) in out

        def test_flag_opt_with_long_name_only(self, cmd, f_ln_opt):
            # Try flag opt long only
            out = cmd.run(f_ln_opt.ln_to_cli())
            assert f_ln_opt.to_assert_str(1) in out

        def test_error_on_long_name_only_opt_name(self, cmd, f_ln_opt):
            # Try flag opt long only opt name
            # Should generate an error
            out = cmd.gen_error(f"--{f_ln_opt.name}")
            assert self.err_msgs.unknown_opt_msg(f_ln_opt) in out

        def test_no_conflict_between_ln_and_sn(self, cmd, f_sn_opt, f_dupl_ln_sn_opt):
            # Try flag opt long name same as another's short name
            out = cmd.run("--f")
            assert f_dupl_ln_sn_opt.to_assert_str(1) in out

            out = cmd.run("--f", "-f")
            assert f_dupl_ln_sn_opt.to_assert_str(1) in out
            assert f_sn_opt.to_assert_str(1) in out

        def test_short_name_aliasing(self, cmd, f_sn_al_opt):
            # Try short name aliases
            out = cmd.run(f'--{f_sn_al_opt.name}', '-a', '-b')
            assert f_sn_al_opt.to_assert_str(3) in out

        def test_sn_with_sn_aliases(self, cmd, f_sn_and_al_opt):
            # Try short name with aliases
            out = cmd.run('-c', '-d', '-e')
            assert f_sn_and_al_opt.to_assert_str(3) in out

        def test_error_on_opt_name_with_sn_and_sn_aliases(self, cmd, f_sn_and_al_opt):
            out = cmd.gen_error(f"--{f_sn_and_al_opt.name}", '-d', '-e')
            assert self.err_msgs.unknown_opt_msg(f_sn_and_al_opt) in out

        def test_long_name_aliasing(self, cmd, f_ln_al_opt):
            # Try long name aliases
            out = cmd.run(f"--{f_ln_al_opt.name}", '--fa', '--fb')
            assert f_ln_al_opt.to_assert_str(3) in out

        def test_ln_with_ln_aliases(self, cmd, f_ln_and_al_opt):
            # Try long name with aliases
            out = cmd.run('--fc', '--fd', '--fe')
            assert f_ln_and_al_opt.to_assert_str(3) in out

        def test_error_on_opt_name_with_ln_and_ln_aliases(self, cmd, f_ln_and_al_opt):
            out = cmd.gen_error(f"--{f_ln_and_al_opt.name}", '--fd', '--fe')
            assert self.err_msgs.unknown_opt_msg(f_ln_and_al_opt) in out

        def test_sn_and_ln_aliases_only(self, cmd, f_ln_sn_al_opt):
            # Try long/short name aliases
            out = cmd.run(f'--{f_ln_sn_al_opt.name}', '-z', '--sn_ln_1', '--sn_ln_2')
            assert f_ln_sn_al_opt.to_assert_str(4) in out

    class TestHiddenOpts(Common):
        cmdn = Common.cmdn__hidden_opt

        @pytest.fixture
        def h_opt(self, cmd):
            return cmd.hidden_opt

        @pytest.fixture
        def v_opt(self, cmd):
            return cmd.visible_opt

        def test_help_msg(self, cmd, v_opt):
            help = cmd.get_help_msg()
            help.assert_num_args(0)
            help.assert_num_opts(4)
            help.assert_help_opt_at(0)
            help.assert_vk_opt_at(1)
            help.assert_opt_at(3, v_opt)
            help.assert_v_opt_at(2)

        def test_hidden_opt_is_available(self, cmd, h_opt):
            out = cmd.run(h_opt.ln_to_cli())
            assert h_opt.to_assert_str(1) in out

        def test_visible_opt_only(self, cmd, v_opt):
            out = cmd.run(v_opt.ln_to_cli())
            assert v_opt.to_assert_str(1) in out

        def test_hidden_and_visible_opt_only(self, cmd, h_opt, v_opt):
            out = cmd.run(h_opt.ln_to_cli(), h_opt.ln_to_cli(), v_opt.ln_to_cli())
            assert h_opt.to_assert_str(2) in out
            assert v_opt.to_assert_str(1) in out

        def test_error_on_random_opt(self, cmd):
            r_opt = "random"
            out = cmd.gen_error(f"--{r_opt}")
            assert self.err_msgs.unknown_opt_msg(r_opt) in out
