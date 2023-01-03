import pytest
from .shared import CLICommon

class Common(CLICommon):
    pass

class T_OptBuilding(Common):

    class TestOptionSingleValueOpts(Common):
        _cmd = Common.cmd_testers.test_args.single_value_optional_opt

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
            help.assert_opts(e_opt, "help", i_opt, "vk", "v")
        
        def test_expl_sv_opt(self, cmd, e_opt, ev):
            out = cmd.run(e_opt.ln_to_cli(), ev)
            cmd.assert_args(out, (e_opt, ev))
        
        def test_impl_sv_opt(self, cmd, i_opt, iv):
            out = cmd.run(i_opt.ln_to_cli(), iv)
            cmd.assert_args(out, (i_opt, iv))
        
        def test_both_opts(self, cmd, e_opt, i_opt, ev, iv):
            out = cmd.run(i_opt.ln_to_cli(), iv, e_opt.ln_to_cli(), ev)
            cmd.assert_args(out, (i_opt, iv), (e_opt, ev))
        
        def test_both_opts_in_reverse_order(self, cmd, e_opt, i_opt, ev, iv):
            out = cmd.run(e_opt.ln_to_cli(), ev, i_opt.ln_to_cli(), iv)
            cmd.assert_args(out, (i_opt, iv), (e_opt, ev))

        def test_error_on_multi_opt(self, cmd, e_opt, ev):
            another_ev = f"another_{ev}"
            out = cmd.gen_error(e_opt.ln_to_cli(), ev, another_ev)
            assert self.err_msgs.too_many_args(another_ev) in out
    
    class TestRequiredOpt(Common):
        _cmd = Common.cmd_testers.test_args.single_value_required_opt

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
            help.assert_opts("help", "vk", o_opt, r_opt, "v")

        def test_req_opt_given(self, cmd, r_opt, rv):
            out = cmd.run(r_opt.ln_to_cli(), rv)
            cmd.assert_args(out, (r_opt, rv))

        def test_req_and_optional_opt_given(self, cmd, o_opt, r_opt, ov, rv):
            out = cmd.run(r_opt.ln_to_cli(), rv, o_opt.ln_to_cli(), ov)
            cmd.assert_args(out, (r_opt, rv), (o_opt, ov))

        def test_error_on_no_opts_given(self, cmd, r_opt):
            out = cmd.gen_error()
            assert self.err_msgs.missing_required_arg(r_opt) in out

        def test_error_on_optional_opt_only(self, cmd, r_opt, o_opt, ov):
            out = cmd.gen_error(o_opt.ln_to_cli(), ov)
            assert self.err_msgs.missing_required_arg(r_opt) in out

    class TestMultiValueOpts(Common):
        _cmd = Common.cmd_testers.test_args.multi_opts

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
            help.assert_opts(d_im_m_opt, d_m_opt, "help", im_m_opt, "vk", m_opt, req_m_opt, "v")

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
            cmd.assert_args(
                out,
                (m_opt, m_opt_v),
                (im_m_opt, im_m_opt_v),
                (req_m_opt, req_m_opt_v),
                (d_m_opt, d_m_opt_v),
                (d_im_m_opt, d_im_m_opt_v),
            )

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
            cmd.assert_args(
                out,
                (m_opt, [m_opt_v_dlim]),
                (im_m_opt, [im_m_opt_v_dlim]),
                (req_m_opt, [req_m_opt_v_dlim]),
                (d_m_opt, d_m_opt_v),
                (d_im_m_opt, d_im_m_opt_v),
            )

        def test_multiple_occurrences_stack(self, cmd, m_opt, req_m_opt, m_opt_v, im_m_opt_v, req_m_opt_v):
            out = cmd.run(
                m_opt.ln_to_cli(), *m_opt_v,
                m_opt.ln_to_cli(), *im_m_opt_v,
                req_m_opt.ln_to_cli(), *req_m_opt_v,
                req_m_opt.ln_to_cli(), *req_m_opt_v,
            )
            cmd.assert_args(
                out,
                (m_opt, [*m_opt_v, *im_m_opt_v]),
                (req_m_opt, [*req_m_opt_v, *req_m_opt_v]),
            )

        def test_error_on_missing_required_opt(self, cmd, req_m_opt):
            out = cmd.gen_error()
            assert self.err_msgs.missing_required_arg(req_m_opt) in out

    class TestFlagOpts(Common):
        _cmd = Common.cmd_testers.test_args.flag_opts

        @pytest.fixture
        def e_opt(self, cmd):
            return cmd.ex_f_opt

        @pytest.fixture
        def i_opt(self, cmd):
            return cmd.im_f_opt

        def test_help_msg(self, cmd, e_opt, i_opt):
            help = cmd.get_help_msg()
            help.assert_opts(e_opt, "help", i_opt, "vk", "v")

        def test_no_flag_opts_given(self, cmd):
            out = cmd.run()
            cmd.assert_args(out, None)

        def test_one_flag_opt_given(self, cmd, e_opt):
            out = cmd.run(e_opt.ln_to_cli())
            cmd.assert_args(out, (e_opt, 1))

        def test_two_flag_opts_given(self, cmd, e_opt, i_opt):
            out = cmd.run(e_opt.ln_to_cli(), i_opt.ln_to_cli())
            cmd.assert_args(out, (e_opt, 1), (i_opt, 1))

        def test_multiple_occurrences_stack(self, cmd, e_opt, i_opt):
            out = cmd.run(
                e_opt.ln_to_cli(),
                e_opt.ln_to_cli(),
                i_opt.ln_to_cli(),
                e_opt.ln_to_cli(),
                i_opt.ln_to_cli()
            )
            cmd.assert_args(out, (e_opt, 3), (i_opt, 2))

        def test_error_on_flag_opt_with_value(self, cmd, i_opt):
            sv = "single_val"
            out = cmd.gen_error(i_opt.ln_to_cli(), sv)
            assert self.err_msgs.too_many_args(sv) in out

    class TestOptsWithValueNames(Common):
        _cmd = Common.cmd_testers.test_args.opts_with_value_names

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
            help.assert_opts("help", "vk", m_opt, ln_s_opt, e_s_opt, i_s_opt, "v")
        
        def test_single_value_opt_with_value_name(self, cmd, e_s_opt, i_s_opt, sv_e, sv_i):
            out = cmd.run(i_s_opt.ln_to_cli(), sv_i, e_s_opt.ln_to_cli(), sv_e)
            cmd.assert_args(out, (i_s_opt, sv_i), (e_s_opt, sv_e))

        def test_multi_value_opt_with_value_name(self, cmd, m_opt, mv):
            out = cmd.run(m_opt.ln_to_cli(), *mv)
            cmd.assert_args(out, (m_opt, mv))

        def test_single_value_opt_with_value_and_long_names(self, cmd, ln_s_opt, sv_i):
            out = cmd.run(ln_s_opt.ln_to_cli(), sv_i)
            cmd.assert_args(out, (ln_s_opt, sv_i))

    class TestOptsWithAliases(Common):
        _cmd = Common.cmd_testers.test_args.opts_with_aliases

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
            help.assert_args(None)
            help.assert_opts(
                f_sn_and_al_opt,
                f_dupl_ln_sn_opt,
                f_sn_opt,
                f_ln_and_al_opt,
                f_ln_al_opt,
                f_sn_al_opt,
                f_ln_sn_al_opt,
                "help",
                "vk",
                f_ln_opt,
                m_opt,
                oc_opt,
                s_opt,
                "v"
            )

        def test_single_val_opt_as_long_name(self, cmd, s_opt, tv):
            # Try single opt long name
            out = cmd.run(s_opt.ln_to_cli(), tv)
            cmd.assert_args(out, (s_opt, tv))

        def test_single_val_opt_as_short_name(self, cmd, s_opt, tv):
            # Try single opt short name
            out = cmd.run(s_opt.sn_to_cli(), tv)
            cmd.assert_args(out, (s_opt, tv))

        def test_error_on_single_val_opt_name(self, cmd, s_opt, tv):
            # Try single opt opt name
            # Should fail as ln and sn are used on the CLI
            out = cmd.gen_error(f"--{s_opt.name}", tv)
            assert self.err_msgs.unknown_opt_msg(s_opt) in out

        def test_multi_val_opt_as_long_name(self, cmd, m_opt, tv):
            # Try multi opt long name
            out = cmd.run(m_opt.ln_to_cli(), tv, tv)
            cmd.assert_args(out, (m_opt, [tv, tv]))

        def test_multi_val_opt_as_short_name(self, cmd, m_opt, tv):
            # Try multi opt short name
            out = cmd.run(m_opt.sn_to_cli(), tv, tv, tv)
            cmd.assert_args(out, (m_opt, [tv, tv, tv]))

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
            cmd.assert_args(out, (m_opt, [tv, tv, tv, tv, tv]))

        def test_flag_opt_long_name(self, cmd, oc_opt):
            # Try occurrence counter long name
            out = cmd.run(oc_opt.ln_to_cli())
            cmd.assert_args(out, (oc_opt, 1))

        def test_flag_opt_short_name(self, cmd, oc_opt):
            # Try occurrence counter short name
            out = cmd.run(oc_opt.sn_to_cli())
            cmd.assert_args(out, (oc_opt, 1))

        def test_flag_opt_ln_sn_stacking(self, cmd, oc_opt):
            # Try occurrence counter with both short and long name
            out = cmd.run(oc_opt.sn_to_cli(), oc_opt.ln_to_cli(), oc_opt.sn_to_cli(), oc_opt.ln_to_cli())
            cmd.assert_args(out, (oc_opt, 4))

        def test_error_on_flag_opt_name(self, cmd, oc_opt):
            # Try occurrence counter opt name
            out = cmd.gen_error(f"--{oc_opt.name}")
            assert self.err_msgs.unknown_opt_msg(oc_opt) in out

        def test_flag_opt_with_short_name_only(self, cmd, f_sn_opt):
            # Try flag opt short only
            out = cmd.run(f_sn_opt.sn_to_cli())
            cmd.assert_args(out, (f_sn_opt, 1))

        def test_error_on_short_name_only_opt_name(self, cmd, f_sn_opt):
            # Try flag opt short only opt name
            # Should generate an error
            out = cmd.gen_error(f"--{f_sn_opt.name}")
            assert self.err_msgs.unknown_opt_msg(f_sn_opt) in out

        def test_flag_opt_with_long_name_only(self, cmd, f_ln_opt):
            # Try flag opt long only
            out = cmd.run(f_ln_opt.ln_to_cli())
            cmd.assert_args(out, (f_ln_opt, 1))

        def test_error_on_long_name_only_opt_name(self, cmd, f_ln_opt):
            # Try flag opt long only opt name
            # Should generate an error
            out = cmd.gen_error(f"--{f_ln_opt.name}")
            assert self.err_msgs.unknown_opt_msg(f_ln_opt) in out

        def test_no_conflict_between_ln_and_sn(self, cmd, f_sn_opt, f_dupl_ln_sn_opt):
            # Try flag opt long name same as another's short name
            out = cmd.run("--f")
            cmd.assert_args(out, (f_dupl_ln_sn_opt, 1))

            out = cmd.run("--f", "-f")
            cmd.assert_args(out, (f_dupl_ln_sn_opt, 1), (f_sn_opt, 1))

        def test_short_name_aliasing(self, cmd, f_sn_al_opt):
            # Try short name aliases
            out = cmd.run(f'--{f_sn_al_opt.name}', '-a', '-b')
            cmd.assert_args(out, (f_sn_al_opt, 3))

        def test_sn_with_sn_aliases(self, cmd, f_sn_and_al_opt):
            # Try short name with aliases
            out = cmd.run('-c', '-d', '-e')
            cmd.assert_args(out, (f_sn_and_al_opt, 3))

        def test_error_on_opt_name_with_sn_and_sn_aliases(self, cmd, f_sn_and_al_opt):
            out = cmd.gen_error(f"--{f_sn_and_al_opt.name}", '-d', '-e')
            assert self.err_msgs.unknown_opt_msg(f_sn_and_al_opt) in out

        def test_long_name_aliasing(self, cmd, f_ln_al_opt):
            # Try long name aliases
            out = cmd.run(f"--{f_ln_al_opt.name}", '--fa', '--fb')
            cmd.assert_args(out, (f_ln_al_opt, 3))

        def test_ln_with_ln_aliases(self, cmd, f_ln_and_al_opt):
            # Try long name with aliases
            out = cmd.run('--fc', '--fd', '--fe')
            cmd.assert_args(out, (f_ln_and_al_opt, 3))

        def test_error_on_opt_name_with_ln_and_ln_aliases(self, cmd, f_ln_and_al_opt):
            out = cmd.gen_error(f"--{f_ln_and_al_opt.name}", '--fd', '--fe')
            assert self.err_msgs.unknown_opt_msg(f_ln_and_al_opt) in out

        def test_sn_and_ln_aliases_only(self, cmd, f_ln_sn_al_opt):
            # Try long/short name aliases
            out = cmd.run(f'--{f_ln_sn_al_opt.name}', '-z', '--sn_ln_1', '--sn_ln_2')
            cmd.assert_args(out, (f_ln_sn_al_opt, 4))

    class TestHiddenOpts(Common):
        _cmd = Common.cmd_testers.test_args.hidden_opt

        @pytest.fixture
        def h_opt(self, cmd):
            return cmd.hidden_opt

        @pytest.fixture
        def v_opt(self, cmd):
            return cmd.visible_opt

        def test_help_msg(self, cmd, v_opt):
            help = cmd.get_help_msg()
            help.assert_args(None)
            help.assert_opts("help", "vk", "v", v_opt)

        def test_hidden_opt_is_available(self, cmd, h_opt):
            out = cmd.run(h_opt.ln_to_cli())
            cmd.assert_args(out, (h_opt, 1))

        def test_visible_opt_only(self, cmd, v_opt):
            out = cmd.run(v_opt.ln_to_cli())
            cmd.assert_args(out, (v_opt, 1))

        def test_hidden_and_visible_opt_only(self, cmd, h_opt, v_opt):
            out = cmd.run(h_opt.ln_to_cli(), h_opt.ln_to_cli(), v_opt.ln_to_cli())
            cmd.assert_args(out, (h_opt, 2), (v_opt, 1))

        def test_error_on_random_opt(self, cmd):
            r_opt = "random"
            out = cmd.gen_error(f"--{r_opt}")
            assert self.err_msgs.unknown_opt_msg(r_opt) in out
