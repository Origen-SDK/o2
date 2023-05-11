import pytest
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

    class TestErrorCases(CLICommon):
        @pytest.mark.skip
        def test_invalid_cmd_toml(self):
            # FOR_PR need to make app specific
            out = self.in_app_cmds.origen.run(with_env={"ORIGEN_APP_COMMANDS": "test_case_cmds/invalid.toml,test_case_cmds/error_cases.toml"}) # run_cli_cmd(["-h"]) #.split("\n\n")
            print(out)
            help = self.HelpMsg(out)
            assert list(help.subcmds.keys()) == ["add_aux_cmd", "cmd_testers", "help", "python_no_app_aux_cmds"]
            assert f"Unable to add auxillary commands at '{self.aux_cmd_configs_dir}{os.sep}./invalid_aux_cmd_path.toml' from config '{self.aux_cmd_configs_dir}{os.sep}invalid_aux_cmd_path_config.toml'. The following error was met" in out

        @pytest.mark.skip
        def test_error_global_and_in_app_setting_used(self):
            fail
