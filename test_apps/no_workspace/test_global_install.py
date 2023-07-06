from .t_invocation_env import T_InvocationEnv, PyProjectSrc

class GlobalInstalBase(T_InvocationEnv):
    @classmethod
    def set_params(cls):
        cls.local_origen = True
        cls.invocation = PyProjectSrc.Global
        cls.cli_dir = cls.site_cli_dir
        cls.file_based_evals = True

# Pyproject closer to root - no plugins
class TestGlobalInstallNoPlugins(GlobalInstalBase):
    @classmethod
    def set_params(cls):
        super().set_params()
        cls.target_pyproj_dir = cls.site_packages_dir.parent
        cls.has_pls = False

# Pyproject at site-packages dir - with plugins
class TestGlobalInstallWithPlugins(GlobalInstalBase):
    @classmethod
    def set_params(cls):
        super().set_params()
        cls.target_pyproj_dir = cls.site_packages_dir
        cls.has_pls = True

# Pyproject at binary location - no plugins
class TestGlobalInstallAtCliDir(GlobalInstalBase):
    @classmethod
    def set_params(cls):
        super().set_params()
        cls.target_pyproj_dir = cls.site_cli_dir
        cls.has_pls = False
