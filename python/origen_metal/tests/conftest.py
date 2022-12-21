import tests.shared
from tests.shared import *

def pytest_collection_modifyitems(items):
    for item in items:
        for m in item.iter_markers():
            if m.name == "ldap":
                item.add_marker(pytest.mark.online)
