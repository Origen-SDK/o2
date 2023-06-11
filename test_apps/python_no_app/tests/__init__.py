import sys, pathlib, importlib, contextlib
import pytest

sys.path.insert(0, str(pathlib.Path(__file__).parent.parent.joinpath("tests")))

import tests as python_app_tests
_python_app_tests_root = pathlib.Path(__file__).parent.joinpath(
    "../../python_app/tests/__init__.py").resolve()
_python_app_tests_spec = importlib.util.spec_from_file_location(
    "python_app_tests", str(_python_app_tests_root))
_python_app_tests = importlib.util.module_from_spec(_python_app_tests_spec)
sys.modules["python_app_tests"] = _python_app_tests
_python_app_tests_spec.loader.exec_module(_python_app_tests)

@contextlib.contextmanager
def python_app_shared():
    _tests_modules_ = dict(
        filter(lambda mod: mod[0].split('.')[0] == "tests",
               sys.modules.items()))
    for name, mod in _tests_modules_.items():
        if f"python_app_{name}" in sys.modules:
            sys.modules[name] = sys.modules[f"python_app_{name}"]
        else:
            sys.modules.pop(name)
    try:
        yield
    finally:
        for name, mod in _tests_modules_.items():
            sys.modules[name] = mod

# Have pytest's assert rewriting take over:
# https://docs.pytest.org/en/stable/writing_plugins.html#assertion-rewriting
# pytest.register_assert_rewrite("tests.shared")
pytest.register_assert_rewrite("tests.cmd_building")
pytest.register_assert_rewrite("tests.cli")
pytest.register_assert_rewrite("origen.helpers.regressions")
pytest.register_assert_rewrite("test_apps_shared_test_helpers")
