import pytest, os, origen
from origen.helpers.regressions.cli.command import SrcTypes #, CmdExtOpt
from .shared import CLICommon, CmdExtOpt

class T_ExtendingFromAppCmds(CLICommon):
    extend_core_cmds_env = {"ORIGEN_APP_EXTEND_CORE_CMDS": "1"}
    extend_pl_test_ext_stacking = {"ORIGEN_APP_EXT_PL_TEST_EXT_STACKING": "1"}
    exts = CmdExtOpt.from_src(
        "example",
        SrcTypes.APP,
        CmdExtOpt(
            "generic_app_ext_action",
            help="Action from the app",
            multi=True,
        ),
        CmdExtOpt(
            "generic_app_ext_flag",
            help="Flag ext from the app",
        ),
    )
    core_cmd = origen.helpers.regressions.cli.CLI.in_app_cmds.eval.extend(
        exts,
        with_env=extend_core_cmds_env,
        from_configs=CLICommon.configs.suppress_plugin_collecting_config,
    )
    stacked_core_cmd = origen.helpers.regressions.cli.CLI.in_app_cmds.eval.extend(
        [
            *CLICommon.exts.exts["generic_core_ext"]["exts"],
            *exts
        ],
        from_configs=[CLICommon.exts.core_cmd_exts_cfg],
        with_env=extend_core_cmds_env,
    )
    py_pl_cmd = CLICommon.python_plugin.plugin_test_ext_stacking.extend(
        [
            *CLICommon.exts.exts["plugin.python_plugin.plugin_test_ext_stacking"]["exts"][1:3],
            *CLICommon.exts.test_apps_shared_generic_exts,
            *exts
        ],
        from_configs=[CLICommon.exts.pl_ext_stacking_from_aux_cfg],
        with_env=extend_pl_test_ext_stacking,
    )
    py_pl_subc = CLICommon.python_plugin.plugin_test_ext_stacking.subc.extend(
        [
            *CLICommon.exts.exts["plugin.python_plugin.plugin_test_ext_stacking.subc"]["exts"][1:3],
            *CLICommon.exts.test_apps_shared_generic_exts,
            *exts
        ],
        from_configs=[CLICommon.exts.pl_ext_stacking_from_aux_cfg],
        with_env=extend_pl_test_ext_stacking,
    )
    aux_cmd = CLICommon.aux.ns.dummy_cmds.dummy_cmd.extend(
        [
            *CLICommon.exts.exts["aux.dummy_cmds.dummy_cmd"]["exts"][2:6],
            *exts
        ],
        from_configs=[CLICommon.exts.pl_ext_stacking_from_aux_cfg],
        with_env=CLICommon.exts.exts["aux.dummy_cmds.dummy_cmd"]["env"]
    )
    aux_subc = CLICommon.aux.ns.dummy_cmds.dummy_cmd.subc.extend(
        [
            *CLICommon.exts.exts["aux.dummy_cmds.dummy_cmd.subc"]["exts"][2:6],
            *exts
        ],
        from_configs=[CLICommon.exts.pl_ext_stacking_from_aux_cfg],
        with_env=CLICommon.exts.exts["aux.dummy_cmds.dummy_cmd"]["env"]
    )
    na = "no_action"

    missing_ext_impl_cmd = CLICommon.python_plugin.plugin_test_args.extend(
        CmdExtOpt.from_src(
            "example",
            SrcTypes.APP,
            CmdExtOpt(
                "app_ext_missing_impl",
                help="App extension missing the implementation",
            ),
        ),
        with_env={"ORIGEN_APP_PL_CMD_MISSING_EXT_IMPL": "1"}
    )

    def test_extending_global_cmds(self):
        cmd = self.core_cmd
        help = cmd.get_help_msg()
        help.assert_args(cmd.code)
        help.assert_opts(
            cmd.generic_app_ext_action,
            cmd.generic_app_ext_flag,
            *self.in_app_cmds.standard_opts()
        )

        assert help.aux_exts == None
        assert help.pl_exts == None
        assert help.app_exts == True

        d = cmd.demos["minimal"]
        out = d.run()
        cmd.generic_app_ext_flag.assert_present(None, out)
        d.assert_present(out)

        out = d.run(add_args=[cmd.generic_app_ext_action.ln_to_cli(), self.na, cmd.generic_app_ext_flag.ln_to_cli()])
        cmd.generic_app_ext_action.assert_present([self.na], out)
        cmd.generic_app_ext_flag.assert_present(1, out)
        d.assert_present(out)

    def test_stacking_pl_aux_and_app_ext(self):
        cmd = self.stacked_core_cmd
        help = cmd.get_help_msg()
        help.assert_args(cmd.code)
        help.assert_opts(
            cmd.core_cmd_exts_generic_core_ext,
            cmd.generic_app_ext_action,
            cmd.generic_app_ext_flag,
            "help", "mode", "no_targets",
            cmd.pl_ext_cmds_generic_ext,
            "targets", "v", 'vk'
        )
        assert help.aux_exts == ['core_cmd_exts']
        assert help.pl_exts == ['pl_ext_cmds']
        assert help.app_exts == True

        d = cmd.demos["minimal"]
        out = d.run(add_args=[
            cmd.generic_app_ext_action.ln_to_cli(), self.na,
            cmd.generic_app_ext_flag.ln_to_cli(),
            cmd.pl_ext_cmds_generic_ext.ln_to_cli(),
            cmd.core_cmd_exts_generic_core_ext.ln_to_cli(),
        ])
        cmd.generic_app_ext_action.assert_present([self.na], out)
        cmd.generic_app_ext_flag.assert_present(1, out)
        cmd.pl_ext_cmds_generic_ext.assert_present(1, out)
        cmd.core_cmd_exts_generic_core_ext.assert_present(1, out)
        d.assert_present(out)

    def test_extending_pl_cmd(self):
        cmd = self.py_pl_cmd
        help = cmd.get_help_msg()
        help.assert_args(cmd.single_arg)
        help.assert_opts(
            cmd.flag_opt,
            cmd.generic_app_ext_action,
            cmd.generic_app_ext_flag,
            "help", "m", "nt",
            cmd.pl_ext_stacking_from_aux_action,
            cmd.pl_ext_stacking_from_aux_flag,
            "t",
            cmd.test_apps_shared_ext_action,
            cmd.test_apps_shared_ext_flag,
            "v", 'vk',
        )
        help.assert_subcmds("help", cmd.subc)

        assert help.aux_exts == ['pl_ext_stacking_from_aux']
        assert help.pl_exts == ['test_apps_shared_test_helpers']
        assert help.app_exts == True

        out = cmd.run()
        cmd.assert_args(
            out,
            (cmd.single_arg, None),
            (cmd.pl_ext_stacking_from_aux_action, None),
            (cmd.test_apps_shared_ext_action, None),
            (cmd.generic_app_ext_flag, None),
        )

        sa_v = "single_arg_val"
        out = cmd.run(sa_v, cmd.generic_app_ext_action.ln_to_cli(), self.na, cmd.generic_app_ext_flag.ln_to_cli())
        cmd.assert_args(
            out,
            (cmd.single_arg, sa_v),
            (cmd.pl_ext_stacking_from_aux_action, None),
            (cmd.test_apps_shared_ext_action, None),
            (cmd.generic_app_ext_action, [self.na]),
            (cmd.generic_app_ext_flag, 1),
        )

    def test_extending_pl_subc(self):
        cmd = self.py_pl_subc
        help = cmd.get_help_msg()
        help.assert_args(cmd.single_arg)
        help.assert_opts(
            cmd.flag_opt,
            cmd.generic_app_ext_action,
            cmd.generic_app_ext_flag,
            "help", "m", "nt",
            cmd.pl_ext_stacking_from_aux_action_subc,
            cmd.pl_ext_stacking_from_aux_flag_subc,
            "t",
            cmd.test_apps_shared_ext_action,
            cmd.test_apps_shared_ext_flag,
            "v", 'vk',
        )
        help.assert_subcmds(None)

        out = cmd.run()
        cmd.assert_args(
            out,
            (cmd.single_arg, None),
            (cmd.pl_ext_stacking_from_aux_action_subc, None),
            (cmd.test_apps_shared_ext_action, None),
            (cmd.generic_app_ext_action, None),
            (cmd.generic_app_ext_flag, None),
        )

        sa_v = "single_arg_val"
        out = cmd.run(sa_v, cmd.generic_app_ext_action.ln_to_cli(), self.na, cmd.generic_app_ext_flag.ln_to_cli())
        cmd.assert_args(
            out,
            (cmd.single_arg, sa_v),
            (cmd.pl_ext_stacking_from_aux_action_subc, None),
            (cmd.test_apps_shared_ext_action, None),
            (cmd.generic_app_ext_action, [self.na]),
            (cmd.generic_app_ext_flag, 1),
        )

    def test_extending_aux_cmd(self):
        cmd = self.aux_cmd
        help = cmd.get_help_msg()
        help.assert_args(cmd.action_arg)
        help.assert_opts(
            cmd.generic_app_ext_action,
            cmd.generic_app_ext_flag,
            "help", "m", "nt",
            cmd.pl_ext_stacking_from_aux_action,
            cmd.pl_ext_stacking_from_aux_flag,
            cmd.python_plugin_action,
            cmd.python_plugin_flag,
            "t", "v", 'vk',
        )
        help.assert_subcmds("help", cmd.subc)

        out = cmd.run()
        cmd.assert_args(
            out,
            (cmd.action_arg, None),
            (cmd.pl_ext_stacking_from_aux_action, None),
            (cmd.python_plugin_action, None),
            (cmd.generic_app_ext_action, None),
            (cmd.generic_app_ext_flag, None),
        )

        sa_v = "single_arg_val"
        out = cmd.run(sa_v, cmd.generic_app_ext_action.ln_to_cli(), self.na, cmd.generic_app_ext_flag.ln_to_cli())
        cmd.assert_args(
            out,
            (cmd.action_arg, [sa_v]),
            (cmd.pl_ext_stacking_from_aux_action, None),
            (cmd.python_plugin_action, None),
            (cmd.generic_app_ext_action, [self.na]),
            (cmd.generic_app_ext_flag, 1),
        )

    def test_extending_aux_subc(self):
        cmd = self.aux_subc
        help = cmd.get_help_msg()
        help.assert_args(cmd.action_arg)
        help.assert_opts(
            cmd.flag_opt,
            cmd.generic_app_ext_action,
            cmd.generic_app_ext_flag,
            "help", "m", "nt",
            cmd.pl_ext_stacking_from_aux_action_subc,
            cmd.pl_ext_stacking_from_aux_flag_subc,
            cmd.python_plugin_action_subc,
            cmd.python_plugin_flag_subc,
            "t", "v", 'vk',
        )
        help.assert_subcmds(None)

        out = cmd.run()
        cmd.assert_args(
            out,
            (cmd.action_arg, None),
            (cmd.pl_ext_stacking_from_aux_action_subc, None),
            (cmd.python_plugin_action_subc, None),
            (cmd.generic_app_ext_action, None),
            (cmd.generic_app_ext_flag, None),
        )

        sa_v = "single_arg_val"
        out = cmd.run(sa_v, cmd.generic_app_ext_action.ln_to_cli(), self.na, cmd.generic_app_ext_flag.ln_to_cli())
        cmd.assert_args(
            out,
            (cmd.action_arg, [sa_v]),
            (cmd.pl_ext_stacking_from_aux_action_subc, None),
            (cmd.python_plugin_action_subc, None),
            (cmd.generic_app_ext_action, [self.na]),
            (cmd.generic_app_ext_flag, 1),
        )

    def test_error_msg_on_missing_implementation(self):
        cmd = self.missing_ext_impl_cmd
        help = cmd.get_help_msg()
        help.assert_args(cmd.single_arg, cmd.multi_arg)
        help.assert_opts(
            cmd.app_ext_missing_impl,
            cmd.flag_opt,
            "help", "m", "nt",
            cmd.opt_taking_value,
            "t", "v", 'vk'
        )

        assert help.aux_exts == None
        assert help.pl_exts == None
        assert help.app_exts == True

        out = cmd.run()
        r = origen.app.root.joinpath('example/commands/extensions')
        assert "Could not find implementation for app extension 'None'" in out
        assert f"From root '{r}', searched:" in out
        assert f"plugin.python_plugin.plugin_test_args.py" in out
        assert f"plugin{os.sep}python_plugin.plugin_test_args.py" in out
        assert f"plugin{os.sep}python_plugin{os.sep}plugin_test_args.py" in out

    @pytest.mark.skip
    def error_msg_on_extending_unknown_cmd(self):
        fail

    @pytest.mark.skip
    def test_error_in_before(self):
        fail

    @pytest.mark.skip
    def test_error_in_after(self):
        fail

    @pytest.mark.skip
    def test_error_in_cleanup(self):
        fail