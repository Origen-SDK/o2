import origen, pytest, pathlib
from .shared import CLICommon

class T_Eval(CLICommon):
    _cmd= origen.helpers.regressions.cli.CLI.global_cmds.eval
    _no_config_run_opts = {
        "with_configs": CLICommon.configs.suppress_plugin_collecting_config,
        "bypass_config_lookup": True
    }
    script_dir = pathlib.Path(__file__).parent.joinpath("tests__cmd__eval__scripts")

    def eval_script(self, name):
        return self.script_dir.joinpath(f"{name}.py")

    @pytest.fixture
    def no_config_run_opts(self):
        return self._no_config_run_opts

    def test_help_msg(self, cmd, no_config_run_opts):
        help = cmd.get_help_msg(run_opts=no_config_run_opts)
        help.assert_cmd(cmd)

    def test_with_single_statement(self, cmd, no_config_run_opts):
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

    def test_clean_eval(self):
        eval_prefix = "Origen Version From Eval: "
        out = self.eval("print (f'" + eval_prefix + "{origen.version}' )")
        assert out == f"{eval_prefix}{origen.version}\n"

    def test_eval_with_script(self, cmd):
        out = cmd.run(cmd.scripts.ln_to_cli(), self.eval_script("hi"))
        assert out == "eval_script__say_hi: hi!\n"

        out = cmd.run(
            cmd.scripts.ln_to_cli(), self.eval_script("override_preface"),
            cmd.scripts.ln_to_cli(), self.eval_script("hi")
        )
        assert out == "eval_script_override: hi!\n"

    def test_eval_context_persists_over_scripts(self, cmd):
        out = cmd.run(
            "preface='preface_override'",
            cmd.scripts.ln_to_cli(), self.eval_script("hi"),
            "print(hi)"
        )
        assert out == "preface_override: hi!\nhi!\n"

    def test_error_in_script(self, cmd):
        err = self.eval_script("err")
        out = cmd.gen_error(
            "print('test_error_in_script')",
            cmd.scripts.ln_to_cli(), err,
            return_full=True
        )

        stdout = out["stdout"]
        errs = self.extract_logged_errors(stdout)
        assert errs[0] == f"Exception occurred evaluating from script '{err}'"
        assert len(errs) == 1

        stdout = stdout.split("\n")
        assert "test_error_in_script" in stdout[1]
        assert stdout[2] == "tests__cmd__eval__scripts: gen error"
        assert stdout[3] == ''
        assert len(stdout) == 4

        stderr = out["stderr"].split("\n")
        assert "Traceback" in stderr[0]
        assert "line 2" in stderr[1]
        assert stderr[2] == "NameError: name 'hello' is not defined"
        assert stdout[3] == ''
        assert len(stderr) == 4

    def test_eval_with_invalid_script(self, cmd):
        invalid = self.eval_script("invalid")
        out = cmd.gen_error(
            "print('hi')",
            cmd.scripts.ln_to_cli(), invalid,
            return_full=True
        )
        assert out["stderr"] == ''
        assert f"Could not find script file '{invalid}'" in out['stdout']
