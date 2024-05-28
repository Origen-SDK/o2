from test_apps_shared_test_helpers.cli import CLIShared

class T_IntraCmdConflicts(CLIShared):
    cmd = CLIShared.python_plugin.intra_cmd_conflicts
    conflicts = CLIShared.python_plugin.intra_cmd_conflicts_list

    @classmethod
    def setup_class(cls):
        cls.cmd_help = cls.cmd.get_help_msg()
        cls.cmd_conflicts = cls.cmd_help.logged_errors

    @classmethod
    def teardown_class(cls):
        delattr(cls, "cmd_help")
        delattr(cls, "cmd_conflicts")

    def test_help_msg(self):
        # Check help message
        cmd = self.cmd
        help = self.cmd_help
        help.assert_args(cmd.arg0, cmd.arg1, cmd.arg2)
        help.assert_opts(
            cmd.arg_clash,
            cmd.intra_opt_conflicts,
            "help",
            cmd.inter_opt_conflicts,
            cmd.opt,
            cmd.reserved_prefix_in_ln_lna,
            "v",
            "vk",
        )
        help.assert_subcmds(cmd.conflicts_subc, "help")

        assert help.aux_exts == None
        assert help.pl_exts == None
        assert help.app_exts == False

    def test_conflicts_during_cmd_building(self):
        for c in reversed(self.conflicts):
            m = self.cmd_conflicts.pop()
            assert self.err_msgs.to_conflict_msg(self.cmd, c) in m

    def test_all_error_messages_checked(self):
        assert len(self.cmd_conflicts) == 0

    def test_cmd_conflicts_resolve_correctly(self):
        cmd = self.cmd
        out = cmd.run(
            "0", "1", "2",
            "--opt", "--opt0",
            "--arg0", "--arg1",
            "--ext_opt_lna",
            "--intra_opt_cons", "-c", "-e",
            "-d",
        )
        cmd.assert_args(
            out,
            (cmd.arg0, "0"),
            (cmd.arg1, "1"),
            (cmd.arg2, "2"),
            (cmd.opt, 2),
            (cmd.arg_clash, 2),
            (cmd.reserved_prefix_in_ln_lna, 1),
            (cmd.intra_opt_conflicts, 3),
            (cmd.inter_opt_conflicts, 1),
        )

    def test_subc_help_msg(self):
        # Check help message
        cmd = self.cmd.conflicts_subc
        help = cmd.get_help_msg()

        help.assert_args(cmd.arg0, cmd.sub_arg_1)
        help.assert_opts(
            "help",
            cmd.inter_subc_conflicts,
            cmd.intra_subc_lna_iln_conflict,
            cmd.opt,
            cmd.intra_subc_conflicts,
            "v",
            "vk",
        )
        help.assert_subcmds(None)

        assert help.aux_exts == None
        assert help.pl_exts == None
        assert help.app_exts == False

    def test_cmd_subc_conflicts_resolve_correctly(self):
        cmd = self.cmd.conflicts_subc
        out = cmd.run(
            "a", "b",
            "--opt", "--subc_opt",
            "--intra_subc_conflicts", "-r",
            "--intra_subc_lna_iln_conflict",
            "--inter_subc_conflicts"
        )
        cmd.assert_args(
            out,
            (cmd.arg0, "a"),
            (cmd.sub_arg_1, "b"),
            (cmd.opt, 2),
            (cmd.intra_subc_conflicts, 2),
            (cmd.intra_subc_lna_iln_conflict, 1),
            (cmd.inter_subc_conflicts, 1),
        )
