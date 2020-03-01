import sys
import _origen
from pathlib import Path
import importlib

from origen.tester import Tester, DummyTester
from origen.producer import Producer

config = _origen.config()
status = _origen.status()
root = Path(status["root"])
version = status["origen_version"]
logger = _origen.logger

app = None
dut = None
tester = Tester()
producer = Producer()
#tester = _origen.tester.PyTester("placeholder")
#tester.register_generator(DummyTester)

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

# Returns the context (locals) that are available by default within files
# loaded by Origen, e.g. dut, tester, origen, etc.
def standard_context():
    return {
        "origen": sys.modules[__name__],
        "dut": dut, 
        "tester": tester,
    }

# class Tester:
#     def __init__(self, path):
#         self.path = path
#         self.db = _origen.tester.PyTester(path)

# def instantiate_tester(path):
#     t = Tester(path)
#     # -- do some error checking here ---
#     # ...
#     tester = t
#     return tester

# Returns the dummy tester
#def instantiate_dummy_tester():
#    return instantiate_tester(origen.testers.dummy)
