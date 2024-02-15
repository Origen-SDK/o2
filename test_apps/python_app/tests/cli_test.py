import pytest, pathlib, sys
import subprocess
import os
import origen

pytest.register_assert_rewrite("t_invocation_env")

sys.path.insert(-1, str(pathlib.Path(__file__).parent.parent.parent.joinpath("no_workspace")))
from t_invocation_env import T_InvocationBaseTests

from cli.tests__app_cmd_building import T_AppCmdBuilding
from cli.tests__core_cmds import T_AppWorkspaceCoreCommands
from cli.tests__cmd_exts_from_app import T_ExtendingFromAppCmds
from cli.tests__reserved_opts import T_ReservedOpts
from cli.tests__cmd_integration import T_CommandIntegration
from cli.tests__intra_cmd_conflicts import T_IntraCmdConflicts
from cli.tests__extending_app_cmds import T_ExtendingAppCmds
from cli.tests__non_extendable_err_msgs import T_NonExtendableErrMsgs

class TestAppCmdBuilding(T_AppCmdBuilding):
    pass

class TestExtendingAppCmds(T_ExtendingAppCmds):
    pass

class TestIntraCmdConflicts(T_IntraCmdConflicts):
    pass

class TestExtendingFromAppCommands(T_ExtendingFromAppCmds):
    pass

class TestAppWorkspaceCoreCommands(T_AppWorkspaceCoreCommands):
    pass

class TestReservedOpts(T_ReservedOpts):
    pass

class TestCommandIntegration(T_CommandIntegration):
    pass

class TestNonExtendableErrMsgs(T_NonExtendableErrMsgs):
    pass

origen_cli = os.getenv('TRAVIS_ORIGEN_CLI') or 'origen'

def test_origen_v():
    process = subprocess.Popen([f'{origen_cli}', '-v'],
                               stdout=subprocess.PIPE,
                               universal_newlines=True)
    # wait for the process to finish and read the result, 0 is success
    assert process.wait() == 0
    # Process is done
    # Read std out
    first_stdout_line = process.stdout.readline()
    assert "Origen" in first_stdout_line
    assert " 2." in first_stdout_line
    second_stdout_line = process.stdout.readline()
    assert "App:" in second_stdout_line

def test_bad_command():
    process = subprocess.Popen([f'{origen_cli}', 'thisisnotacommand'],
                               stderr=subprocess.PIPE,
                               universal_newlines=True)
    assert process.wait() == 2
    assert "error:" in process.stderr.readline()

@pytest.mark.skip
class TestAuxCommandsAreAdded:
    ...

@pytest.mark.skip
class TestModeOpts():
    def test_():
        fail

class TestAppInvocation(T_InvocationBaseTests):
    @classmethod
    def set_params(cls):
        cls.invocation = cls.PyProjectSrc.App
        cls.target_pyproj_dir = pathlib.Path(__file__).parent.parent
