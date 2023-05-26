from test_apps_shared_test_helpers.cli import apply_ext_output_args
from origen.boot import on_load

@on_load
def load_ext(mod):
    apply_ext_output_args(mod)
