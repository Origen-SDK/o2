# FOR_PR clean up
import re
import shutil
import origen, pytest, origen_metal, getpass, os
from pathlib import Path
# import origen_metal
from origen.helpers.env import in_new_origen_proc, run_cli_cmd
from tests import configs as config_funcs
from test_apps_shared_test_helpers.cli import CmdExtOpt

from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg

class TestExtensions(CLIShared):
    @pytest.mark.skip
    def test_extending_pl_cmd_from_aux_cmds(self):
        fail

    @pytest.mark.skip
    def test_extending_aux_cmd_from_pl(self):
        fail

    @pytest.mark.skip
    class ErrorConditions():
        def test_exception_in_before_cmd(self):
            actions = ["before_cmd_exception"]
            out = self.cmd.gen_error(
                self.sv,
                self.ext_action.ln_to_cli(), *actions,
                return_full=True,
            )
            stderr = out["stderr"]
            stdout = out["stdout"]
            action_strs = self.ext_action.to_assert_str(actions)
            sv_str = self.cmd.single_arg.to_assert_str(self.sv)
            print(stdout)
            assert "RuntimeError: 'before_cmd_exception' encountered!" in stderr
            assert action_strs[0] in stdout
            assert action_strs[1] not in stdout
            assert action_strs[2] in stdout
            assert sv_str not in stdout
            fail

        def test_exception_in_after_cmd(self):
            fail

        def test_exception_in_cmd(self):
            actions = [self.na]
            out = self.cmd.gen_error(
                "gen_error",
                self.ext_action.ln_to_cli(), *actions,
                return_full=True,
            )
            fail

        def test_exception_in_clean_up(self):
            fail
        
        def test_exceptions_in_multiple_clean_ups(self):
            fail
        
        def test_exceptions_in_before_and_cleanups(self):
            fail
        
        def test_missing_ext_plugin_mod(self):
            fail

        def test_missing_ext_plugin_mod(self):
            fail

        def test_missing_ext_aux_mod(self):
            fail

        def test_missing_multiple_ext_mods(self):
            fail
        
        def test_exception_in_on_load(self):
            fail
        
        def test_exception_during_mod_load(self):
            fail

    @pytest.mark.skip
    def test_extending_origen_cmd_from_global_context_only(self):
        fail

class TestCurrentCommand:
    @pytest.mark.skip
    def test_current_command_from_core_cmd(self):
        eval_cmd

    @pytest.mark.skip
    def test_current_command_from_pl_cmd(self):
        pl_cmd

    @pytest.mark.skip
    def test_current_command_from_aux_cmd(self):
        aux_cmd

    @pytest.mark.skip
    def test_current_command_from_app_cmd(self):
        app_cmd