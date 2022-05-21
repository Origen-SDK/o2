# see the following docs
# https://docs.python.org/3/library/subprocess.html#subprocess.Popen
# https://docs.python.org/3/library/subprocess.html#subprocess.call
# https://docs.python.org/3/library/subprocess.html#subprocess.run

# Makes a lot of sense to run tests that require a Python workspace here
# rather than as Rust cli tests (Rust tests would need to ensure a working app
# is available before running).
#
# Global commands could go here if the working dir is changed first.
# Otherwise Rust cli tests can be used. See the rust/origen/cli/test directory
#
# an interactive command test write to stdin like this:
#   process = subprocess.Popen(['origen', '-v']),
#                stdin=subprocess.PIPE,
#                stdout=subprocess.PIPE,
#                stderr=subprocess.PIPE,
#                universal_newlines=True,
#                bufsize=0)
#  process.stdin.write("yes\n")
#  process.stdin.close()
#  read output, etc

import pytest, pathlib
import subprocess
import os
import origen

origen_cli = os.getenv('TRAVIS_ORIGEN_CLI') or 'origen'


def test_origen_v():
    #import pdb; pdb.set_trace()
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
    assert process.wait() == 1
    assert "error:" in process.stderr.readline()


def test_origen_g():
    os.chdir(origen.root)
    process = subprocess.Popen([
        f'{origen_cli}', 'g', r'./example/patterns/toggle.py', '-t',
        r'./targets/eagle_with_smt7.py'
    ],
                               universal_newlines=True)
    assert process.wait() == 0

class TestBadConfigs:
    @property
    def bad_config_env(self):
        return {
            **os.environ,
            "origen_bypass_config_lookup": "1",
            "origen_config_paths": str(
                pathlib.Path(__file__).parent.joinpath("origen_utilities/configs/ldap/test_bad_ldap_config.toml").absolute()
            )
        }

    def test_origen_v(self):
        r = subprocess.run(
            [origen_cli, '-v'],
            capture_output=True,
            env=self.bad_config_env
        )
        assert r.returncode == 1
        out = r.stdout.decode("utf-8").strip()
        err = r.stderr.decode("utf-8").strip()
        p = pathlib.Path("tests/origen_utilities/configs/ldap/test_bad_ldap_config.toml")
        assert "Couldn't boot app to determine the in-application Origen version" in out
        assert f"invalid type: string \"hi\", expected an integer for key `ldaps.bad.timeout` in {str(p)}" in out
        assert err == ""

    def test_origen_cmd(self):
        r = subprocess.run(
            [origen_cli, 'g', r'./example/patterns/toggle.py', '-t', r'./targets/eagle_with_smt7.py'],
            capture_output=True,
            env=self.bad_config_env
        )
        assert r.returncode == 1
        out = r.stdout.decode("utf-8").strip()
        err = r.stderr.decode("utf-8").strip()
        p = pathlib.Path("tests/origen_utilities/configs/ldap/test_bad_ldap_config.toml")
        assert f"invalid type: string \"hi\", expected an integer for key `ldaps.bad.timeout` in {str(p)}" in out
        assert err == ""

    def test_bad_config_path(self):
        r = subprocess.run(
            [origen_cli, '-v'],
            capture_output=True,
            env={
                **self.bad_config_env,
                **{
                    "origen_config_paths": str(pathlib.Path(__file__).parent.joinpath("missing.toml").absolute())
                }
            }
        )
        assert r.returncode == 1
        out = r.stdout.decode("utf-8").strip()
        err = r.stderr.decode("utf-8").strip()
        assert "Couldn't boot app to determine the in-application Origen version" in out
        assert "missing.toml either does not exists or is not accessible" in out
        assert err == ""
