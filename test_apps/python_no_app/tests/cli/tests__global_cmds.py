import pytest
from origen.helpers.env import run_cli_cmd
from .shared import CLICommon
from tests.cmd_building.shared import CLICommon as CmdTestersCommon

class T_GlobalCmds(CLICommon):
    @pytest.mark.parametrize("cmd", CLICommon.global_cmds.cmds, ids=CLICommon.global_cmds.all_names)
    def test_core_commands_are_available(self, cmd):
        ''' Just testing that "-h" (or help <cmd> for some commands) doesn't crash for all core commands '''
        help = cmd.get_help_msg()
        assert len(help.opts) >= 3
        # FOR_PR
        # help.assert_bare_opts_present()