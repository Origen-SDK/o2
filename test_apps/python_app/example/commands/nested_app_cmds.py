from test_apps_shared_test_helpers.aux_cmds import run as output_exts

def say_hi(lvl):
    import os
    if os.environ.get("ORIGEN_APP_EXT_NESTED") == "1":
        output_exts()
    else:
        print(f"Hi from 'nested_app_cmds' level {lvl}!")

def run(**args):
    say_hi(0)