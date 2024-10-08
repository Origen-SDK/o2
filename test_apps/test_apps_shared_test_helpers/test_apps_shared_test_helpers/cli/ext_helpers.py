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
            if phase is None:
                p_out = 'For'
            else:
                p_out = phase
            print(f"Start Action {p_out} CMD: {action}")
            if action.startswith("inc_flag__"):
                if phase == "Before":
                    _, is_cmd, f = action.split("__", 2)
                    if is_cmd == "cmd":
                        args = origen.current_command.args
                    else:
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
            elif action == "display_current_command":
                cc = origen.current_command
                print(f"Class: {cc.__class__.__name__}")
                cs = cc.source
                print(f"Command Src Class: {cs.__class__.__name__}")
                st = cs.source_type
                print(f"Src Type Class: {st.__class__.__name__}")
                print(f"Src Plugin Class: {cs.plugin.__class__.__name__}")
                print(f"Base Cmd: {cc.base_cmd}")
                print(f"Sub Cmds: {cc.subcmds}")
                print(f"Args: {cc.args}")
                print(f"Arg Indices: {cc.arg_indices}")
                print(f"Exts: {dict(cc.exts)}")
                print(f"Src Path: {cs.path}")
                print(f"Src Plugin: {cs.plugin.name}")
                print(f"Src Type: {st}")
                print(f"Src is_core_cmd: {st.is_core_cmd}")
                print(f"Src is_plugin_cmd: {st.is_plugin_cmd}")
                print(f"Src is_aux_cmd: {st.is_aux_cmd}")
                print(f"Src is_app_cmd: {st.is_app_cmd}")
                print(f"Src root name: {st.root_name}")
            elif action == "show_ext_mods":
                # TEST_NEEDED CLI check for extension mods
                for n, e in origen.current_command.exts.items():
                    print(f"{n}: {e.mod}")
            elif action == "show_arg_indices":
                print(origen.current_command.arg_indices)
            elif action == "show_ext_arg_indices":
                print({ n: v.arg_indices for n, v in origen.current_command.exts.items() })
            elif action == "no_action":
                pass
            else:
                raise RuntimeError(f"No action '{action}' is known!")
            print(f"End Action {p_out} CMD: {action}")

def get_action_results(output, actions):
    retn = {}
    for action in actions:
        a = {}
        r = output.split(f"Start Action Before CMD: {action}")[1].strip()
        a["Before"], r = r.split(f"End Action Before CMD: {action}")
        r = output.split(f"Start Action After CMD: {action}")[1].strip()
        a["After"], r = r.split(f"End Action After CMD: {action}")
        retn[action] = a
    return retn
