# This needs to be commented out to run pdoc, but is required for functionality

import sys, types

if "origen_metal._origen_metal" in sys.modules:
    # If "origen_metal._origen_metal" is already defined,
    # use this library instead of the native one
    from origen_metal._origen_metal import *
    _origen_metal = sys.modules["origen_metal._origen_metal"]
else:
    from ._origen_metal import *

sessions = _origen_metal.framework.sessions.sessions()
users = _origen_metal.framework.users.users()

running_on_windows = _origen_metal.running_on_windows
running_on_linux = _origen_metal.running_on_linux


# https://www.python.org/dev/peps/pep-0562/
def __getattr__(name):
    if name == "current_user":
        return users.current_user
    elif name == "version":
        from origen_metal.utils.version import Version
        return Version(__getattr__("__version__"))
    elif name == "__version__":
        import importlib_metadata
        return importlib_metadata.version(__name__)
    else:
        raise AttributeError(f"module '{__name__}' has no attribute '{name}'")
