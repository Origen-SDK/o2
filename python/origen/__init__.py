import sys
import _origen
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

    if app.current_target["target_file"]:
        # Also see:
        #https://docs.python.org/2/library/imp.html#imp.load_source
        global_vars = {}
        local_vars = {}
        with open(app.current_target["target_file"]) as f:
            code = compile(f.read(), "/home/stephen/Code/github/reboot/example/app/lib/p2.py", 'exec')
            exec(code, global_vars, local_vars)
        

else:
    app = None

dut = None
tester = None
