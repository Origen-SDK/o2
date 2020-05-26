import sys
import _origen
from pathlib import Path
import importlib
from contextlib import contextmanager
import pickle
from origen.helpers.doc import internal_members
from typing import List, Dict

from origen.tester import Tester, DummyTester
from origen.producer import Producer

config = _origen.config()
''' Dictionary of configurable workspace settings.

    Keys include: ``{{ list(origen.config.keys())|pprint }}``

    Returns:
        dict: Configurable workspace settings.

    See Also
    ---------
    :ref:`Configuring Origen <guides/getting_started/configuring_your_workspace:Configuring Your Workspace>`
'''

status = _origen.status()
''' Dictionary of various application and workspace attributes
    Keys include: ``{{ list(origen.status.keys())|pprint }}``

    Returns:
        dict: Application and/or workspace attributes as key-value pairs.
'''

root = Path(status["root"])
''' If applicable, returns the application's root.

    Returns:
        pathlib.OSPath: Application's root as an OS-specific path object.
        None: If not in an application's workspace.
'''

version = status["origen_version"]
''' Returns the version of the Origen executable.

    Returns:
        str: Origen executable version

    >>> origen.version
    '{{ origen_version }}'
'''

logger = _origen.logger
''' Direct access to the build-in logger module for logging and displaying user-friendly output.

    Returns:
        logger: Pointer to _origen.logger

    See Also
    --------
    :mod:`_origen.logger`
    {{ ref_for('logger', 'Logging Output') }}
'''

running_on_windows = _origen.on_windows()
''' Indicates if Origen is currently running on Windows.

    Returns:
        True: Origen is currently executing on Windows
        False: origen is currently __not__ executing on Windows

    >>> origen.running_on_windows
    False
'''

running_on_linux = _origen.on_linux()
''' Indicates if Origen is currently running on Linux.

    Returns:
        True: Origen is currently executing on Linux
        False: origen is currently __not__ executing on Linux

    >>> origen.running_on_linux
    True
'''

_reg_description_parsing = False

frontend_root = Path(__file__).parent.absolute()
''' Returns the directory of the ``origen`` module
    as a `pathlib.Path <https://docs.python.org/3/library/pathlib.html#concrete-paths>`_
'''

app = None
''' Pointer to the current application instance, or ``None``, if Origen was not invoked from within an application workspace.

    If ``app`` is not ``None``, it should be chld of the base class :class:`origen.application.Base`

    See Also
    --------
    :ref:`The Application Workspace <guides/getting_started/workspaces:The Application Workspace>`
'''

dut = None
''' Pointer to the current DUT, or ``None``, if no DUT has been set.

    if ``dut`` is not ``None``, then it should be a child of the base class :class:`origen.controller.Base`
'''

tester = Tester()
''' Pointer to the global tester object, :class:`origen.tester.Tester`
'''

producer = Producer()
''' Pointer to the global producer object, :py:class:`origen.producer.Producer`
'''
mode = "development"

if status["is_app_present"]:
    sys.path.insert(0, status["root"])
    a = importlib.import_module(f'{_origen.app_config()["name"]}.application')
    app = a.Application()

def set_mode(val: str) -> None:
    """ Sets the current mode """
    global mode
    if val:
        mode = _origen.clean_mode(val)

def load_file(path, globals={}, locals={}):
    context = {**standard_context(), **locals}
    with open(path) as f:
        code = compile(f.read(), path, 'exec')
        exec(code, globals, context)

def test_ast() -> List[str]:
    ''' Returns a serialized representation of the AST '''
    return pickle.loads(bytes(_origen.test_ast()))

@contextmanager
def reg_description_parsing():
    global _reg_description_parsing
    orig = _reg_description_parsing
    _reg_description_parsing = True
    yield
    _reg_description_parsing = orig

def standard_context():
    ''' Returns the context (locals) that are available by default within files
        loaded by Origen, e.g. dut, tester, origen, etc.
    '''

    return {
        "origen": sys.modules[__name__],
        "dut": lambda: __import__("origen").dut,
        "tester": lambda: __import__("origen").tester,
    }

__all__ = [
    *internal_members(sys.modules[__name__]),
    'config',
    'status',
    'root',
    'version',
    'logger',
    'running_on_windows',
    'running_on_linux',
    'frontend_root',
    'app',
    'dut',
    'tester',
    'producer'
]