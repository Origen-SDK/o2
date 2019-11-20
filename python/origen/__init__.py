import _origen;
from pathlib import Path
import sys
from pathlib import Path

config = _origen.config()
status = _origen.status()
root = Path(status["root"])
version = status["origen_version"]

if status["is_app_present"]:
    # Add app's lib directory to the load path
    app_lib = root.joinpath("app").joinpath("lib")
    sys.path.insert(0, str(app_lib))
    print(sys.path)

    import importlib
    #app_module = importlib.import_module(_origen.app_config()["name"], package=None)
    = importlib.import_module("application2", package="example")
    #app = c.Application()
    import example.application2
    app = example.application2.Application()

    #import example
else:
    app = None

dut = None
tester = None
