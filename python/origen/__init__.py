import sys
import _origen
from pathlib import Path

config = _origen.config()
status = _origen.status()
root = Path(status["root"])
version = status["origen_version"]

app = None
dut = None
tester = None

if status["is_app_present"]:
    sys.path.insert(0, str(root))

    import importlib
    a = importlib.import_module(f'{_origen.app_config()["name"]}.application')
    app = a.Application()

    if app.current_target["target_file"]:
        # Also see:
        #https://docs.python.org/2/library/imp.html#imp.load_source
        global_vars = {}
        local_vars = {}
        with open(app.current_target["target_file"]) as f:
            code = compile(f.read(), app.current_target["target_file"], 'exec')
            exec(code, global_vars, local_vars)
