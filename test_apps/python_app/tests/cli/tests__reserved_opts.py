import pytest, origen
from .shared import CLICommon, Cmd, CmdOpt, CmdExtOpt
from origen.helpers.regressions.cli.command import SrcTypes

class T_ReservedOpts(CLICommon):
    cmd = CLICommon.app_sub_cmd(
        "reserved_opt_error_gen",
        help = "Generate error messages when reserved opts are used",
        opts=[
            CmdOpt("conflicting_help", help="Conflicting Opt"),
            CmdOpt("conflicting_target", help="Conflicting Opt"),
            CmdOpt("non_conflicting", help="Non-Conflicting Opt", ln="non_conflicting", sn="n"),
        ],
        subcmds=[
            Cmd(
                "single_conflicting_opt",
                help="Generate error messages for reserved opts",
                opts=[
                    CmdOpt("non_conflicting", help="Non-Conflicting Opt"),
                    CmdOpt("conflicting_target", help="Conflicting Opt"),
                ],
            ),
            Cmd(
                "multiple_conflicting_opts",
                help="Generate error messages for reserved opts",
                opts=[
                    CmdOpt("conflicting_target", help="Conflicting Opt"),
                    CmdOpt("conflicting_mode", help="Conflicting Opt"),
                    CmdOpt("not_conflicting", help="Non-Conflicting Opt", sn='n'),
                    CmdOpt("conflicting_no_targets", help="Conflicting Opt"),
                    CmdOpt("conflicting_target_again", help="Conflicting Opt"),
                    CmdOpt("conflicting_v", help="Conflicting Opt"),
                    CmdOpt("conflicting_vk", help="Conflicting Opt"),
                    CmdOpt("conflicting_help", help="Conflicting Opt", ln_aliases=["help_alias"], sn_aliases=["g", "i"]),
                ],
                subcmds=[
                    Cmd(
                        "subc",
                        help="Generate error messages for reserved opts - subc",
                        opts=[
                            CmdOpt("conflicting_help", help="Conflicting Opt"),
                        ],
                        subcmds=[
                            Cmd(
                                "subc",
                                help="Generate error messages for reserved opts - subc - subc",
                                opts=[
                                    CmdOpt("conflicting_help", help="Conflicting Opt", ln_aliases=["help2", "help3"]),
                                    CmdOpt("conflicting_v", help="Conflicting Opt", sn="c"),
                                ],
                            ),
                        ]
                    )
                ]
            )
        ],
        with_env={"ORIGEN_APP_TEST_RESERVED_OPT_ERRORS": "1"},
    )

    @classmethod
    def setup_class(cls):
        if not hasattr(cls, "base_cmd_help"):
            cls.base_cmd_help = cls.cmd.get_help_msg()
        if not hasattr(cls, "ext_cmd_help"):
            cls.ext_cmd_help = cls.ext_cmd.get_help_msg()

    @classmethod
    def teardown_class(cls):
        delattr(cls, "base_cmd_help")
        delattr(cls, "ext_cmd_help")

    def test_opts_are_added_with_respect_to_errors(self):
        help = self.base_cmd_help
        help.assert_args(None)
        help.assert_opts(
            self.cmd.conflicting_help,
            self.cmd.conflicting_target,
            "h", "m",
            self.cmd.non_conflicting,
            "nt", "t", "v", "vk"
        )
        help.assert_subcmds("help", self.cmd.multiple_conflicting_opts, self.cmd.single_conflicting_opt)

        cmd = self.cmd.single_conflicting_opt
        help = cmd.get_help_msg()
        help.assert_opts(
            cmd.conflicting_target,
            "h", "m", "nt",
            cmd.non_conflicting,
            "t", "v", "vk"
        )

        cmd = self.cmd.multiple_conflicting_opts
        help = cmd.get_help_msg()
        help.assert_opts(
            cmd.conflicting_help,
            cmd.conflicting_mode,
            cmd.conflicting_no_targets,
            cmd.conflicting_target,
            cmd.conflicting_target_again,
            cmd.conflicting_v,
            cmd.conflicting_vk,
            "h", "m",
            cmd.not_conflicting,
            "nt", "t", "v", "vk"
        )

        cmd = cmd.subc
        help = cmd.get_help_msg()
        help.assert_opts(
            cmd.conflicting_help,
            "h", "m", "nt", "t", "v", "vk"
        )

        cmd = cmd.subc
        help = cmd.get_help_msg()
        help.assert_opts(
            cmd.conflicting_v,
            cmd.conflicting_help,
            "h", "m", "nt", "t", "v", "vk"
        )

    errors = [
        (cmd, cmd.conflicting_help, [("sn", "h")]),
        (cmd, cmd.conflicting_target, [("sn", "t"), ("ln", "target")]),
        (cmd.multiple_conflicting_opts, cmd.multiple_conflicting_opts.conflicting_help, [("ln", "help"), ("sn", "h")]),
        (cmd.multiple_conflicting_opts, cmd.multiple_conflicting_opts.conflicting_vk, [("lna", "verbosity_keywords"), ("ln", "vk")]),
        (cmd.multiple_conflicting_opts, cmd.multiple_conflicting_opts.conflicting_v, [("ln", "verbosity"), ("sn", "v")]),
        (cmd.multiple_conflicting_opts, cmd.multiple_conflicting_opts.conflicting_no_targets, [("lna", "no_targets"), ("lna", "no_target")]),
        (cmd.multiple_conflicting_opts, cmd.multiple_conflicting_opts.conflicting_target_again, [("lna", "targets"), ("ln", "target"), ("sn", "t")]),
        (cmd.multiple_conflicting_opts, cmd.multiple_conflicting_opts.conflicting_mode, [("lna", "mode")]),
        (cmd.multiple_conflicting_opts, cmd.multiple_conflicting_opts.conflicting_target, [("sn", "t")]),
        (cmd.multiple_conflicting_opts.subc, cmd.multiple_conflicting_opts.subc.conflicting_help, [("sn", "h")]),
        (cmd.multiple_conflicting_opts.subc.subc, cmd.multiple_conflicting_opts.subc.subc.conflicting_v, [("sna", "v"), ("ln", "verbosity")]),
        (cmd.multiple_conflicting_opts.subc.subc, cmd.multiple_conflicting_opts.subc.subc.conflicting_help, [("lna", "help")]),
        (cmd.single_conflicting_opt, cmd.single_conflicting_opt.conflicting_target, [("ln", "target")]),
    ]
    @pytest.mark.parametrize(
        "cmd,opt,type,name",
        [(o[0], o[1], inner[0], inner[1]) for o in errors for inner in o[2]],
        ids=[f"{o[0].name}-{o[1].name}-{inner[0]}-{inner[1]}" for o in errors for inner in o[2]],
    )
    def test_error_msg_using_reserved_opts(self, cmd, opt, type, name):
        if type == "sn":
            assert cmd.reserved_opt_sn_conflict_msg(opt, name) in self.base_cmd_help.text
        elif type == "sna":
            assert cmd.reserved_opt_sna_conflict_msg(opt, name) in self.base_cmd_help.text
        elif type == "ln":
            assert cmd.reserved_opt_ln_conflict_msg(opt, name) in self.base_cmd_help.text
        elif type == "lna":
            assert cmd.reserved_opt_lna_conflict_msg(opt, name) in self.base_cmd_help.text
        else:
            raise RuntimeError(f"Unknown type {type}")

    def test_opts_are_still_available_under_non_reserved_names(self):
        cmd = self.cmd
        out = cmd.run("--conflicting_help", "--conflicting_target", "--non_conflicting", "-n")
        cmd.assert_args(
            out,
            (cmd.conflicting_help, 1),
            (cmd.conflicting_target, 1),
            (cmd.non_conflicting, 2)
        )

        cmd = self.cmd.single_conflicting_opt
        out = cmd.run("--non_conflicting", "--conflicting_target")
        cmd.assert_args(
            out,
            (cmd.conflicting_target, 1),
            (cmd.non_conflicting, 1)
        )

        cmd = self.cmd.multiple_conflicting_opts
        out = cmd.run(
            "--conflicting_target",
            "--conflicting_mode",
            "-n",
            "--conflicting_no_targets",
            "--conflicting_target_again",
            "--conflicting_v",
            "--conflicting_vk",
            "--conflicting_help", "-g", "-i", "--help_alias",
        )
        cmd.assert_args(
            out,
            (cmd.conflicting_target, 1),
            (cmd.conflicting_mode, 1),
            (cmd.not_conflicting, 1),
            (cmd.conflicting_no_targets, 1),
            (cmd.conflicting_target_again, 1),
            (cmd.conflicting_v, 1),
            (cmd.conflicting_vk, 1),
            (cmd.conflicting_help, 4),
        )

        cmd = cmd.subc
        out = cmd.run("--conflicting_help")
        cmd.assert_args(out, (cmd.conflicting_help, 1))

        cmd = cmd.subc
        out = cmd.run(
            "--conflicting_help", "--help2", "--help3",
            "-c",
        )
        cmd.assert_args(
            out,
            (cmd.conflicting_help, 3),
            (cmd.conflicting_v, 1)
        )

    ext_error_msgs_env = {"ORIGEN_APP_EXT_TEST_RESERVED_OPT_ERRORS": "1", "origen_bypass_config_lookup": "1"}
    ext_cmd = origen.helpers.regressions.cli.CLI.in_app_cmds.eval.extend(
        CmdExtOpt.from_src(
            "example",
            SrcTypes.APP,
            CmdExtOpt(
                "conflicting_target",
                help="Conflicting Core Extension"
            ),
            CmdExtOpt(
                "conflicting_no_target",
                help="Conflicting Core Extension",
                sn="n",
            ),
            CmdExtOpt(
                "conflicting_mode",
                help="Conflicting Core Extension",
                ln="mode_conflict",
                sn_aliases=["m"]
            ),
            CmdExtOpt(
                "conflicting_help",
                help="Conflicting Core Extension",
                ln="help_conflict",
                ln_aliases=["help1"]
            ),
            CmdExtOpt(
                "conflicting_v",
                help="Conflicting Core Extension",
                sn_aliases=["w"]
            ),
            CmdExtOpt(
                "conflicting_vk",
                help="Conflicting Core Extension"
            ),
        ),
        with_env=ext_error_msgs_env,
        from_configs=CLICommon.configs.suppress_plugin_collecting_config,
    )

    def test_ext_opts_are_added_with_respect_to_errors(self):
        cmd = self.ext_cmd
        help = self.ext_cmd_help
        help.assert_args(cmd.code)
        help.assert_opts(
            cmd.conflicting_target,
            cmd.conflicting_v,
            cmd.conflicting_vk,
            "h",
            cmd.conflicting_help,
            "m",
            cmd.conflicting_mode,
            cmd.conflicting_no_target,
            "nt", "t", "v", "vk"
        )
        help.assert_subcmds(None)

    ext_errors = [
        (ext_cmd, ext_cmd.conflicting_target, [("ln", "target"), ("sn", "t")]),
        (ext_cmd, ext_cmd.conflicting_no_target, [("lna", "no_target"), ("lna", "no_targets")]),
        (ext_cmd, ext_cmd.conflicting_help, [("sn", "h"), ("lna", "help")]),
        (ext_cmd, ext_cmd.conflicting_mode, [("lna", "mode")]),
        (ext_cmd, ext_cmd.conflicting_v, [("ln", "verbosity"), ("sn", "v")]),
        (ext_cmd, ext_cmd.conflicting_vk, [("lna", "verbosity_keywords"), ("lna", "vk")]),
    ]
    @pytest.mark.parametrize(
        "cmd,opt,type,name",
        [(o[0], o[1], inner[0], inner[1]) for o in ext_errors for inner in o[2]],
        ids=[f"{o[0].name}-{o[1].name}-{inner[0]}-{inner[1]}" for o in ext_errors for inner in o[2]],
    )
    def test_ext_error_msg_using_reserved_opts(self, cmd, opt, type, name):
        if type == "sn":
            assert cmd.reserved_opt_sn_conflict_msg(opt, name) in self.ext_cmd_help.text
        elif type == "sna":
            assert cmd.reserved_opt_sna_conflict_msg(opt, name) in self.ext_cmd_help.text
        elif type == "ln":
            assert cmd.reserved_opt_ln_conflict_msg(opt, name) in self.ext_cmd_help.text
        elif type == "lna":
            assert cmd.reserved_opt_lna_conflict_msg(opt, name) in self.ext_cmd_help.text
        else:
            raise RuntimeError(f"Unknown type {type}")

    def test_ext_opts_are_still_available_under_non_reserved_names(self):
        cmd = self.ext_cmd
        out = cmd.run(
            "from test_apps_shared_test_helpers.aux_cmds import run; run()",
            "--conflicting_target",
            "-n",
            "--mode_conflict", "-m",
            "--help_conflict", "--help1",
            "--conflicting_v", "-w",
            "--conflicting_vk",
        )
        self.Cmd.assert_args(
            cmd,
            out,
            (cmd.conflicting_target, 1),
            (cmd.conflicting_no_target, 1),
            (cmd.conflicting_mode, 2),
            (cmd.conflicting_help, 2),
            (cmd.conflicting_v, 2),
            (cmd.conflicting_vk, 1),
        )
