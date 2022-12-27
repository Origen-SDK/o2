import pytest
from origen.helpers.env import run_cli_cmd
from .shared import CLICommon
from tests.cmd_building.shared import CLICommon as CmdTestersCommon

class T_GlobalCmds(CLICommon):
    def test_origen_v(self):
        out = run_cli_cmd(["-v"])
        assert "Origen: " in out
        assert "CLI:    " in out
        assert "app" not in out.lower()

    def test_global_help_message_core_commands(self):
        out = run_cli_cmd(["-h"])
        help = CmdTestersCommon.HelpMsg(out)
        assert help.root_cmd is True
        assert "Origen: 2." in help.version_str

        assert len(help.opts) == 3
        help.assert_help_opt_at(0)
        help.assert_vk_opt_at(1)
        help.assert_v_opt_at(2)

        # TODO check order?
        assert set(s["name"] for s in help.subcmds) == set(self.global_cmds.all_names_add_help)
        assert help.app_cmd_shortcuts == None
        assert help.pl_cmd_shortcuts == {
            "plugin_says_hi": ("python_plugin", "plugin_says_hi"),
            "echo": ("python_plugin", "echo"),
            "plugin_test_args": ("python_plugin", "plugin_test_args"),
            "plugin_test_ext_stacking": ("python_plugin", "plugin_test_ext_stacking"),
        }
        assert help.aux_cmd_shortcuts == {
            "python_no_app_tests": ("cmd_testers", "python_no_app_tests"),
            "test_current_command": ("cmd_testers", "test_current_command"),
            "test_nested_level_1": ("cmd_testers", "test_nested_level_1"),
            "test_arguments": ("cmd_testers", "test_arguments"),
            "error_cases": ("cmd_testers", "error_cases"),
            "say_hi": ("python_no_app_aux_cmds", "say_hi"),
            "say_bye": ("python_no_app_aux_cmds", "say_bye"),
        }

    @pytest.mark.parametrize("cmd", CLICommon.global_cmds.cmds, ids=CLICommon.global_cmds.all_names)
    def test_core_commands_are_available(self, cmd):
        ''' Just testing that "-h" doesn't crash for all core commands '''
        help = cmd.get_help_msg()
        assert len(help.opts) >= 3