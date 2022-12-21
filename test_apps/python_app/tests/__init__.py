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
_om_tests_root = pathlib.Path(__file__).parent.joinpath(
    "../../../python/origen_metal/tests/__init__.py").resolve()
_om_tests_spec = importlib.util.spec_from_file_location(
    "om_tests", str(_om_tests_root))
_om_tests = importlib.util.module_from_spec(_om_tests_spec)
sys.modules["om_tests"] = _om_tests
_om_tests_spec.loader.exec_module(_om_tests)


@contextlib.contextmanager
def om_shared():
    _tests_modules_ = dict(
        filter(lambda mod: mod[0].split('.')[0] == "tests",
               sys.modules.items()))
    for name, mod in _tests_modules_.items():
        if f"om_{name}" in sys.modules:
            sys.modules[name] = sys.modules[f"om_{name}"]
        else:
            sys.modules.pop(name)
    try:
        yield
    finally:
        for name, mod in _tests_modules_.items():
            sys.modules[name] = mod
