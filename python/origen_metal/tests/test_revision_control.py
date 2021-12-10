import pytest, importlib
from origen_metal._origen_metal import __test__
from origen_metal.utils.revision_control.supported import Git
from origen_metal import utils


def test_modules_revision_control_and_rc_aliased():
    assert utils.revision_control == utils.rc


def test_git_can_instantiated():
    driver = Git({"local": "./", "remote": "test.git"})
    assert isinstance(driver, Git)
    assert driver.system() == "Git"


def test_pyapi_internal_git_path_is_valid():
    git_mod = importlib.import_module(__test__.python_git_mod_path())
    assert git_mod.Git == Git
