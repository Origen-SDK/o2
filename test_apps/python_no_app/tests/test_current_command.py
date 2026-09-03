import origen, pytest, pathlib
from types import SimpleNamespace
from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg

class TestCurrentCommand(CLIShared):
    @classmethod
    def parse_current_cmd(cls, out):
        def quote(s):
            return f"r'{s}'"
        out = out.split("Start Action For CMD: display_current_command\n")[1].split("End Action For CMD: display_current_command")[0].split("\n")[:-1]
        print(out)
        assert out[0] == "Class: CurrentCommand"
        assert out[1] == "Command Src Class: CommandSource"
        assert out[2] == "Src Type Class: SourceType"
        assert out[3] == "Src Plugin Class: Application"
        offset = 3
        return SimpleNamespace(**{
            "base_cmd": out[offset + 1].split("Base Cmd: ")[1],
            "subcmds": eval(out[offset + 2].split("Sub Cmds: ")[1]),
            "args": eval(out[offset + 3].split("Args: ")[1]),
            "arg_indices": eval(out[offset + 4].split("Arg Indices: ")[1]),
            "exts": eval(out[offset + 5].split("Exts: ")[1]),
            "src_path": eval(quote(out[offset + 6].split("Src Path: ")[1])),
            "src_plugin": eval(quote(out[offset + 7].split("Src Plugin: ")[1])),
            "src_type": eval(quote(out[offset + 8].split("Src Type: ")[1])),
            "is_core_cmd": eval(out[offset + 9].split("Src is_core_cmd: ")[1]),
            "is_plugin_cmd": eval(out[offset + 10].split("Src is_plugin_cmd: ")[1]),
            "is_aux_cmd": eval(out[offset + 11].split("Src is_aux_cmd: ")[1]),
            "is_app_cmd": eval(out[offset + 12].split("Src is_app_cmd: ")[1]),
            "src_root_name": eval(quote(out[offset + 13].split("Src root name: ")[1])),
        })

    # TEST_NEEDED: CurrentCommand need a better way to handle and test other sources
    @classmethod
    def assert_current_cmd(cls, out, base, subcmds, args, arg_indices, exts, path, src, src_name):
        cmd = cls.parse_current_cmd(out)
        assert cmd.base_cmd == base
        assert cmd.subcmds == subcmds
        assert cmd.args == args
        assert cmd.arg_indices == arg_indices
        assert cmd.exts == exts
        assert pathlib.Path(cmd.src_path) == path
        if src == "plugin":
            assert cmd.src_plugin == src_name
            assert cmd.src_type == "plugin"
            assert not cmd.is_core_cmd
            assert cmd.is_plugin_cmd
            assert not cmd.is_aux_cmd
            assert not cmd.is_app_cmd
            assert cmd.src_root_name == "plugin"
        else:
            RuntimeError("Currently only plugin sources are supported")

    def test_current_command_is_none(self):
        assert origen.current_command is None

    @pytest.mark.skip
    def test_current_command_from_core_cmd(self):
        # TEST_NEEDED Current Command core case
        eval_cmd

    def test_current_command_from_pl_cmd(self):
        out = self.python_plugin.do_actions.run("display_current_command")
        f = origen.plugins["python_plugin"].root.joinpath("python_plugin/commands/do_actions.py")
        self.assert_current_cmd(
            out,
            "_plugin_dispatch_",
            ["do_actions"],
            {"actions": ['display_current_command']},
            {'actions': [1]},
            {},
            f,
            "plugin",
            "python_plugin"
        )

    def test_arg_indices(self):
        cmd = CLIShared.python_plugin.plugin_test_args.extend(
            CLIShared.exts.exts["plugin.python_plugin.plugin_test_args"]["exts"],
            from_configs=CLIShared.exts.exts_workout_cfg
        )

        ext_flag = cmd.flag_extension
        ext_ha = cmd.hidden_opt
        ext_action = cmd.exts_workout_action

        args = "show_arg_indices"
        exts = "show_ext_arg_indices"
        # Index 0 is the command name
        # NOTE: per the clap API, when flags (options not accepting values) are used, only the last index is given
        out = cmd.run(
            "sv", "m0", "m1", "m2", # indices 1, 2-4
            ext_flag.ln_to_cli(), # 5
            cmd.opt_taking_value.ln_to_cli(), "opt_val", # 6 (opt name), 7 (value)
            ext_flag.sn_to_cli(), # 8
            ext_ha.ln_to_cli(), # 9
            ext_action.ln_to_cli(), args, exts, # 10 (opt name), 11, 12 (values)
            cmd.multi_val_delim_opt.ln_to_cli(), "d0,d1,d2"
        )
        parsed = self.get_action_results(out, [args, exts])
        assert eval(parsed[args]["Before"]) == {
            cmd.single_arg.name: [1],
            cmd.multi_arg.name: [2, 3, 4],
            cmd.opt_taking_value.name: [7]
        }
        assert eval(parsed[exts]["Before"]) == {
            "aux.exts_workout": {
                ext_flag.name: [8],
                ext_ha.name: [9],
                ext_action.name: [11, 12],
                cmd.multi_val_delim_opt.name: [14, 15, 16],
            }
        }

    @pytest.mark.skip
    def test_current_command_from_aux_cmd(self):
        # TEST_NEEDED Current Command core case
        aux_cmd

    @pytest.mark.skip
    def test_current_command_from_app_cmd(self):
        # TEST_NEEDED Current Command app case
        # Obviously move to app
        app_cmd