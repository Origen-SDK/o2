import os
from .t_invocation_env import T_InvocationEnv, T_InvocationBaseTests, no_workspace_test_dir, PyProjectSrc

class TestUserInstall(T_InvocationEnv):
    user_install_dir = no_workspace_test_dir.joinpath("user_install")

    @classmethod
    def set_params(cls):
        cls.local_origen = True
        cls.has_pls = True
        cls.target_pyproj_dir = cls.user_install_dir
        cls.move_pyproject = False
        cls.invocation = PyProjectSrc.UserGlobal
        cls.cli_dir = cls.debug_cli_dir
        cls.file_based_evals = True

    @classmethod
    def setup_method(cls):
        super().setup_method()
        os.environ["ORIGEN_PYPROJECT"] = str(cls.user_install_dir)

    @classmethod
    def teardown_method(cls):
        del os.environ["ORIGEN_PYPROJECT"]

    def test_exts_in_user_global_context(self):
        out = self.global_cmds.eval.run("print('hi with exts')", "-b", "-a")
        assert "Hi from python-plugin during 'eval'!" in out
        assert "Hi again from python-plugin during 'eval'!" in out

class TestErrorCasesWithFallback():
    class TestInvalidPyProjectDir(T_InvocationBaseTests):
        invalid_install_dir = no_workspace_test_dir.joinpath("no_dir")

        @classmethod
        def set_params(cls):
            cls.local_origen = True
            cls.has_pls = True
            cls.move_pyproject = False
            cls.file_based_evals = True
            cls.error_case = f"Errors encountered resolving pyproject: ORIGEN_PYPROJECT '{cls.invalid_install_dir}' does not exists!"
            cls.error_case_global_fallback = True

            cls.invocation = None
            cls.cli_dir = cls.site_cli_dir
            cls.target_pyproj_dir = None


        @classmethod
        def setup_method(cls):
            super().setup_method()
            os.environ["ORIGEN_PYPROJECT"] = str(cls.invalid_install_dir)

        @classmethod
        def teardown_method(cls):
            del os.environ["ORIGEN_PYPROJECT"]

        def test_error_message(self):
            out = self.global_cmds.eval.run("1==1")
            errors = self.extract_logged_errors(out)
            assert errors[0] == f"Errors encountered resolving pyproject: ORIGEN_PYPROJECT '{self.invalid_install_dir}' does not exists!"
            assert errors[1] == "Dependency source has not been set - defaulting to global Python installation"
            assert errors[2] == "Dependency source has not been set - defaulting to global Python installation"
            assert len(errors) == 3

    class TestMissingPyProject(T_InvocationBaseTests):
        missing_pyproject = no_workspace_test_dir

        @classmethod
        def set_params(cls):
            cls.local_origen = True
            cls.has_pls = True
            cls.move_pyproject = False
            cls.file_based_evals = True
            cls.error_case = f"Errors encountered resolving pyproject: Could not locate pyproject.toml from ORIGEN_PYPROJECT {cls.missing_pyproject.joinpath('pyproject.toml')}"
            cls.error_case_global_fallback = True

            cls.invocation = None
            cls.cli_dir = cls.site_cli_dir
            cls.target_pyproj_dir = None

        @classmethod
        def setup_method(cls):
            super().setup_method()
            os.environ["ORIGEN_PYPROJECT"] = str(cls.missing_pyproject)

        @classmethod
        def teardown_method(cls):
            del os.environ["ORIGEN_PYPROJECT"]

        def test_error_message(self):
            out = self.global_cmds.eval.run("1==1")
            errors = self.extract_logged_errors(out)
            print(out)
            assert errors[0] == f"Errors encountered resolving pyproject: Could not locate pyproject.toml from ORIGEN_PYPROJECT {self.missing_pyproject.joinpath('pyproject.toml')}"
            assert errors[1] == "Dependency source has not been set - defaulting to global Python installation"
            assert errors[2] == "Dependency source has not been set - defaulting to global Python installation"
            assert len(errors) == 3

    class TestMalformedPyProject():
        # Use the template pyproject as an example of a malformed one
        malformed_pyproject = no_workspace_test_dir.joinpath("templates")

        @classmethod
        def set_params(cls):
            cls.local_origen = True
            cls.has_pls = True
            cls.move_pyproject = False
            cls.file_based_evals = True

            cls.invocation = None
            cls.cli_dir = cls.site_cli_dir
            cls.target_pyproj_dir = None

        @classmethod
        def setup_method(cls):
            os.environ["ORIGEN_PYPROJECT"] = str(cls.malformed_pyproject)

        @classmethod
        def teardown_method(cls):
            del os.environ["ORIGEN_PYPROJECT"]

        def test_error_message(self):
            # Pyproject found but malformed should print the poetry errors as it tries to run.
            # Should not fall back to global install, even if its available. Pyproject should be fixed.
            out = T_InvocationBaseTests.global_cmds.eval.run("1==1", run_opts={"return_details": True})
            err = f"Invalid TOML file {self.malformed_pyproject.joinpath('pyproject.toml').as_posix()}"
            assert err in out["stderr"]
            errors = T_InvocationBaseTests.extract_logged_errors(out["stdout"])
            assert err in errors[1]
