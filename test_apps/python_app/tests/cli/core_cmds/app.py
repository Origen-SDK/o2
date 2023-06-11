from test_apps_shared_test_helpers.cli.auditors import CmdNamespaceAuditor
from ..shared import CLICommon
import pytest

class T_App(CLICommon):
    class TestAppCommands(CmdNamespaceAuditor, CLICommon):
        nspace = CLICommon.app_cmds
        nspace_help_offset = 3
        empty_nspace = CLICommon.empty_app

    def test_app_cmd_help_msg(self):
        cmd = CLICommon.in_app_cmds.app
        help = cmd.get_help_msg()
        help.assert_cmd(cmd)
        out = cmd.gen_error()
        assert out == help.text