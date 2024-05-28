import pathlib
from cli.shared import CLICommon

class TestBadConfigs(CLICommon):
    @property
    def bad_ldap_config_path(self):
        # TODO use a shared method for LDAP config path
        return pathlib.Path(__file__).parent.joinpath("origen_utilities/configs/ldap/test_bad_ldap_config.toml").absolute()

    @property
    def missing_toml(self):
        return self.to_config_path("missing.toml").absolute()

    @property
    def eval_m(self):
        return "hello from eval"

    def test_origen_v(self):
        err = self.cmds.v.gen_error(
            return_full=True, 
            run_opts=self.no_config_run_opts_plus_config(self.bad_ldap_config_path)
        )
        assert err["returncode"] == 1
        p = pathlib.Path("tests/origen_utilities/configs/ldap/test_bad_ldap_config.toml")
        assert f"invalid type: string \"hi\", expected an integer for key `ldaps.bad.timeout` in {str(p)}" in err["stdout"]
        assert err["stderr"] == ""

    def test_origen_cmd(self):
        err = self.cmds.eval.gen_error(
            f"print( '{self.eval_m}' )",
            return_full=True, 
            run_opts=self.no_config_run_opts_plus_config(self.bad_ldap_config_path)
        )
        assert err["returncode"] == 1
        p = pathlib.Path("tests/origen_utilities/configs/ldap/test_bad_ldap_config.toml")
        assert f"invalid type: string \"hi\", expected an integer for key `ldaps.bad.timeout` in {str(p)}" in err["stdout"]
        assert self.eval_m not in err["stdout"]
        assert err["stderr"] == ""

    def test_bad_config_path(self):
        err = self.cmds.eval.gen_error(
            f"print( '{self.eval_m}' )",
            return_full=True, 
            run_opts=self.no_config_run_opts_plus_config(self.missing_toml)
        )
        assert err["returncode"] == 1
        assert "missing.toml either does not exists or is not accessible" in err["stdout"]
        assert self.eval_m not in err["stdout"]
        assert err["stderr"] == ""
