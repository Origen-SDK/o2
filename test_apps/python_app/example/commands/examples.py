import origen
import _origen
from origen.boot import run_cmd
import os


def run(**kwargs):
    os.chdir(origen.root)
    run_cmd("generate",
            files=["example/patterns"],
            reference_dir="approved",
            targets=["dut/eagle", "tester/v93k_smt7", "tester/j750"])

    run_cmd("generate",
            files=["example/flows/o1_testcases/prb1.py"],
            reference_dir="approved",
            targets=["dut/o1_dut", "tester/v93k_smt7"])

    stats = origen.tester.stats()

    changes = stats['changed_pattern_files'] > 0 or stats[
        'changed_program_files'] > 0
    new_files = stats['new_pattern_files'] > 0 or stats['new_program_files'] > 0

    if changes or new_files:
        _origen.exit_fail()
    else:
        _origen.exit_pass()
