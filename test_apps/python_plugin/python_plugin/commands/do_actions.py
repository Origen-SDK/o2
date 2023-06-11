from test_apps_shared_test_helpers.cli.ext_helpers import do_action

def run(**args):
    do_action(args.get("actions"), None)