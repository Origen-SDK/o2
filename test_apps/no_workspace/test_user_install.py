import os
from .t_invocation_env import T_InvocationEnv, no_workspace_test_dir, PyProjectSrc

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


# TEST_NEEDED User Invocations: Error Cases
# @pytest.mark.skip
# class TestErrorCases(T_InvocationEnv):
#     def test_origen_pkg_not_installed(self):
#         fail

#     def test_missing_pyproject(self):
#         fail
