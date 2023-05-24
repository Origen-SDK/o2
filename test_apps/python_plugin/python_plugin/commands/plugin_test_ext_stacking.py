from test_apps_shared_test_helpers.aux_cmds import run as tas_run
from origen.boot import run as run_wrapper

def run(**args):
    raise RuntimeError("This shouldn't be used!")

@run_wrapper
def run_func(**args):
    tas_run(**args)