import origen
import _origen
from origen.boot import run_cmd
import os


def run(**kwargs):
    debug = "debug" in kwargs
    os.chdir(origen.root)
    origen.boot.exit = False

    run_cmd("generate",
            args={
                "files": ["example/patterns"],
                "reference_dir": "approved",
            },
            debug=debug,
            targets=["dut/eagle", "tester/v93k_smt7", "tester/j750"])

    run_cmd("generate",
            args={
                "files": ["example/flows/o1_testcases/prb1.py", "example/flows/o1_testcases/prb2.py"],
                "reference_dir": "approved",
            },
            debug=debug,
            targets=["dut/o1_dut", "tester/v93k_smt7", "tester/v93k_smt8"])

    stats = origen.tester.stats()

    changes = stats['changed_pattern_files'] > 0 or stats[
        'changed_program_files'] > 0
    new_files = stats['new_pattern_files'] > 0 or stats['new_program_files'] > 0

    if changes or new_files:
        _origen.exit_fail()
    else:
        _origen.exit_pass()
