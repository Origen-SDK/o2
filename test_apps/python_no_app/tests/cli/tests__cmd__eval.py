import origen, pytest
from .shared import CLICommon

class T_Eval(CLICommon):
    _cmd= origen.helpers.regressions.cli.CLI.global_cmds.eval
    _no_config_run_opts = {
        "with_configs": CLICommon.suppress_plugin_collecting_config,
        "bypass_config_lookup": True
    }

    @pytest.fixture
    def no_config_run_opts(self):
        return self._no_config_run_opts

    def test_help_msg(self, cmd, no_config_run_opts):
        help = cmd.get_help_msg(run_opts=no_config_run_opts)
        help.assert_summary(cmd.help)
        help.assert_args(cmd.code)
        help.assert_bare_opts()

    def test_with_single_statment(self, cmd, no_config_run_opts):
        d = cmd.demos["multi_statement_single_arg"]
        out = d.run(run_opts=no_config_run_opts)
        d.assert_present(out)

    def test_with_multiple_statements(self, cmd, no_config_run_opts):
        d = cmd.demos["multi_statement_multi_args"]
        out = d.run(run_opts=no_config_run_opts)
        d.assert_present(out)

    def test_error_with_no_input(self, cmd, no_config_run_opts):
        err = cmd.gen_error(run_opts=no_config_run_opts)
        assert self.err_msgs.missing_required_arg(cmd.code) in err
    
    def test_error_in_statements(self, cmd, no_config_run_opts):
        d = cmd.demos["gen_name_error"]
        out = d.gen_error(run_opts=no_config_run_opts)
        d.assert_present(out)

    # FOR_PR
    @pytest.mark.skip
    def test_error_in_statements_still_runs_cleanup(self, cmd, no_config_run_opts):
        d = cmd.demos["gen_name_error"]
        out = d.gen_error(return_full=True)
        d.assert_present(out["stderr"])
        print(out["stdout"])
        fail
