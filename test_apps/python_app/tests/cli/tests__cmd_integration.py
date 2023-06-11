import pytest
from .shared import CLICommon, Cmd, CmdOpt, CmdArg

class T_CommandIntegration(CLICommon):
    class TestDisablingAppOpts(CLICommon):
        disabling_app_opts = CLICommon.app_cmds.disabling_app_opts
        disabling_app_opts_from_pl = CLICommon.plugins.python_plugin.disabling_app_opts_from_pl
        disabling_app_opts_from_aux = CLICommon.aux.ns.python_app_aux_cmds.disabling_app_opts_from_aux
        no_app_opts = ["h", "v", "vk"]
        no_target_opts = ["h", "m", "v", "vk"]
        no_mode_opts = ["h", 'nt', "t", "v", "vk"]

        def test_app_opts_are_added_by_default(self):
            help = self.disabling_app_opts.get_help_msg()
            help.assert_args(None)
            help.assert_bare_app_opts()

        cmds = [
            (disabling_app_opts.disable_targets_opt, no_target_opts),
            (disabling_app_opts.disable_mode_opt, no_mode_opts),
            (disabling_app_opts.disable_app_opts, no_app_opts),
            (disabling_app_opts_from_pl.disable_targets_opt, no_target_opts),
            (disabling_app_opts_from_pl.disable_mode_opt, no_mode_opts),
            (disabling_app_opts_from_pl.disable_app_opts, no_app_opts),
            (disabling_app_opts_from_aux.disable_targets_opt, no_target_opts),
            (disabling_app_opts_from_aux.disable_mode_opt, no_mode_opts),
            (disabling_app_opts_from_aux.disable_app_opts, no_app_opts),
        ]
        ids = [f"{cmd[0].parent.name}.{cmd[0].name}" for cmd in cmds]
        @pytest.mark.parametrize("cmd,opts", cmds, ids=ids)
        def test_disabling_app_opts(self, cmd, opts):
            help = cmd.get_help_msg()
            help.assert_opts(*opts)

        @pytest.mark.parametrize("cmd,opts", cmds, ids=ids)
        def test_app_opt_disables_are_inherited(self, cmd, opts):
            cmd = cmd.disable_subc
            help = cmd.get_help_msg()
            help.assert_opts(*opts)

        @pytest.mark.parametrize("cmd,opts", cmds, ids=ids)
        def test_inherited_app_opt_disables_can_be_overridden(self, cmd, opts):
            cmd = cmd.override_subc
            help = cmd.get_help_msg()
            help.assert_bare_app_opts()