# FOR_PR switch to common method. Use nested_common as example for using common functions when running
from test_apps_shared_test_helpers.cli import output_args

def run(**args):
    if len(args) > 0:
        print(output_args(None, args))
    else:
        print("No args or opts given!")

# def run(**args):
#     print(f"Arg Keys: {list(args.keys())}")
#     if len(args) > 0:
#         for n, arg in args.items():
#             print(f"Arg: {n} ({arg.__class__}): {arg}")
#     else:
#         print("No args or opts given!")
