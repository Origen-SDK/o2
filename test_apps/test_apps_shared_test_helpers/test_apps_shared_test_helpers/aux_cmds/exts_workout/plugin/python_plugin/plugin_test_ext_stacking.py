# FOR_PR cleanup and conform to other method

from test_apps_shared_test_helpers.cli import after_cmd_ext_args_str, before_cmd_ext_args_str, clean_up_ext_args_str
from origen.boot import before_cmd, after_cmd, clean_up


import origen
def do_action(actions, phase):
    if actions:
        for action in actions:
            print(f"Start Action {phase} CMD: {action}")
            if action == "show_cmd_args":
                print(origen.current_command.args)
            if action == "show_ext_args":
                print({ext_name: ext.args for ext_name, ext in origen.current_command.exts.items()})
                    # print(f"{ext_name} args: {ext.args}")
                # print(origen.current_command.exts["exts_workout"].args)
            if action == "update_cmd_args":
                if phase == "Before":
                    origen.command.args["single_arg"] = "updated"
            if action == "clear_cmd_args":
                origen.command.args["single_val"] = None
            if action == "before_cmd_exception":
                raise RuntimeError("'before_cmd_exception' encountered!")
            if action == "update_ext_workout_args":
                if phase == "Before":
                    args = origen.current_command.exts["aux.exts_workout"].args
                    # Increment the counter
                    args["flag_extension"] += 1

                    # Append to a multi-arg
                    args["multi_val_opt"].append("update_mv_opt")

                    # Overwrite an arg
                    args["single_val_opt"] = "update_sv_opt"

                    # Set a new arg
                    args["new_arg"] = "new_arg_for_ext"
            if action == "update_flag_opts":
                if phase == "Before":
                    origen.current_command.args["flag_opt"] += 1
                    # origen.current_command.exts["aux.exts_workout"].args["flag_extension"] += 1
                    origen.current_command.exts["aux.pl_ext_stacking_from_aux"].args["pl_ext_stacking_flag"] += 1
                    origen.current_command.exts["plugin.python_plugin_the_second"].args["pl_the_2nd_ext_flag"] += 1

            if action == "show_exts":
                fail
            if action == "update_aux_ext":
                fail
            if action == "current_command_BIST":
                fail
            print(f"End Action {phase} CMD: {action}")

action_opt = "ext_action"

@before_cmd
def before(**args):
    print(before_cmd_ext_args_str(args))
    action = args.get(action_opt, None)
    if action:
        do_action(action, "Before")

@after_cmd
def after(**args):
    print(after_cmd_ext_args_str(args))
    action = args.get(action_opt, None)
    if action:
        do_action(action, "After")

@clean_up
def clean_up(**args):
    print(clean_up_ext_args_str(args))
    action = args.get(action_opt, None)
    if action:
        do_action(action, "CleanUp")
