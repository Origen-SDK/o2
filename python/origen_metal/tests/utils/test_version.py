import origen_metal
import pytest
from origen_metal._helpers import pip_show
from origen_metal.utils import version
from origen_metal.utils.version import Version, semver, pep440
from pathlib import Path
from tests import test_dir

class TestVersion:
    '''
        Only need to test the Python API.
        Other tests of Version itself are handled on the Rust side
    '''

    pyproject_path = test_dir.parent.joinpath("pyproject.toml")
    cargo_path = test_dir.parent.parent.parent.joinpath("rust/pyapi_metal/Cargo.toml")
    pep440_str = "1.2.3.dev4"
    semver_str = "1.2.3-dev.4"

    def test_version_form_string(self):
        ver = Version(self.pep440_str)
        assert isinstance(ver, Version)
        assert str(ver) == self.pep440_str

    def test_version_as_pep440_is_default(self):
        ver = Version(self.semver_str)
        assert str(ver) == self.pep440_str

    def test_pep440_from_string(self):
        ver = pep440(self.pep440_str)
        assert str(ver) == self.pep440_str
        ver = pep440(self.semver_str)
        assert str(ver) == self.pep440_str

    def test_semver_from_string(self):
        ver = semver(self.pep440_str)
        assert str(ver) == self.semver_str
        ver = semver(self.semver_str)
        assert str(ver) == self.semver_str

    def test_invalid_version_from_string(self):
        with pytest.raises(ValueError, match=r"unexpected character 'b' while parsing minor version number"):
            Version("1.b.c")

    def test_version_from_pyproject(self):
        # path as string
        v = version.from_pyproject(str(self.pyproject_path))
        assert str(v) == current_version()

        # path as pathlib.Path
        v = version.from_pyproject(self.pyproject_path)
        assert str(v) == current_version()

    def test_invalid_pyproject_path(self):
        f = "path/to/nowhere/pyproject.toml"
        with pytest.raises(RuntimeError, match=f"Source file '{f}' does not exist!"):
            version.from_pyproject(f)

    def test_version_from_cargo(self):
        print(self.cargo_path)
        # path as string
        v = version.from_cargo(str(self.cargo_path))
        assert str(v) == origen_metal._origen_metal.__version__

        # path as pathlib.Path
        v = version.from_cargo(self.cargo_path)
        assert str(v) == origen_metal._origen_metal.__version__

    def test_invalid_cargo_path(self):
        f = "path/to/nowhere/cargo.toml"
        with pytest.raises(RuntimeError, match=f"Source file '{f}' does not exist!"):
            version.from_cargo(f)
    
    def test_to_semver_string(self):
        ver = Version(self.pep440_str)
        assert(str(ver)) == self.pep440_str
        assert(ver.semver_str) == self.semver_str

    def test_to_pep440_string(self):
        ver = semver(self.semver_str)
        assert(str(ver)) == self.semver_str
        assert(ver.pep440_str) == self.pep440_str

    def test_switching_from_pep400_to_semver(self):
        ver = pep440(self.pep440_str)
        assert ver.is_pep440 is True
        assert ver.is_semver is False
        assert(str(ver)) == self.pep440_str

        assert ver.convert_to_pep440() is False
        assert(str(ver)) == self.pep440_str

        assert ver.convert_to_semver() is True
        assert ver.is_pep440 is False
        assert ver.is_semver is True
        assert(str(ver)) == self.semver_str

    def test_switching_from_semver_to_pep440(self):
        ver = semver(self.semver_str)
        assert ver.is_semver is True
        assert ver.is_pep440 is False
        assert(str(ver)) == self.semver_str

        assert ver.convert_to_semver() is False
        assert(str(ver)) == self.semver_str

        assert ver.convert_to_pep440() is True
        assert ver.is_semver is False
        assert ver.is_pep440 is True
        assert(str(ver)) == self.pep440_str

_current_version = None
def current_version():
    global _current_version
    if _current_version is None:
        _current_version = pip_show('origen_metal', wrap_poetry=True).version
    return _current_version

def test_om_version():
    assert isinstance(origen_metal.version, Version)
    assert str(origen_metal.version) == current_version()

def test_om_version_str():
    assert origen_metal.__version__ == current_version()
