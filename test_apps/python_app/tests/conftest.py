import os, pytest
from ._shared import tmp_dir

pytest.register_assert_rewrite("origen.helpers.regressions")
pytest.register_assert_rewrite("test_apps_shared_test_helpers")
pytest.register_assert_rewrite("cli.tests__core_cmds")

# Move the session store into a local test directory
os.environ['origen_session__user_root'] = str(tmp_dir())
os.environ['origen_app_app_session_root'] = str(tmp_dir())

def pytest_collection_modifyitems(items):
    for item in items:
        for m in item.iter_markers():
            if m.name == "ldap":
                item.add_marker(pytest.mark.online)
