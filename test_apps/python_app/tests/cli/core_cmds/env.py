from ..shared import CLICommon

class T_Env(CLICommon):
    _cmd = CLICommon.in_app_cmds.env

    def test_help_msg(self, cmd, cached_help):
        cached_help.assert_cmd(cmd)

    def test_help_msg_on_no_subcs(self, cmd, cached_help):
        out = cmd.gen_error()
        assert out == cached_help.text
