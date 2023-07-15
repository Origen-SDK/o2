import pytest
from test_apps_shared_test_helpers.cli import CLIShared

class Common(CLIShared):
    @pytest.fixture
    def exts(self):
        return self.config["exts_by_ns"]

    @pytest.fixture
    def ec(self, exts):
        return exts.ec

    @pytest.fixture
    def tas(self, exts):
        return exts.tas

class T_ExtConflicts(CLIShared):
    class TestWithPLCmd(Common):
        config = CLIShared.exts.ext_conflicts["plugin.python_plugin.plugin_test_args"]
        conflicts = config["conflicts_list"]
        cmd = CLIShared.python_plugin.plugin_test_args.extend(
            config["exts"],
            from_configs=[config["cfg"]],
            with_env=config["env"]
        )

        @classmethod
        def setup_class(cls):
            cls.cmd_help = cls.cmd.get_help_msg()
            cls.cmd_conflicts = cls.cmd_help.logged_errors

        @classmethod
        def teardown_class(cls):
            delattr(cls, "cmd_help")
            delattr(cls, "cmd_conflicts")

        def test_help_msg(self, ec, tas):
            # Check help message
            cmd = self.cmd
            help = self.cmd_help

            help.assert_args(cmd.single_arg, cmd.multi_arg)
            help.assert_opts(
                ec.aux_conflict_ln_and_aliases,
                ec.conflict_sn,
                ec.alias,
                ec.flag,
                ec.pl_aux_conflict,
                tas.flag,
                ec.ext_opt_in_ln,
                ec.ext_opt_in_lna,
                ec.ext_self_conflict,
                ec.ext_self_conflict_2,
                cmd.flag_opt,
                ec.repeated_sn_and_aliases,
                "help",
                cmd.sn_only,
                ec.ns_self_conflict,
                cmd.opt_taking_value,
                tas.opt_taking_value,
                cmd.opt_with_aliases,
                ec.opt_taking_value,
                tas.pl_aux_conflict,
                ec.pl_aux_sn_conflict_aux,
                tas.pl_conflict_ln_and_aliases,
                tas.pl_aux_sn_conflict_pl,
                ec.same_iln_and_ln_alias,
                ec.same_ln_and_ln_alias,
                ec.single_arg,
                ec.subc,
                "v",
                "vk",
            )
            help.assert_subcmds("help", cmd.subc)

            assert help.aux_exts == ['ext_conflicts']
            assert help.pl_exts == ['test_apps_shared_test_helpers']
            assert help.app_exts == False

        def test_conflict_messages(self):
            for c in reversed(self.conflicts):
                assert self.err_msgs.to_conflict_msg(self.cmd, c) in self.cmd_conflicts.pop()

        def test_error_messages_checked(self):
            assert len(self.cmd_conflicts) == 0

        def test_ext_conflicts_resolve_correctly(self, tas, ec):
            cmd = self.cmd
            out = cmd.run(
                "abc", "d", "e",
                "--opt", "z",
                "--flag", "--flag", "--flag",
                "-n",
                "-a", "--alias", "-b",

                "--single_arg",
                "--otv",
                "--conflict_sn",
                "--ext_opt.aux.ext_conflicts.pl_aux_conflict",
                "--pl_aux_sn_conflict_aux",
                "--other_alias_aux", "-d",
                "--ext_opt.aux.ext_conflicts.flag",
                "--ext_opt.aux.ext_conflicts.alias", "--alias_aux",
                "--subc",
                "--ns_self_conflict",
                "--ext_self_conflict",
                "--ext_self_conflict_2_1", "--ext_self_conflict_2",
                "--ext_opt_in_ln",
                "--ext_opt_in_lna", "--ext_opt_in_lna_2",
                "--same_ln_and_ln_alias",
                "--same_iln_and_ln_alias",
                "-g", "-e", "--repeated_lna",

                "--pl_aux_conflict", "--pl_aux_conflict", '--pl_aux_conflict', '--pl_aux_conflict',
                "-s", "-s",
                "--opt_taking_value",
                "--other_alias_pl", "--pl_conflict_ln_and_aliases", "-c",
                "--ext_opt.plugin.test_apps_shared_test_helpers.flag", "--ext_opt.plugin.test_apps_shared_test_helpers.flag",
            )
            cmd.assert_args(
                out,
                (cmd.single_arg, "abc"),
                (cmd.multi_arg, ["d", "e"]),
                (cmd.flag_opt, 3),
                (cmd.sn_only, 1),
                (cmd.opt_taking_value, "z"),
                (cmd.opt_with_aliases, 3),

                (ec.single_arg, 1),
                (ec.opt_taking_value, 1),
                (ec.conflict_sn, 1),
                (ec.pl_aux_conflict, 1),
                (ec.pl_aux_sn_conflict_aux, 1),
                (ec.aux_conflict_ln_and_aliases, 2),
                (ec.flag, 1),
                (ec.alias, 2),
                (ec.subc, 1),
                (ec.ns_self_conflict, 1),
                (ec.ext_self_conflict, 1),
                (ec.ext_self_conflict_2, 2),
                (ec.ext_opt_in_ln, 1),
                (ec.ext_opt_in_lna, 2),
                (ec.same_ln_and_ln_alias, 1),
                (ec.same_iln_and_ln_alias, 1),
                (ec.repeated_sn_and_aliases, 3),

                (tas.pl_aux_conflict, 4),
                (tas.pl_aux_sn_conflict_pl, 2),
                (tas.opt_taking_value, 1),
                (tas.pl_conflict_ln_and_aliases, 3),
                (tas.flag, 2),
            )

    class TestWithPLSubcmd(Common):
        config = CLIShared.exts.ext_conflicts["plugin.python_plugin.plugin_test_args.subc"]
        cmd = CLIShared.python_plugin.plugin_test_args.subc.extend(
            config["exts"],
            from_configs=[config["cfg"]],
            with_env=config["env"]
        )

        @classmethod
        def setup_class(cls):
            cls.cmd_help = cls.cmd.get_help_msg()
            cls.cmd_conflicts = cls.cmd_help.logged_errors

        @classmethod
        def teardown_class(cls):
            delattr(cls, "cmd_help")
            delattr(cls, "cmd_conflicts")

        def test_help_msg(self, ec, tas):
            cmd = self.cmd
            help = self.cmd_help

            help.assert_args(cmd.single_arg)
            help.assert_opts(
                tas.subc_pl_aux_conflict,
                ec.flag_opt,
                ec.more_conflicts,
                tas.flag_opt,
                cmd.flag_opt,
                "help",
                tas.more_conflicts,
                cmd.subc_sn_only,
                cmd.subc_opt_with_aliases,
                ec.subc_pl_aux_conflict,
                "v",
                "vk",
            )
            help.assert_subcmds(None)

            assert help.aux_exts == ['ext_conflicts']
            assert help.pl_exts == ['test_apps_shared_test_helpers']
            assert help.app_exts == False

        def test_conflict_messages(self):
            cmd = self.cmd
            conflicts = self.config["conflicts_list"]

            for c in reversed(conflicts):
                m = self.err_msgs.to_conflict_msg(cmd, c)
                assert m in self.cmd_conflicts.pop()

        def test_all_errors_checked(self):
            assert len(self.cmd_conflicts) == 0

        def test_conflicts_resolve_correctly(self, tas, ec):
            cmd = self.cmd
            out = cmd.run(
                "argv",
                "--flag_opt",
                "-n", "-n",
                "-a", "--subc_opt", "-b", "--subc_alias",

                '--pl0', '--pl1', '-c', '--subc_pl_aux',
                '--ext_opt.plugin.test_apps_shared_test_helpers.flag_opt', '--ext_opt.plugin.test_apps_shared_test_helpers.flag_opt',
                '--more_conflicts', '-d', '--more_conflicts',

                "-e", "--aux0", "--subc_pl_aux_conflict",
                "--ext_opt.aux.ext_conflicts.flag_opt", "--ext_opt.aux.ext_conflicts.flag_opt", "--ext_opt.aux.ext_conflicts.flag_opt",
                '--ext_opt.aux.ext_conflicts.more_conflicts', '--ext_opt.aux.ext_conflicts.more_conflicts'
            )
            cmd.assert_args(
                out,
                (cmd.single_arg, "argv"),
                (cmd.flag_opt, 1),
                (cmd.subc_sn_only, 2),
                (cmd.subc_opt_with_aliases, 4),

                (tas.subc_pl_aux_conflict, 4),
                (tas.flag_opt, 2),
                (tas.more_conflicts, 3),

                (ec.subc_pl_aux_conflict, 3),
                (ec.flag_opt, 3),
                (ec.more_conflicts, 2),
            )

    class TestWithCoreCmd(CLIShared):
        eval_config = CLIShared.exts.ext_conflicts["origen.eval"]
        eval_cmd = CLIShared.cmds.eval.extend(
        CLIShared.exts.exts["origen.eval"]["global_exts"] + eval_config["exts"],
            from_configs=[eval_config["cfg"]],
            with_env=eval_config["env"]
        )

        creds_clear_config = CLIShared.exts.ext_conflicts["origen.credentials.clear"]
        creds_clear_cmd = CLIShared.cmds.creds.clear.extend(
            creds_clear_config["exts"],
            from_configs=[creds_clear_config["cfg"]],
            with_env=creds_clear_config["env"]
        )

        @classmethod
        def setup_class(cls):
            cls.eval_cmd_help = cls.eval_cmd.get_help_msg()
            cls.eval_cmd_conflicts = cls.eval_cmd_help.logged_errors
            cls.creds_clear_cmd_help = cls.creds_clear_cmd.get_help_msg()
            cls.creds_clear_cmd_conflicts = cls.creds_clear_cmd_help.logged_errors

        @classmethod
        def teardown_class(cls):
            delattr(cls, "eval_cmd_help")
            delattr(cls, "eval_cmd_conflicts")
            delattr(cls, "creds_clear_cmd_help")
            delattr(cls, "creds_clear_cmd_conflicts")

        def test_ext_arg_conflicts_with_core_cmd_help_msg(self):
            cmd = self.eval_cmd
            help = self.eval_cmd_help
            aux_code_opt = CLIShared.exts.ext_conflicts["origen.eval"]["exts"][0]
            pl_code_opt = CLIShared.exts.ext_conflicts["origen.eval"]["exts"][1]

            help.assert_args(cmd.code)
            help.assert_opts(
                cmd.say_hi_after_eval,
                cmd.say_hi_before_eval,
                pl_code_opt,
                aux_code_opt,
                "help",
                cmd.scripts,
                cmd.say_hi_during_cleanup,
                "v",
                "vk",
            )
            help.assert_subcmds(None)

            assert help.aux_exts == ['ext_conflicts']
            assert help.pl_exts == ["python_plugin", 'test_apps_shared_test_helpers']
            assert help.app_exts == False

        def test_ext_arg_conflicts_with_core_cmd_msgs(self):
            # Use eval cmd for arg
            cmd = self.eval_cmd

            conflicts = self.eval_config["conflicts_list"]
            for c in reversed(conflicts):
                m = self.err_msgs.to_conflict_msg(cmd, c)
                assert m in self.eval_cmd_conflicts.pop()
            assert len(self.eval_cmd_conflicts) == 0


        def test_ext_opt_conflicts_with_core_cmd_help_msg(self):
            cmd = self.creds_clear_cmd
            help = self.creds_clear_cmd_help
            ec = self.creds_clear_config["exts_by_ns"].ec
            tas = self.creds_clear_config["exts_by_ns"].tas

            help.assert_args(None)
            help.assert_opts(
                cmd.all,
                ec.cmd_conflicts_aux,
                tas.cmd_conflicts_pl,
                cmd.datasets,
                ec.all,
                tas.all,
                "help",
                "v",
                "vk",
            )
            help.assert_subcmds(None)

            assert help.aux_exts == ['ext_conflicts']
            assert help.pl_exts == ['test_apps_shared_test_helpers']
            assert help.app_exts == False


        def test_ext_opt_conflicts_with_core_cmd(self):
            # Use credentials.clear for opts
            cmd = self.creds_clear_cmd

            conflicts = self.creds_clear_config["conflicts_list"]
            for c in reversed(conflicts):
                m = self.err_msgs.to_conflict_msg(cmd, c)
                assert m in self.creds_clear_cmd_conflicts.pop()
            assert len(self.creds_clear_cmd_conflicts) == 0
