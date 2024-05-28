import pytest, origen
from .shared import CLICommon
from .core_cmds.aux_cmds import T_AuxCmds
from .core_cmds.env import T_Env
from .core_cmds.generate import T_Generate
from .core_cmds.target import T_Target
from .core_cmds.app import T_App
from .core_cmds.plugin import T_Plugin
from .core_cmds.plugins import T_Plugins

class T_AppWorkspaceCoreCommands(CLICommon):
    def test_app_workspace_help_message(self):
        help = self.in_app_cmds.origen.get_help_msg()
        assert help.root_cmd is True
        assert "Origen CLI: 2." in help.version_str
        help.assert_bare_opts()

        assert set(help.subcmd_names) == set(self.in_app_cmds.all_names_add_help)
        assert help.app_cmd_shortcuts == self.cmd_shortcuts.app
        assert help.pl_cmd_shortcuts == self.cmd_shortcuts.pl
        assert help.aux_cmd_shortcuts == self.cmd_shortcuts.aux

    @pytest.mark.parametrize("cmd", CLICommon.in_app_cmds.cmds, ids=CLICommon.in_app_cmds.all_names)
    def test_core_commands_are_available(self, cmd):
        ''' Just testing that "-h" doesn't crash for all core commands '''
        help = cmd.get_help_msg()
        assert len(help.opts) >= 3
        # TESTS_NEEDED CLI add check for app opts when applicable

    class TestApp(T_App):
        pass

    class TestAuxCmds(T_AuxCmds):
        pass

    class TestEnv(T_Env):
        pass

    class TestEval(CLICommon):
        _cmd= origen.helpers.regressions.cli.CLI.in_app_cmds.eval

        def test_help_msg(self, cmd, no_config_run_opts):
            help = cmd.get_help_msg(run_opts=no_config_run_opts)
            help.assert_summary(cmd.help)
            help.assert_args(cmd.code)
            opts = list(self.in_app_cmds.standard_opts())
            opts.insert(3, cmd.scripts)
            help.assert_opts(*opts)

        def test_basic_eval(self, cmd, no_config_run_opts):
            d = cmd.demos["multi_statement_single_arg"]
            out = d.run(run_opts=no_config_run_opts)
            d.assert_present(out)

    class TestGenerate(T_Generate):
        pass

    class TestPlugin(T_Plugin):
        pass

    class TestPlugins(T_Plugins):
        pass

    class TestTarget(T_Target):
        pass

    class TestInteractive(CLICommon):
        _cmd= origen.helpers.regressions.cli.CLI.in_app_cmds.i

        def test_help_msg(self, cmd, no_config_run_opts):
            help = cmd.get_help_msg(run_opts=no_config_run_opts)
            help.assert_summary(cmd.help)
            help.assert_args(None)
            help.assert_bare_app_opts()

        @pytest.mark.skip
        def test_interactive(self, cmd, no_config_run_opts):
            # TEST_NEEDED CLI try to get an interactive test that just starts/stops
            proc = subprocess.Popen(["poetry", "run", "origen", "i"], universal_newlines=True, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            try:
                proc.stdin.flush()
                #proc.stdout.flush()
                m = 'print("hi from interactive!")'
                import time
                # time.sleep(10)
                assert proc.poll() is None
                # proc.stdin.write(f"{m}\n".encode())
                assert proc.poll() is None
                # lines = proc.stdout.readlines()
                # print(lines)
                # assert lines[-1] == m

                m = "print('hi again!')"
                # proc.stdin.write(f"{m}\n".encode())
                assert proc.poll() is None
                # lines = proc.stdout.readlines()
                # assert lines[0] == m

                proc.stdin.write("exit()\n")
                assert proc.wait(3) == 0
                lines = proc.stdout.readline()
                print(lines)
            finally:
                if proc.poll() is None:
                    proc.kill()
                # print(proc.stdout.readline())
                # print(proc.stdout.readline())
                # print(proc.stdout.readline())
                # print(proc.stdout.readline())
                for l in proc.stdout:
                    # lines = proc.stdout.readlines()
                    if "CMD" in l:
                        break
                    print(l)
            fail

    # class TestCredentials(CLICommon):
    #     def test_credentials(self):
    #         # TEST_NEEDED CLI credentials from app context
