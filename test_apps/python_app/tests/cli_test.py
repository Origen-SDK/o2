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

import pytest
import subprocess
import os
import origen

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
    third_stdout_line = process.stdout.readline()
    assert " 2." in second_stdout_line
    assert " 2." in third_stdout_line


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
        r'./targets/dut/eagle.py'
    ],
                               universal_newlines=True)
    assert process.wait() == 0
