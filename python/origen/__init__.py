import _origen
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

    import importlib
    a = importlib.import_module(f'{_origen.app_config()["name"]}.application')
    app = a.Application()

else:
    app = None

dut = None
tester = None
