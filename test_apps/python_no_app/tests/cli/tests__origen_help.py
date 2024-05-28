from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg
import pytest

class T_OrigenHelp(CLIShared):
    help_invocations = [
        ("sn", ["-h"]),
        ("ln", ["--help"]),
        ("subc", ["help"]),
        ("sn_and_ln", ["-h", "--help"],
        ("help_on_no_subc", [])),
    ]
    @pytest.mark.parametrize("args", [h[1] for h in help_invocations], ids=[h[0] for h in help_invocations])
    def test_origen_help(self, args):
        out = self.run_cli_cmd(args)
        self.assert_v(out, None)
        self.assert_core_help(out)

    verbose_help_invocs = [
        ("sns", (["-h", "-v", "-v", "-v"], 3, None)),
        ("ln_with_vks", (["-vv", "--help", "--vk", "vk0,vk1"], 2, ["vk0", "vk1"])),
        ("subc_with_vks", (["-vv", "--vk", "vka", "--verbose", "--vk", "vkb", "help"], 3, ["vka", "vkb"]),
        ("no_subc", (["--verbose", "--vk", "vk", "--verbose"], 2, ["vk"]))),

        # These won't show since verbosity is not enabled, but should get the help message without other errors
        ("vks_only", (["--vk", "vk0", "--vk", "vk1"], None, None)),
    ]
    @pytest.mark.parametrize("args,vlvl,vks", [h[1] for h in verbose_help_invocs], ids=[h[0] for h in verbose_help_invocs])
    def test_help_with_verbosity(self, args, vlvl, vks):
        out = self.run_cli_cmd(args)
        self.assert_v(out, vlvl, vks)
        self.assert_core_help(out)
