import origen

def extract_args_flag(action):
    _, is_cmd, f = action.split("__", 2)
    if is_cmd == "cmd":
        args = origen.current_command.args
    else:
        print("Exts:")
        print(origen.current_command.exts.keys())
        ext = f.rsplit("_flag", 1)[0]
        t = is_cmd.split("_")[0]
        args = origen.current_command.exts[f"{t}.{ext}"].args
    return (args, f)

def extract(action):
    a, is_cmd, rest = action.split("__", 2)
    if is_cmd == "cmd":
        args = origen.current_command.args
        (name, rest) = rest.split("__", 1)
        params = rest.split("__")
    else:
        t = is_cmd.split("_")[0]
        (ext, rest) = rest.split("__", 1)
        args = origen.current_command.exts[f"{t}.{ext}"].args
        (name, rest) = rest.split("__", 1)
        name = f"{ext}_{name}"
        params = rest.split("__")
    return (a, args, name, params)

def do_action(actions, phase):
    if actions:
        for action in actions:
            print(f"Start Action {phase} CMD: {action}")
            if action.startswith("inc_flag__"):
                if phase == "Before":
                    _, is_cmd, f = action.split("__", 2)
                    if is_cmd == "cmd":
                        args = origen.current_command.args
                    else:
                        print("Exts:")
                        print(origen.current_command.exts.keys())
                        ext = f.rsplit("_flag", 1)[0]
                        t = is_cmd.split("_")[0]
                        args = origen.current_command.exts[f"{t}.{ext}"].args
                    args[f] += 1
            elif action.startswith("set_flag"):
                if phase == "Before":
                    args, f = extract_args_flag(action)
                    args[f] = -1
            elif action.startswith("inc_multi_arg"):
                if phase == "Before":
                    (_, args, name, params) = extract(action)
                    args[name] = params
            elif action.startswith("set_arg__"):
                if phase == "Before":
                    _, args, arg, vals = extract(action)
                    args[arg] = vals[0]
            elif action == "show_cmd_args":
                print(origen.current_command.args)
            elif action == "show_ext_args":
                print({ext_name: ext.args for ext_name, ext in origen.current_command.exts.items()})
            elif action == "exts_workout__test_updating_args":
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
            elif action == "no_action":
                pass
            else:
                raise RuntimeError(f"No action '{action}' is known!")
            print(f"End Action {phase} CMD: {action}")