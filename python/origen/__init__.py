import sys
import _origen
from pathlib import Path
import importlib
from contextlib import contextmanager

config = _origen.config()
status = _origen.status()
root = Path(status["root"])
version = status["origen_version"]
logger = _origen.logger
_reg_description_parsing = False

app = None
dut = None
tester = None
mode = "development"

if status["is_app_present"]:
    sys.path.insert(0, status["root"])
    a = importlib.import_module(f'{_origen.app_config()["name"]}.application')
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

@contextmanager
def write_transaction(bit_collection):
    bc = bit_collection.start_write_transaction()
    yield bc
    bc.end_write_transaction()
        
@contextmanager
def verify_transaction(bit_collection):
    bc = bit_collection.start_verify_transaction()
    yield bc
    bc.end_verify_transaction()

@contextmanager
def reg_description_parsing():
    global _reg_description_parsing
    orig = _reg_description_parsing
    _reg_description_parsing = True
    yield
    _reg_description_parsing = orig

# Returns the context (locals) that are available by default within files
# loaded by Origen, e.g. dut, tester, origen, etc.
def standard_context():
    return {
        "origen": sys.modules[__name__],
        "dut": dut, 
        "tester": tester, 
    }
