import sys, pathlib, importlib, contextlib

sys.path.insert(0, str(pathlib.Path(__file__).parent.parent.joinpath("tests")))

# Allow reuse of shared stuff, base classes, helpers, etc. from origen_metal
# However, can't straight import it since it overrides some stuff - want them in separate namespaces.
# Imported manually here as "om_tests"
# Other tests can import as needed. E.g.:
#
# from om_tests import test_frontend
# test_frontend.TestRevisionControlFrontend.DummyRC()
#
import tests as om_tests
_om_tests_root = pathlib.Path(__file__).parent.joinpath("../../../python/origen_metal/tests/__init__.py").resolve()
_om_tests_spec = importlib.util.spec_from_file_location("om_tests", str(_om_tests_root))
_om_tests = importlib.util.module_from_spec(_om_tests_spec)
sys.modules["om_tests"] = _om_tests
_om_tests_spec.loader.exec_module(_om_tests)

@contextlib.contextmanager
def om_shared():
    _tests = sys.modules["tests"]
    _tests_shared = sys.modules["tests.shared"]
    sys.modules["tests"] = sys.modules["om_tests"]
    sys.modules["tests.shared"] = sys.modules["om_tests.shared"]
    try:
        yield
    finally:
        sys.modules["tests.shared"] = _tests_shared
        sys.modules["tests"] = _tests
