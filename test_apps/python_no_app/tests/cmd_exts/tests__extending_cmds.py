import pytest
from test_apps_shared_test_helpers.cli import CLIShared, CmdExtOpt

class Common(CLIShared):
    sv = "single_val"

class T_ExtendingCmds(CLIShared):
    class TestExtensionOpts(Common):
        cmd = CLIShared.python_plugin.plugin_test_args.extend(
            CLIShared.exts.exts["plugin.python_plugin.plugin_test_args"]["exts"],
            from_configs=CLIShared.exts.exts_workout_cfg
        )
        subcmd = CLIShared.python_plugin.plugin_test_args.subc.extend(
            CLIShared.exts.exts["plugin.python_plugin.plugin_test_args.subc"]["exts"],
            from_configs=CLIShared.exts.exts_workout_cfg
        )

        ext_flag = cmd.flag_extension
        ext_sa = cmd.single_val_opt
        ext_ma = cmd.multi_val_opt
        ext_ma_delim = cmd.multi_val_delim_opt
        ext_action = cmd.exts_workout_action
        ext_ha = cmd.hidden_opt
        sa = cmd.single_arg
        ma = cmd.multi_arg
        sv_opt = cmd.opt_taking_value
        f_opt = cmd.flag_opt
        opt_als = cmd.opt_with_aliases
        sn_only = cmd.sn_only

        mv = ["m0", "m1", "m2"]
        ov = "opt_value"
        ext_sv = "ext_single_val"
        ext_mv = ["mv0", "mv1"]
        ext_mvd = ["mvd0", "mvd1"]
        ext_rv = ["no_action"]

        def get_action_results(self, output, actions):
            retn = {}
            for action in actions:
                a = {}
                r = output.split(f"Start Action Before CMD: {action}")[1].strip()
                a["Before"], r = r.split(f"End Action Before CMD: {action}")
                r = output.split(f"Start Action After CMD: {action}")[1].strip()
                a["After"], r = r.split(f"End Action After CMD: {action}")
                retn[action] = a
            return retn

        def test_help_msg(self):
            help = self.cmd.get_help_msg()
            help.assert_args(self.sa, self.ma)
            help.assert_opts(
                self.ext_action,
                self.ext_flag,
                self.f_opt,
                "help",
                self.ext_ma,
                self.ext_ma_delim,
                self.sn_only,
                self.sv_opt,
                self.opt_als,
                self.ext_sa,
                "v",
                "vk",
            )
            help.assert_subcmds("help", self.cmd.subc)

            assert help.aux_exts == ['exts_workout']
            assert help.pl_exts == None
            assert help.app_exts == False

        def test_using_extended_opts(self):
            out = self.cmd.run(
                self.sv, *self.mv,
                self.ext_flag.ln_to_cli(), self.ext_flag.sn_to_cli(),
                self.cmd.opt_taking_value.ln_to_cli(), self.ov,
                self.ext_sa.sn_to_cli(), self.ext_sv,
                self.ext_ma.ln_to_cli(), *self.ext_mv, ','.join(self.ext_mvd),
                self.ext_ma_delim.ln_to_cli(), ','.join(self.ext_mvd),
                self.ext_action.ln_to_cli(), *self.ext_rv,
                self.cmd.flag_opt.ln_to_cli(),
                self.ext_ha.ln_to_cli(),
                self.cmd.flag_opt.ln_to_cli(),
            )
            self.cmd.assert_args(
                out,
                (self.cmd.single_arg, self.sv),
                (self.cmd.multi_arg, self.mv),
                (self.cmd.opt_taking_value, self.ov),
                (self.cmd.flag_opt, 2),
                (self.ext_flag, 2),
                (self.ext_sa, self.ext_sv),
                (self.ext_ma, [*self.ext_mv, ','.join(self.ext_mvd)]),
                (self.ext_ma_delim, self.ext_mvd),
                (self.ext_action, self.ext_rv),
                (self.ext_ha, 1),
            )

        def test_error_on_required_ext_opt(self):
            err = self.cmd.gen_error("single_val")
            assert self.err_msgs.missing_required_arg(self.ext_action) in err

        def test_accessing_cmd_args_from_ext(self):
            actions = ["show_cmd_args"]
            out = self.cmd.run(
                self.sv, *self.mv,
                self.ext_action.ln_to_cli(), *actions
            )
            self.cmd.assert_args(
                out,
                (self.cmd.single_arg, self.sv),
                (self.cmd.multi_arg, self.mv),
                (self.ext_action, actions),
            )
            r = self.get_action_results(out, actions)
            assert eval(r["show_cmd_args"]["Before"]) == {
                self.cmd.single_arg.name: self.sv,
                self.cmd.multi_arg.name: self.mv
            }
            assert eval(r["show_cmd_args"]["After"]) == {
                self.cmd.single_arg.name: self.sv,
                self.cmd.multi_arg.name: self.mv
            }

        def test_accessing_ext_args_directly(self):
            actions = ["show_ext_args"]
            out = self.cmd.run(
                self.sv, *self.mv,
                self.ext_action.ln_to_cli(), *actions
            )
            self.cmd.assert_args(
                out,
                (self.cmd.single_arg, self.sv),
                (self.cmd.multi_arg, self.mv),
                (self.ext_action, actions),
            )
            r = self.get_action_results(out, actions)
            assert eval(r["show_ext_args"]["Before"]) == {
                "aux.exts_workout": {self.ext_action.name: actions}
            }
            assert eval(r["show_ext_args"]["After"]) == {
                "aux.exts_workout": {self.ext_action.name: actions}
            }

        def test_manipulating_cmd_args_in_ext(self):
            actions = ["show_cmd_args", "set_arg__cmd__single_arg__updated"]
            out = self.cmd.run(
                self.sv, *self.mv,
                self.ext_action.ln_to_cli(), *actions
            )
            self.cmd.assert_args(
                out,
                (self.cmd.single_arg, "updated"),
                (self.cmd.multi_arg, self.mv),
                (self.ext_action, actions),
            )
            r = self.get_action_results(out, actions)
            assert eval(r["show_cmd_args"]["Before"]) == {
                self.cmd.single_arg.name: self.sv,
                self.cmd.multi_arg.name: self.mv
            }
            assert eval(r["show_cmd_args"]["After"]) == {
                self.cmd.single_arg.name: "updated",
                self.cmd.multi_arg.name: self.mv
            }

        def test_manipulating_ext_args_in_ext(self):
            actions = ["show_ext_args", "exts_workout__test_updating_args"]
            out = self.cmd.run(
                self.sv,
                self.ext_action.ln_to_cli(), *actions,
                self.ext_sa.sn_to_cli(), self.ext_sv,
                self.ext_ma.ln_to_cli(), *self.ext_mv,
                self.ext_ma_delim.ln_to_cli(), ','.join(self.ext_mvd),
                self.ext_flag.ln_to_cli(),
                self.ext_flag.ln_to_cli(),
            )
            self.cmd.assert_args(
                out,
                (self.cmd.single_arg, self.sv),
                (self.ext_flag, 3, {"Before": 2}),
                (self.ext_sa, "update_sv_opt", {"Before": self.ext_sv}),
                (self.ext_ma, self.ext_mv + ["update_mv_opt"], {"Before": self.ext_mv}),
                (self.ext_ma_delim, self.ext_mvd),
                (self.ext_action, actions),
                (CmdExtOpt("new_arg", src_name="exts_workout"), "new_arg_for_ext", {"Before": False})
            )

        def test_extending_subcommand(self):
            actions = ["no_action"]
            out = self.subcmd.run(
                self.sv,
                self.subcmd.flag_opt.ln_to_cli(),
                self.subcmd.exts_workout_action.ln_to_cli(),
                *actions,
            )
            self.subcmd.assert_args(
                out,
                (self.subcmd.single_arg, self.sv),
                (self.subcmd.flag_opt, 1),
                (self.subcmd.exts_workout_action, actions),
            )

        def test_subc_help_msg(self):
            subc = self.subcmd
            help = subc.get_help_msg()
            help.assert_args(subc.single_arg)
            help.assert_opts(
                subc.exts_workout_action,
                subc.flag_opt,
                "help",
                subc.subc_sn_only,
                subc.subc_opt_with_aliases,
                "v",
                "vk",
            )
            help.assert_subcmds(None)

            assert help.aux_exts == ['exts_workout']
            assert help.pl_exts == None
            assert help.app_exts == False

    @pytest.mark.skip
    def test_basic_extending_from_pl(self):
        help = self.global_cmds.eval.get_help_msg()
        assert help.pl_exts == []

    class TestExtensionStacking(Common):
        cmd = CLIShared.python_plugin.plugin_test_ext_stacking.extend(
            CLIShared.exts.exts["plugin.python_plugin.plugin_test_ext_stacking"]["exts"],
            from_configs=[
                CLIShared.exts.exts_workout_cfg,
                CLIShared.exts.pl_ext_stacking_from_aux_cfg
            ]
        )
        subc = CLIShared.python_plugin.plugin_test_ext_stacking.subc.extend(
            CLIShared.exts.exts["plugin.python_plugin.plugin_test_ext_stacking.subc"]["exts"],
            from_configs=[
                CLIShared.exts.exts_workout_cfg,
                CLIShared.exts.pl_ext_stacking_from_aux_cfg
            ],
        )

        def test_ext_stacking_help_msg(self):
            cmd = self.cmd
            help = cmd.get_help_msg()
            help.assert_args(cmd.single_arg)
            help.assert_opts(
                cmd.exts_workout_action,
                cmd.flag_opt,
                "help",
                cmd.pl_ext_stacking_from_aux_action,
                cmd.pl_ext_stacking_from_aux_flag,
                cmd.python_plugin_the_second_action,
                cmd.python_plugin_the_second_flag,
                "v",
                "vk",
            )
            help.assert_subcmds("help", self.subc)

            assert help.aux_exts == ['pl_ext_stacking_from_aux', 'exts_workout']
            assert help.pl_exts == ['python_plugin_the_second']
            assert help.app_exts == False

        def test_ext_stacking(self):
            cmd = self.cmd
            actions = [self.na]
            out = cmd.run(
                self.sv,
                cmd.flag_opt.ln_to_cli(),
                cmd.exts_workout_action.ln_to_cli(), *actions,
                cmd.pl_ext_stacking_from_aux_action.ln_to_cli(), *actions,
                cmd.pl_ext_stacking_from_aux_flag.ln_to_cli(), cmd.pl_ext_stacking_from_aux_flag.ln_to_cli(),
                cmd.python_plugin_the_second_action.ln_to_cli(), *actions,
                cmd.python_plugin_the_second_flag.ln_to_cli(),
            )
            cmd.assert_args(
                out,
                (cmd.single_arg, self.sv),
                (cmd.flag_opt, 1),
                (cmd.exts_workout_action, actions),
                (cmd.pl_ext_stacking_from_aux_action, actions),
                (cmd.pl_ext_stacking_from_aux_flag, 2),
                (cmd.python_plugin_the_second_action, actions),
                (cmd.python_plugin_the_second_flag, 1),
            )

        def test_manipulating_other_ext_args(self):
            cmd = self.cmd
            actions = [
                "inc_flag__cmd__flag_opt",
                "inc_flag__aux_ext__pl_ext_stacking_from_aux_flag",
                "inc_flag__plugin_ext__python_plugin_the_second_flag"
            ]
            out = cmd.run(
                self.sv,
                cmd.flag_opt.ln_to_cli(),
                cmd.exts_workout_action.ln_to_cli(), *actions,
                cmd.pl_ext_stacking_from_aux_flag.ln_to_cli(), cmd.pl_ext_stacking_from_aux_flag.ln_to_cli(),
                cmd.python_plugin_the_second_flag.ln_to_cli(),
            )
            cmd.assert_args(
                out,
                (cmd.single_arg, self.sv),
                (cmd.flag_opt, 2),
                (cmd.exts_workout_action, actions),
                (cmd.pl_ext_stacking_from_aux_flag, 3, {"Before": 2}),
                (cmd.python_plugin_the_second_flag, 2, {"Before": 1}),
            )

        def test_subc_ext_stacking_help_msg(self):
            subc = self.subc
            help = subc.get_help_msg()
            help.assert_args(subc.single_arg)
            help.assert_opts(
                subc.exts_workout_action_subc,
                subc.flag_opt,
                "help",
                subc.pl_ext_stacking_from_aux_action_subc,
                subc.pl_ext_stacking_from_aux_flag_subc,
                subc.python_plugin_the_second_action_subc,
                subc.python_plugin_the_second_flag_subc,
                "v",
                "vk",
            )
            help.assert_subcmds(None)

            assert help.aux_exts == ['pl_ext_stacking_from_aux', 'exts_workout']
            assert help.pl_exts == ['python_plugin_the_second']
            assert help.app_exts == False

        def test_subc_ext_stacking(self):
            actions = [self.na]
            subc = self.subc
            out = subc.run(
                self.sv,
                subc.flag_opt.ln_to_cli(),
                subc.exts_workout_action_subc.ln_to_cli(), *actions,
                subc.pl_ext_stacking_from_aux_flag_subc.ln_to_cli(), subc.pl_ext_stacking_from_aux_flag_subc.ln_to_cli(),
                subc.python_plugin_the_second_flag_subc.ln_to_cli(),
            )
            subc.assert_args(
                out,
                (subc.single_arg, self.sv),
                (subc.flag_opt, 1),
                (subc.exts_workout_action_subc, actions),
                (subc.pl_ext_stacking_from_aux_flag_subc, 2),
                (subc.python_plugin_the_second_flag_subc, 1),
            )
        
    class TestExtendingAuxCmds(Common):
        cmd = CLIShared.aux.ns.dummy_cmds.dummy_cmd.extend(
            CLIShared.exts.exts["aux.dummy_cmds.dummy_cmd"]["exts"],
            from_configs=[
                CLIShared.exts.exts_workout_cfg,
                CLIShared.exts.pl_ext_stacking_from_aux_cfg
            ],
            with_env=CLIShared.exts.exts["aux.dummy_cmds.dummy_cmd"]["env"]
        )
        subc = CLIShared.aux.ns.dummy_cmds.dummy_cmd.subc.extend(
            CLIShared.exts.exts["aux.dummy_cmds.dummy_cmd.subc"]["exts"],
            from_configs=[
                CLIShared.exts.exts_workout_cfg,
                CLIShared.exts.pl_ext_stacking_from_aux_cfg
            ],
            with_env=CLIShared.exts.exts["aux.dummy_cmds.dummy_cmd"]["env"]
        )

        @property
        def na(self):
            return "no_action"

        def test_extending_aux_cmd_help_msg(self):
            cmd = self.cmd
            help = cmd.get_help_msg()
            help.assert_args(cmd.action_arg)
            help.assert_opts(
                cmd.exts_workout_action,
                cmd.exts_workout_flag,
                "help",
                cmd.pl_ext_stacking_from_aux_action,
                cmd.pl_ext_stacking_from_aux_flag,
                cmd.python_plugin_action,
                cmd.python_plugin_flag,
                cmd.python_plugin_the_second_action,
                cmd.python_plugin_the_second_flag,
                "v",
                "vk",
            )
            help.assert_subcmds("help", cmd.subc)

            assert help.aux_exts == ['pl_ext_stacking_from_aux', 'exts_workout']
            assert help.pl_exts == ['python_plugin', 'python_plugin_the_second']
            assert help.app_exts == False

        def test_extending_aux_cmd(self):
            cmd = self.cmd
            out = cmd.run(
                self.na,
                cmd.pl_ext_stacking_from_aux_flag.ln_to_cli(), cmd.pl_ext_stacking_from_aux_flag.ln_to_cli(),
                cmd.python_plugin_the_second_flag.ln_to_cli(),
                cmd.python_plugin_flag.ln_to_cli(),
            )
            cmd.assert_args(
                out,
                (cmd.action_arg, [self.na]),
                (cmd.exts_workout_action, None),
                (cmd.pl_ext_stacking_from_aux_flag, 2),
                (cmd.python_plugin_the_second_flag, 1),
                (cmd.python_plugin_flag, 1),
            )

        def test_manipulating_args_from_aux_exts(self):
            cmd = self.cmd
            actions = [
                "inc_flag__aux_ext__pl_ext_stacking_from_aux_flag",
                "inc_flag__plugin_ext__python_plugin_the_second_flag",
                "set_flag__plugin_ext__python_plugin_flag"
            ]
            out = cmd.run(
                self.na,
                cmd.exts_workout_action.ln_to_cli(), *actions,
                cmd.pl_ext_stacking_from_aux_flag.ln_to_cli(), cmd.pl_ext_stacking_from_aux_flag.ln_to_cli(),
                cmd.python_plugin_the_second_flag.ln_to_cli(),
            )
            cmd.assert_args(
                out,
                (cmd.action_arg, [self.na]),
                (cmd.exts_workout_action, actions),
                (cmd.python_plugin_flag, -1, {"Before": None}),
                (cmd.pl_ext_stacking_from_aux_flag, 3, {"Before": 2}),
                (cmd.python_plugin_the_second_flag, 2, {"Before": 1}),
                finalize_ext_args=lambda ext_args: ext_args['python_plugin'].update({'Before Cmd': []})
            )

        def test_extending_aux_cmd_help_msg_subc(self):
            cmd = self.subc
            help = cmd.get_help_msg()
            help.assert_args(cmd.action_arg)
            help.assert_opts(
                cmd.exts_workout_action,
                cmd.exts_workout_flag_subc,
                cmd.flag_opt,
                "help",
                cmd.pl_ext_stacking_from_aux_action_subc,
                cmd.pl_ext_stacking_from_aux_flag_subc,
                cmd.python_plugin_action_subc,
                cmd.python_plugin_flag_subc,
                cmd.python_plugin_the_second_action_subc,
                cmd.python_plugin_the_second_flag_subc,
                "v",
                "vk",
            )
            help.assert_subcmds(None)

            assert help.aux_exts == ['pl_ext_stacking_from_aux', 'exts_workout']
            assert help.pl_exts == ['python_plugin', 'python_plugin_the_second']
            assert help.app_exts == False

        def test_extending_aux_subcmd(self):
            cmd = self.subc
            actions = ["no_action"]
            out = cmd.run(
                self.na,
                cmd.flag_opt.ln_to_cli(),
                cmd.exts_workout_action.ln_to_cli(), *actions,
                cmd.pl_ext_stacking_from_aux_flag_subc.ln_to_cli(),
                cmd.pl_ext_stacking_from_aux_flag_subc.ln_to_cli(),
                cmd.python_plugin_the_second_flag_subc.ln_to_cli(),
                cmd.python_plugin_flag_subc.ln_to_cli(),
            )
            print(out)
            cmd.assert_args(
                out,
                (cmd.action_arg, [self.na]),
                (cmd.flag_opt, 1),
                (cmd.exts_workout_action, actions),
                (cmd.exts_workout_flag_subc, None),
                (cmd.pl_ext_stacking_from_aux_flag_subc, 2),
                (cmd.python_plugin_the_second_flag_subc, 1),
                (cmd.python_plugin_flag_subc, 1),
            )

        def test_manipulating_args_from_aux_subcmd(self):
            cmd = self.subc
            actions = [
                "inc_flag__cmd__flag_opt",
                "inc_multi_arg__cmd__action_arg__updated",
                "inc_flag__aux_ext__pl_ext_stacking_from_aux_flag_subc",
                "inc_flag__plugin_ext__python_plugin_the_second_flag_subc",
                "inc_flag__plugin_ext__python_plugin_flag_subc"
            ]
            out = cmd.run(
                self.na,
                cmd.flag_opt.ln_to_cli(),
                cmd.exts_workout_action.ln_to_cli(), *actions,
                cmd.pl_ext_stacking_from_aux_flag_subc.ln_to_cli(),
                cmd.pl_ext_stacking_from_aux_flag_subc.ln_to_cli(),
                cmd.python_plugin_the_second_flag_subc.ln_to_cli(),
                cmd.python_plugin_flag_subc.ln_to_cli(),
            )
            cmd.assert_args(
                out,
                (cmd.action_arg, ["updated"]),
                (cmd.flag_opt, 2),
                (cmd.exts_workout_action, actions),
                (cmd.exts_workout_flag_subc, None),
                (cmd.pl_ext_stacking_from_aux_flag_subc, 3, {"Before": 2}),
                (cmd.python_plugin_the_second_flag_subc, 2, {"Before": 1}),
                (cmd.python_plugin_flag_subc, 2, {"Before": 1}),
            )

    @pytest.mark.skip
    def test_hidden_exts_full_name(self):
        fail

    def test_extending_origen_cmd_from_plugin(self):
        ''' Test each global command is extendable'''
        cmd = self.global_cmds.eval
        cmd = cmd.extend(
            CLIShared.exts.exts["generic_core_ext"]["exts"],
            from_configs=[CLIShared.exts.core_cmd_exts_cfg]
        )

        help = cmd.get_help_msg()
        help.assert_args(cmd.code)
        help.assert_opts(
            cmd.core_cmd_exts_generic_core_ext,
            "help",
            cmd.pl_ext_cmds_generic_ext,
            "v",
            "vk",
        )
        help.assert_subcmds(None)
        assert help.aux_exts == ['core_cmd_exts']
        assert help.pl_exts == ['pl_ext_cmds']
        assert help.app_exts == False

        d = cmd.global_demo("minimal")
        out = d.run(add_args=[
            cmd.core_cmd_exts_generic_core_ext.ln_to_cli(),
            cmd.pl_ext_cmds_generic_ext.ln_to_cli(),
            cmd.pl_ext_cmds_generic_ext.ln_to_cli(),
        ])
        d.assert_present(out)
        cmd.core_cmd_exts_generic_core_ext.assert_present(1, out)
        cmd.pl_ext_cmds_generic_ext.assert_present(2, out)
