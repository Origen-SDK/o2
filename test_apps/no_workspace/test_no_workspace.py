from .t_invocation_env import T_InvocationEnv, no_workspace_test_dir, PyProjectSrc, site_packages_dir

class TestNoWorkspaceNoPlugins(T_InvocationEnv):
    user_install_dir = no_workspace_test_dir.joinpath("user_install")

    @classmethod
    def set_params(cls):
        cls.target_pyproj_dir = None
        cls.invocation = PyProjectSrc.NoneFound
        cls.cli_dir = site_packages_dir.joinpath("origen/__bin__/bin")
        cls.has_pls = False

    @classmethod
    def setup_method(cls):
        cls.file_based_evals = True
        super().setup_method()

class TestNoWorkspaceWithPlugins(TestNoWorkspaceNoPlugins):
    @classmethod
    def set_params(cls):
        super().set_params()
        cls.has_pls = True

    def test_exts_in_user_global_context(self):
        out = self.global_cmds.eval.run("1==1", "-b", "-a")
        assert "Hi from python-plugin during 'eval'!" in out
        assert "Hi again from python-plugin during 'eval'!" in out
