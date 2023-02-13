# FOR_PR convert assert args
import pytest, pathlib
import subprocess
import os
import origen

from cli.tests__app_cmd_building import T_AppCmdBuilding
from cli.tests__core_cmds import T_AppWorkspaceCoreCommands
from cli.tests__cmd_exts_from_app import T_ExtendingFromAppCmds
from cli.tests__reserved_opts import T_ReservedOpts

class TestAppCmdBuilding(T_AppCmdBuilding):
    pass

class TestExtendingFromAppCommands(T_ExtendingFromAppCmds):
    pass

class TestAppWorkspaceCoreCommands(T_AppWorkspaceCoreCommands):
    pass

class TestReservedOpts(T_ReservedOpts):
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
    assert "App:" in first_stdout_line
    second_stdout_line = process.stdout.readline()
    assert "Origen" in second_stdout_line
    assert " 2." in second_stdout_line

def test_bad_command():
    process = subprocess.Popen([f'{origen_cli}', 'thisisnotacommand'],
                               stderr=subprocess.PIPE,
                               universal_newlines=True)
    assert process.wait() == 2
    assert "error:" in process.stderr.readline()


def test_origen_g():
    os.chdir(origen.root)
    process = subprocess.Popen([
        f'{origen_cli}', 'g', r'./example/patterns/toggle.py', '-t',
        r'./targets/eagle_with_smt7.py'
    ],
                               universal_newlines=True)
    assert process.wait() == 0

@pytest.mark.skip
class TestPluginCommandsAreAdded:
    ...

@pytest.mark.skip
class TestAuxCommandsAreAdded:
    ...

@pytest.mark.skip
class TestAppPluginAndAuxCommandClashing:
    def test_app_cmd_overrides_pl_cmd(self):
        fail

    def test_app_cmd_overrides_aux_cmd(self):
        fail

    # def test_extending_app_cmds(self):
    #     fail


@pytest.mark.skip
class TestModeOpts():
    def test_():
        fail

@pytest.mark.skip
class PluginCmdsInApp():
    def test_app_opts_are_added_by_default(self):
        fail
    
    def test_disabling_app_opts(self):
        fail
    
    def test_disabling_app_opts_individually(self):
        fail

@pytest.mark.skip
class AuxCmdsInApp():
    def test_app_opts_are_added_by_default(self):
        fail
    
    def test_disabling_app_opts(self):
        fail
    
    def test_disabling_app_opts_individually(self):
        fail
