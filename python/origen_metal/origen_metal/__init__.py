# This needs to be commented out to run pdoc, but is required for functionality

import sys

if "origen_metal._origen_metal" in sys.modules:
    # If "origen_metal._origen_metal" is already defined,
    # use this library instead of the native one
    from origen_metal._origen_metal import *
    _origen_metal = sys.modules["origen_metal._origen_metal"]
else:
    from ._origen_metal import *

sessions = _origen_metal.framework.sessions.sessions()
