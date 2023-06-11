import pytest, origen
from .shared import CLICommon, Cmd, CmdOpt, CmdArg

class T_AppCmdBuilding(CLICommon):
    warmup_cmd = CLICommon.app_cmds.arg_opt_warmup
    nested_cmds = CLICommon.app_cmds.nested_app_cmds

    def test_app_command_args_and_opts(self):
        cmd = self.warmup_cmd
        help = cmd.get_help_msg()
        help.assert_args(cmd.first, cmd.second)
        help.assert_opts(
            cmd.flag_opt,
            'h', 'm',
            cmd.multi_opt,
            'nt',
            cmd.single_opt,
            't', 'v', 'vk'
        )
        help.assert_subcmds(None)

        rv = "req_arg"
        m0 = "m0"
        m12 = "m1,m2"
        mo0 = "mo0"
        mo12 = "m01,m02"
        mo4 = "mo4"
        sv = "s_opt"

        out = cmd.run(
            rv,
            m0, m12,
            cmd.flag_opt.sn_to_cli(),
            cmd.single_opt.sna_to_cli(), sv,
            cmd.multi_opt.sna_to_cli(), mo0, mo12,
            cmd.multi_opt.ln_to_cli(), mo4,
            cmd.hidden_flag_opt.ln_to_cli(),
        )
        cmd.assert_args(
            out,
            (cmd.first, rv),
            (cmd.second, [m0, m12]),
            (cmd.flag_opt, 1),
            (cmd.single_opt, sv),
            (cmd.multi_opt, [mo0, mo12, mo4]),
            (cmd.hidden_flag_opt, 1)
        )

        out = cmd.run(
            rv,
            cmd.flag_opt.sn_to_cli(), cmd.flag_opt.sn_to_cli(),
            cmd.multi_opt.sna_to_cli(), mo0, mo12,
        )
        cmd.assert_args(
            out,
            (cmd.first, rv),
            (cmd.flag_opt, 2),
            (cmd.multi_opt, [mo0, mo12])
        )

        out = cmd.gen_error()
        assert self.err_msgs.missing_required_arg(cmd.first) in out

    nested_cmd_testcases = [
        (nested_cmds, 0, None),
        (nested_cmds.nested_l1, 1, None),
        (nested_cmds.nested_l1.nested_l2_a, 2, 'A'),
        (nested_cmds.nested_l1.nested_l2_a.nested_l3_a, 3, 'A-A'),
        (nested_cmds.nested_l1.nested_l2_a.nested_l3_b, 3, 'A-B'),
        (nested_cmds.nested_l1.nested_l2_b, 2, 'B'),
        (nested_cmds.nested_l1.nested_l2_b.nested_l3_a, 3, 'B-A'),
        (nested_cmds.nested_l1.nested_l2_b.nested_l3_b, 3, 'B-B'),
    ]
    nested_cmd_ids = [f"{cmd[0].name} {'Base' if cmd[2] is None else cmd[2]}" for cmd in nested_cmd_testcases]
    @pytest.mark.parametrize("cmd,lvl,sublvl", nested_cmd_testcases, ids=nested_cmd_ids)
    def test_nested_cmds(self, cmd, lvl, sublvl):
        help = cmd.get_help_msg()
        subcs = list(cmd.subcmds.values())
        if len(subcs) == 0:
            help.assert_subcmds(None)
        else:
            subcs.insert(0, "help")
            help.assert_subcmds(*subcs)

        out = cmd.run()
        assert f"Hi from 'nested_app_cmds' level {lvl}{ ' (' + sublvl + ')' if sublvl else ''}!" in out

    def test_enumerated_tomls(self):
        enum_envs = {"with_env": {"ORIGEN_APP_COMMANDS": ",".join([
            str(self.cmd_tomls.simple_toml.relative_to(origen.app.config_dir)),
            str(self.cmd_tomls.simple2_toml),
        ])}}

        help = self.in_app_cmds.origen.get_help_msg(run_opts=enum_envs)
        assert help.app_cmd_shortcuts == self.cmd_shortcuts.simple_cmd_tomls

        help = self.in_app_cmds.app.commands.get_help_msg(run_opts=enum_envs)
        help.assert_subcmds(
            "help",
            self.cmd_tomls.simple,
            self.cmd_tomls.simple2,
            self.cmd_tomls.simple2_with_arg,
            self.cmd_tomls.simple_with_arg,
        )

        cmd = self.cmd_tomls.simple_with_arg
        out = cmd.run("hi", run_opts=enum_envs)
        cmd.assert_args(
            out,
            (cmd.arg, "hi"),
        )

        cmd = self.cmd_tomls.simple2_with_arg
        out = cmd.run("hi", run_opts=enum_envs)
        cmd.assert_args(
            out,
            (cmd.arg, "hi"),
        )

    class TestErrorCases(CLICommon):
        def test_invalid_cmd_toml(self):
            missing = "missing_cmd_toml.toml"
            env = {"with_env": {"ORIGEN_APP_COMMANDS": ",".join([
                str(self.cmd_tomls.invalid_toml.relative_to(origen.app.config_dir)),
                str(self.cmd_tomls.simple_toml.relative_to(origen.app.config_dir)),
                str(self.cmd_tomls.invalid2_toml),
                str(f"cmd_tomls/{missing}"), # Since not absolute, should compute relative to config/ directory, but not from cmd_tomls
                str(self.cmd_tomls.simple2_toml),
            ])}}
            help = self.in_app_cmds.origen.get_help_msg(run_opts=env)
            assert help.app_cmd_shortcuts == self.cmd_shortcuts.simple_cmd_tomls
            assert help.pl_cmd_shortcuts == self.cmd_shortcuts.pl
            assert help.aux_cmd_shortcuts == self.cmd_shortcuts.aux
            errs = help.logged_errors
            assert f"Malformed Commands TOML '{self.cmd_tomls.invalid2_toml}'" in errs.pop()
            assert f"Malformed Commands TOML '{self.cmd_tomls.invalid_toml}'" in errs.pop()
            assert f"Can not locate app commands file '{self.cmd_tomls.root.joinpath(missing)}'" in errs.pop()
            assert len(errs) == 0

        def test_missing_app_cmd_implementation(self):
            env = {"with_env": {"ORIGEN_APP_COMMANDS": ",".join([
                str(self.cmd_tomls.simple_toml.relative_to(origen.app.config_dir)),
            ])}}
            out = self.cmd_tomls.simple.gen_error(run_opts=env, return_full=True)
            errs = self.extract_logged_errors(out["stdout"])
            assert "  simple.py" == errs.pop()
            assert f"  From root '{origen.app.commands_dir}', searched:" == errs.pop()
            assert "Could not find implementation for app command 'simple'" == errs.pop()
            assert len(errs) == 0
