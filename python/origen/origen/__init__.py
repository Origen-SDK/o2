import sys
import re
import os
init_verbosity = 0
cli_path = None
cli_ver = None
vks = []

regexp = re.compile(r'verbosity=(\d+)')
cli_re = re.compile(r'origen_cli=(.+)')
cli_ver_re = re.compile(r'origen_cli_version=(.+)')
vk_re = re.compile(r'verbosity_keywords=(.+)')
for arg in sys.argv:
    matches = regexp.search(arg)
    if matches:
        init_verbosity = int(matches.group(1))
    else:
        matches = vk_re.search(arg)
        if matches:
            vks = matches.group(1).split(",")
        else:
            matches = cli_re.search(arg)
            if matches:
                cli_path = matches.group(1)
                next
            matches = cli_ver_re.search(arg)
            if matches:
                cli_ver = matches.group(1)
                next

import _origen
from _origen import _origen_metal

# Replace origen_metal's native _origen_metal built library
# with the one built from origen.
sys.modules["origen_metal._origen_metal"] = _origen_metal
_origen.initialize(init_verbosity, vks, cli_path, cli_ver)

# Initialize origen_metal's frontend
import origen_metal
om = origen_metal
origen_metal.frontend.initialize()

from pathlib import Path
import importlib
from contextlib import contextmanager
import pickle
from origen.helpers.doc import internal_members
from typing import List, Dict

from origen.tester import Tester, DummyTester
from origen.producer import Producer

import origen.target

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

root = None
''' If applicable, returns the application's root.

    Returns:
        pathlib.Path: Application's root as an OS-specific path object.
        None: If not in an application's workspace.
'''

__console_history_file__ = None
''' History file when ``origen i`` is run. Only valid when an app is present.
'''

if status["is_app_present"]:
    root = Path(status["root"])
    __console_history_file__ = root.joinpath(".origen").joinpath(
        "console_history")

__in_origen_core_app = status["in_origen_core_app"]
''' Indicates if the current application is the Origen core package

    Returns:
        bool
'''

version = _origen.version()
''' Returns the version of the Origen executable.

    Returns:
        str: Origen executable version

    >>> origen.version
    '{{ origen_version }}'
'''

logger = _origen.logger
''' Direct access to the build-in logger module for logging and displaying user-friendly output. Also available as :data:`log`

    Returns:
        _origen.logger: Pointer to _origen.logger

    See Also
    --------
    
    * :mod:`_origen.logger`
    * :link-to:`Logging Output <logger>`
'''

log = _origen.logger
''' Alias of :data:`logger`
'''

running_on_windows = om.running_on_windows
''' Indicates if Origen is currently running on Windows.

    Returns:
        bool:

    >>> origen.running_on_windows
    False
'''

running_on_linux = om.running_on_linux
''' Indicates if Origen is currently running on Linux.

    Returns:
        bool:

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

    If ``app`` is not ``None``, it should be child of the base class :class:`origen.application.Base`

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

# The application's test program interface, this will be lazily instantiated
# the first time a test program Flow() block is encountered
interface = None

# These vars are used to identify when a target load is taking place
_target_loading = False

producer = Producer()
''' Pointer to the global producer object, :py:class:`origen.producer.Producer`
'''

mode = "development"

_plugins = {}
''' Dictionary of Origen plugins (instances of :py:class:`origen.application.Application`)
    that have been referenced and loaded.
    It should never be access directly since a plugin not being present in this dict may only
    mean that it hasn't been loaded yet (via an official API) rather than it not existing.
'''

mailer = _origen.utility.mailer._mailer()
''' Accessor to the global :class:`Mailer <_origen.utility.mailer.Mailer>`

See also:
    * :link-to:`Mailers in the guides <origen_utilities:mailer>`
'''

maillists = _origen.utility.mailer.maillists()

sessions = _origen.utility.sessions.OrigenSessions()
''' Accessor to the global :class:`SessionStore <_origen.utility.session_store.SessionStore`

See also:
    * :link-to:`Sessions in the guides <origen_utilities:session_store>`
'''

users = om.users
''' |dict-like| container for current and added :class:`Users <_origen.users.Users>`

Put another way, accessor for global :class:`Users <_origen.users.Users>` object

See also:
    * :link-to:`Users in the guides <origen_utilities:users>`
'''

# TODO document this somehow
# ldaps = _origen.utility.ldap.ldaps()
# ''' |dict-like| container for current and added :class:`Users <_origen.utility.ldap.LDAP>`

# Put another way, accessor for global :class:`LDAPs <_origen.utility.ldap.LDAPs>` object

# See also:
#     * :link-to:`LDAPs in the guides <origen_utilities:ldap>`
# '''

__instantiate_dut_called = False

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
    # Will convert any paths with / to \ on Windows
    path = Path(path)
    log.trace(f"Loading file '{path}'")
    context = {**standard_context(), **locals}
    with open(path) as f:
        code = compile(f.read(), path, 'exec')
        exec(code, globals, context)


def test_ast() -> List[str]:
    ''' Returns a serialized representation of the AST for the current pattern'''
    return pickle.loads(bytes(_origen.test_ast()))


def flow_ast() -> List[str]:
    ''' Returns a serialized representation of the AST for the current test program flow '''
    return pickle.loads(bytes(_origen.flow_ast()))


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


def has_plugin(name):
    '''
        Returns true if an Origen plugin matching the given name is found in the current environment
    '''
    if name in _plugins:
        return True
    else:
        try:
            a = importlib.import_module(f'{name}.application')
            app = a.Application(root=Path(os.path.abspath(
                a.__file__)).parent.parent,
                                name=name)
            _plugins[name] = app
            return True
        except ModuleNotFoundError:
            return False


def plugin(name):
    '''
        Returns an :class:`Origen application <origen.application.Application>` instance representing
        the given Origen plugin. None is returned if no plugin is found matching the given name within the
        current environment.
    '''
    if has_plugin(name):
        return _plugins[name]
    else:
        raise RuntimeError(
            f"The current Python environment does not contain a plugin named '{name}'"
        )


def __interactive_context__():
    ''' Returns the local context passed to an interactive section ``origen i`` is run.
    '''
    from origen_metal._helpers import interactive
    from origen.registers.actions import write, verify, write_transaction, verify_transaction
    context = {
        "origen": origen,
        "dut": dut,
        "tester": tester,
        "write": write,
        "verify": verify,
        "write_transaction": write_transaction,
        "verify_transaction": verify_transaction
    }
    context.update(interactive.metal_context())
    return context


__all__ = [
    *internal_members(sys.modules[__name__]), 'config', 'status', 'root',
    'version', 'logger', 'log', 'running_on_windows', 'running_on_linux',
    'frontend_root', 'app', 'dut', 'tester', 'producer', 'has_plugin',
    'plugin', 'current_user', 'users', 'mailer'
]


def __getattr__(name: str):
    if name == "ldaps":
        return _origen.utility.ldaps()
    elif name == "current_user":
        return users.current_user
    elif name == "initial_user":
        return users.initial_user
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
