import sys
import _origen
from pathlib import Path
import importlib

config = _origen.config()
status = _origen.status()
root = Path(status["root"])
version = status["origen_version"]
logger = _origen.logger

app = None
dut = None
tester = None
mode = "development"

if status["is_app_present"]:
    sys.path.insert(0, status["root"])
    a = importlib.import_module(f'{_origen.app_config()["id"]}.application')
    app = a.Application()

def set_mode(val):
    global mode
    if val:
        mode = _origen.clean_mode(val)

def load_file(path, globals={}, locals={}):
    context = {**standard_context(), **locals}
    with open(path) as f:
        code = compile(f.read(), path, 'exec')
        exec(code, globals, context)

# Returns the context (locals) that are available by default within files
# loaded by Origen, e.g. dut, tester, origen, etc.
def standard_context():
    return {
        "origen": sys.modules[__name__],
        "dut": dut, 
        "tester": tester, 
    }
