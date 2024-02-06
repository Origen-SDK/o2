import origen
from origen_metal.utils.version import Version
from origen_metal._helpers import pip_show

class TestOrigenVersion:
    _current_version = None

    def current_version(self):
        if self._current_version is None:
            self._current_version = pip_show('origen', wrap_poetry=True).version
        return self._current_version

    def test_origen_version(self):
        assert isinstance(origen.version, Version)
        assert str(origen.version) == self.current_version()

    def test_origen_version_str(self):
        assert origen.__version__ == self.current_version()
