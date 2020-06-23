import origen
import _origen
from origen.boot import run_cmd
import os

def run(**kwargs):
    os.chdir(origen.root)
    run_cmd("generate", files=["example/patterns"], reference_dir="approved")

    stats = origen.tester.stats()
    changes = stats['changed_pattern_files'] > 0 or stats['changed_program_files'] > 0
    new_files = stats['new_pattern_files'] > 0 or stats['new_program_files'] > 0

    if changes or new_files:
        _origen.exit_fail()
    else:
        _origen.exit_pass()
