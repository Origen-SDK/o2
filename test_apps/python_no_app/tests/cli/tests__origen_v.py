from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg
import pytest

class T_OrigenVersion(CLIShared):
    def test_origen_v(self):
        out = self.cmds.v.run()
        self.assert_v(out, None)
        self.assert_no_app_origen_v(out)

    invocs = [
        ("vvv", ["-vvv"], 2, None),
        ("v_v_v", ["-v", "-v", "-v"], 2, None),
        ("vks", ["-v", "--verbose", "--vk", "vk0", "-vv"], 3, ["vk0"]),
        ("vks_2", ["--verbose", "--vk", "vka", "-vv", "--vk", "vkb"], 2, ["vka", "vkb"]),
    ]
    @pytest.mark.parametrize("args,vlvl,vks", [[h[1], h[2], h[3]] for h in invocs], ids=[h[0] for h in invocs])
    def test_origen_v_with_verbosity(self, args, vlvl, vks):
        out = self.run_cli_cmd(args)
        self.assert_v(out, vlvl, vks)
        self.assert_no_app_origen_v(out, version_only=False)
