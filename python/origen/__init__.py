import sys
import re
init_verbosity = 0
for arg in sys.argv:
    regexp = re.compile(r'verbosity=(\d+)')
    matches = regexp.search(arg)
    if matches:
        init_verbosity = int(matches.group(1))
import _origen
_origen.initialize(init_verbosity)
from pathlib import Path
import importlib
from contextlib import contextmanager
import pickle

from origen.tester import Tester, DummyTester
from origen.producer import Producer

import origen.target

config = _origen.config()
status = _origen.status()
root = Path(status["root"])
version = status["origen_version"]
logger = _origen.logger
log = _origen.logger
running_on_windows = _origen.on_windows()
running_on_linux = _origen.on_linux()
_reg_description_parsing = False

app = None
dut = None
tester = Tester()
producer = Producer()
# The application's test program interface, this will be lazily instantiated
# the first time a test program Flow() block is encountered
interface = None

# These vars are used to indenfify when a target load is taking place
_target_loading = False

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
    # Will convert any paths with / to \ on Windows
    path = Path(path)
    log.trace(f"Loading file '{path}'")
    context = {**standard_context(), **locals}
    with open(path) as f:
        code = compile(f.read(), path, 'exec')
        exec(code, globals, context)

def test_ast():
    return pickle.loads(bytes(_origen.test_ast()))

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
        "dut": lambda: __import__("origen").dut,
        "tester": lambda: __import__("origen").tester,
    }