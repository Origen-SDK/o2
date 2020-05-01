# see the following docs
# https://docs.python.org/3/library/subprocess.html#subprocess.Popen
# https://docs.python.org/3/library/subprocess.html#subprocess.call
# https://docs.python.org/3/library/subprocess.html#subprocess.run

# This is just a starter/proof of concept example, will be cleaned up and streamlined
#
# These tests may not be what you really want to test
#
# Makes a lot of sense to run tests that require a Python workspace here
# rather than as Rust cli tests (Rust tests would need to ensure a working app
# is available before running).
#
# The flow appears to be working, had my doubts
# poetry run pytest -> subprocess -> origen cli -> poetry -> Python run time
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


import pytest
import subprocess
import os

origen_cli = os.getenv('TRAVIS_ORIGEN_CLI') or 'origen'

def test_origen_v():                
  process = subprocess.Popen([f'{origen_cli}', '-v'], stdout=subprocess.PIPE, universal_newlines=True)

  # wait for the process to finish and read the result
  while True:
    return_code = process.poll()
    if return_code is not None:
      # Process is done
      # Read std out
      first_stdout_line = process.stdout.readline()
      assert "Origen" in first_stdout_line
      assert " 2." in first_stdout_line
      assert return_code == 0
      break

def test_bad_command():
  process = subprocess.Popen([f'{origen_cli}', 'thisisnotacommand'], stderr=subprocess.PIPE, universal_newlines=True)

  # wait for the process to finish and read the result
  while True:
    return_code = process.poll()
    if return_code is not None:
      # Process is done
      # Read std out
      first_stderr_line = process.stderr.readline()
      assert "error:" in first_stderr_line
      assert return_code == 1
      break

def test_origen_g():
  process = subprocess.Popen([f'{origen_cli}', 'g', r'.\example\patterns\toggle.py', '-t', r'.\targets\dut\eagle.py'], universal_newlines=True)
  # wait for completion and get the outputs
  return_code = process.wait()
  assert return_code == 0
  