import origen, origen_metal, _origen, getpass, pytest, pathlib, sys
from .shared import working_dir

sys.path.insert(-1, str(pathlib.Path(__file__).parent.parent.parent.joinpath("no_workspace")))
from t_invocation_env import T_InvocationBaseTests

def test_import():
    assert "2." in origen.version

def test_app_is_none():
    assert origen.app is None

def test_is_app_present():
    assert origen.is_app_present is False
    assert _origen.is_app_present() is False
    assert origen.status["is_app_present"] is False

class TestWorkspaceInvocation(T_InvocationBaseTests):
    @classmethod
    def set_params(cls):
        cls.invocation = cls.PyProjectSrc.Workspace
        cls.target_pyproj_dir = working_dir

class TestGlobalFEIntegration:
    def test_frontend_is_accessible(self):
        assert (origen_metal.frontend.frontend() is not None)

    def test_current_user_is_available(self):
        assert origen.current_user.id == getpass.getuser()
    
    @pytest.mark.skip
    def test_datastores_are_available(self):
        # TEST_NEEDED Datastores in global invocation
        assert origen.datastores.keys() == ['ldaps']