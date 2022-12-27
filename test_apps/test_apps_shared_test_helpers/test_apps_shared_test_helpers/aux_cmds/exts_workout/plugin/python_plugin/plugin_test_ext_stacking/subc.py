# FOR_PR switch to on-load
from test_apps_shared_test_helpers.cli import after_cmd_ext_args_str, before_cmd_ext_args_str, clean_up_ext_args_str
from origen.boot import before_cmd, after_cmd, clean_up


@before_cmd
def before(**args):
    print(before_cmd_ext_args_str(args))

@after_cmd
def after(**args):
    print(after_cmd_ext_args_str(args))

@clean_up
def clean_up(**args):
    print(clean_up_ext_args_str(args))
