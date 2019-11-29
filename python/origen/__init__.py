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
mode = "development"

if status["is_app_present"]:
    sys.path.insert(0, status["root"])

def set_mode(val):
    global mode
    if val:
        mode = _origen.clean_mode(val)

def load_file(path, globals={}, locals={}):
    with open(path) as f:
        code = compile(f.read(), path, 'exec')
        exec(code, globals, locals)
