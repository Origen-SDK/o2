import pytest, os, origen
from origen.helpers.regressions.cli.command import SrcTypes
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

    missing_ext_impl_cmd = CLICommon.python_plugin.plugin_says_hi.extend(
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
        help.assert_args()
        help.assert_opts(
            cmd.app_ext_missing_impl,
            "help",
            cmd.loudly,
            "m", "nt", "t",
            cmd.to,
            "v", 'vk',
            cmd.times,
        )

        assert help.aux_exts == None
        assert help.pl_exts == None
        assert help.app_exts == True

        out = cmd.run()
        r = origen.app.root.joinpath('example/commands/extensions')
        assert "Could not find implementation for app extension" in out
        assert f"From root '{r}', searched:" in out
        assert f"plugin.python_plugin.plugin_says_hi.py" in out
        assert f"plugin{os.sep}python_plugin.plugin_says_hi.py" in out
        assert f"plugin{os.sep}python_plugin{os.sep}plugin_says_hi.py" in out

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

    class TestAppExtConflicts(CLICommon):
        ''' Test what happens when app/plugin/aux extensions conflict
        Only need a subset to ensure app messaging is okay '''
        config = CLICommon.app_cmds.conflict_exts["python_plugin.plugin_test_args"]
        cmd = CLICommon.python_plugin.plugin_test_args.extend(
            config["exts"],
            from_configs=[config["cfg"]],
            with_env=config["env"]
        )
        _exts = CLICommon.exts.partition_exts(config["exts"])

        @pytest.fixture
        def exts(self):
            return self._exts

        @pytest.fixture
        def cmd_help(self):
            if not hasattr(self, "_cmd_help"):
                self._cmd_help = self.cmd.get_help_msg()
            return self._cmd_help

        def test_help_msg(self, exts, cmd_help):
            cmd = self.cmd
            help = cmd_help
            help.assert_args(cmd.single_arg, cmd.multi_arg)
            help.assert_opts(
                exts.app.app_opt,
                exts.ec.ec_opt,
                exts.app.tas_iln,
                exts.ec.tas_iln,
                cmd.flag_opt,
                "h", "m",
                cmd.sn_only,
                "nt",
                cmd.opt_taking_value,
                cmd.opt_with_aliases,
                "t",
                exts.tas.tas_iln,
                "v", "vk",
                exts.tas.tas_opt,
            )
            help.assert_subcmds("help", cmd.subc)

            assert help.aux_exts == ["ext_conflicts"]
            assert help.pl_exts == ["test_apps_shared_test_helpers"]
            assert help.app_exts == True

        def test_conflict_msgs(self, exts, cmd_help):
            cmd = self.cmd
            cmd_conflicts = cmd_help.logged_errors
            conflicts = [
                ["repeated_sna", exts.app.app_opt, "g", 2],
                ["reserved_prefix_lna", exts.app.app_opt, "ext_opt.res"],
                ["duplicate", exts.app.app_opt, 0],
                ["reserved_prefix_opt_name", "ext_opt.res_app_opt", exts.app.app_opt.displayed],

                ["sna", "sna", exts.tas.tas_opt, cmd.opt_with_aliases, "b"],

                ["sna", "sna", exts.ec.ec_opt, cmd.opt_with_aliases, "a"],
                ["sna", "sna", exts.ec.ec_opt, cmd.opt_with_aliases, "b"],
                ["sna", "sna", exts.ec.ec_opt, exts.tas.tas_opt, "c"],
                ["sn", "sn", exts.ec.tas_iln, cmd.sn_only, "n"],
                ["iln", "iln", exts.ec.tas_iln, exts.tas.tas_iln],
                ["lna", "lna", exts.ec.tas_iln, exts.tas.tas_opt, "t_opt"],

                ["sn", "sna", exts.app.app_opt, cmd.opt_with_aliases, "a"],
                ["lna", "ln", exts.app.app_opt, exts.tas.tas_opt, "tas"],
                ["lna", "ln", exts.app.app_opt, cmd.flag_opt, "flag"],
                ["sna", "sna", exts.app.app_opt, exts.tas.tas_opt, "c"],
                ["sna", "sna", exts.app.app_opt, exts.ec.ec_opt, "d"],
                ["iln", "iln", exts.app.tas_iln, exts.tas.tas_iln],
            ]
            for c in reversed(conflicts):
                m = cmd_conflicts.pop()
                print(m)
                assert self.err_msgs.to_conflict_msg(cmd, c) in m

        def test_exts(self, exts):
            cmd = self.cmd
            sa = "sa_val"
            ma = ["a", "b", "c"]
            opt = "opt_val"
            out = cmd.run(
                sa, *ma,
                "--app", "--app_flag", "-g", "-g", "--app_flag", "-g",
                "-e", "--ec", "-d",
                "--ext_opt.app.tas_iln",
                "--ext_opt.aux.ext_conflicts.tas_iln", "--ext_opt.aux.ext_conflicts.tas_iln",
                "--flag", "--flag", "--flag",
                "-n", "-n", "-n", "-n",
                "--opt", opt,
                "--opt_alias", "-a", "-b", "-a", "-b",
                "--tas_iln", "--tas_iln", "--tas_iln",
                "--tas", "-z", "-c", "--tas", "-z", "-c", "--t_opt",
            )
            cmd.assert_args(
                out,
                (cmd.single_arg, sa),
                (cmd.multi_arg, ma),
                (cmd.flag_opt, 3),
                (cmd.sn_only, 4),
                (cmd.opt_taking_value, opt),
                (cmd.opt_with_aliases, 5),

                (exts.app.app_opt, 6),
                (exts.app.tas_iln, 1),
                (exts.ec.ec_opt, 3),
                (exts.ec.tas_iln, 2),
                (exts.tas.tas_opt, 7),
                (exts.tas.tas_iln, 3),
            )
