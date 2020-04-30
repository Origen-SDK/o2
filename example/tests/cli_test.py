# see the following docs
# https://docs.python.org/3/library/subprocess.html#subprocess.Popen
# https://docs.python.org/3/library/subprocess.html#subprocess.call
# https://docs.python.org/3/library/subprocess.html#subprocess.run

# This is just a starter example, will be cleaned up and streamlined
#
# These tests may not be what you really want to test
#
# Makes a lot of sense to run tests that require a Python workspace here
# rather than as Rust cli tests (Rust tests would need to ensure a working app
# is available before running).
#
# The flow appears to be working, had my doubts
# poetry run pytest -> subprocess -> origen cli -> Python run time
#
# Global commands could go here if the working dir is changed first.
# Otherwise Rust cli tests can be used. See the following:
# https://rust-cli.github.io/book/tutorial/testing.html#testing-cli-applications-by-running-them
# https://docs.rs/predicates/1.0.4/predicates/
# https://docs.rs/assert_cmd/1.0.1/assert_cmd/
# https://crates.io/crates/rexpect

import pytest
import subprocess

def test_origen_v():
  # process = subprocess.Popen(['origen', '-v']),
                # stdin=subprocess.PIPE,
                # stdout=subprocess.PIPE,
                # stderr=subprocess.PIPE,
                # universal_newlines=True,
                # bufsize=0)
  # process.stdin.write("yes\n")
  # process.stdin.close()
                
  process = subprocess.Popen(['origen', '-v'], stdout=subprocess.PIPE, universal_newlines=True)

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
  process = subprocess.Popen(['origen', 'thisisnotacommand'], stderr=subprocess.PIPE, universal_newlines=True)

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
  process = subprocess.Popen(['origen', 'g', r'.\example\patterns\toggle.py', '-t', r'.\targets\dut\eagle.py'])
  # wait for completion and get the outputs
  return_code = process.wait()
  assert return_code == 0