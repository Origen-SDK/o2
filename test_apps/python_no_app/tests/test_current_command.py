import origen, pytest
from types import SimpleNamespace
from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg

class TestCurrentCommand(CLIShared):
    @classmethod
    def parse_current_cmd(cls, out):
        out = out.split("Start Action For CMD: display_current_command\n")[1].split("End Action For CMD: display_current_command")[0].split("\n")[:-1]
        print(out)
        assert out[0] == "Class: CurrentCommand"
        return SimpleNamespace(**{
            "base_cmd": out[1].split("Base Cmd: ")[1],
            "subcmds": eval(out[2].split("Sub Cmds: ")[1]),
            "args": eval(out[3].split("Args: ")[1]),
            "exts": eval(out[4].split("Exts: ")[1]),
        })

    @classmethod
    def assert_current_cmd(cls, out, base, subcmds, args, exts):
        cmd = cls.parse_current_cmd(out)
        assert cmd.base_cmd == base
        assert cmd.subcmds == subcmds
        assert cmd.args == args
        assert cmd.exts == exts

    def test_current_command_is_none(self):
        assert origen.current_command is None

    @pytest.mark.skip
    def test_current_command_from_core_cmd(self):
        # TEST_NEEDED Current Command core case
        eval_cmd

    def test_current_command_from_pl_cmd(self):
        out = self.python_plugin.do_actions.run("display_current_command")
        self.assert_current_cmd(
            out,
            "_plugin_dispatch_",
            ["do_actions"],
            {"actions": ['display_current_command']},
            {}
        )

    @pytest.mark.skip
    def test_current_command_from_aux_cmd(self):
        # TEST_NEEDED Current Command core case
        aux_cmd

    @pytest.mark.skip
    def test_current_command_from_app_cmd(self):
        # TEST_NEEDED Current Command app case
        # Obviously move to app
        app_cmd