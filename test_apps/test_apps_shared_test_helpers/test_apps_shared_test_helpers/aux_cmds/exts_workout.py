import origen
from test_apps_shared_test_helpers.cli import after_cmd_ext_args_str
from test_apps_shared_test_helpers.cli import before_cmd_ext_args_str
from test_apps_shared_test_helpers.cli import clean_up_ext_args_str
# from test_apps_shared_test_helpers.cli import ExtensionDrivers
# def ext_out()
from origen.boot import before_cmd, after_cmd, clean_up

req_opt = "action_opt"

class aux:
    class exts_workout:
        class plugin_test_args:
            @classmethod
            def do_action(cls, actions, phase):
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
                        if action == "show_exts":
                            fail
                        if action == "update_aux_ext":
                            fail
                        if action == "current_command_BIST":
                            fail
                        print(f"End Action {phase} CMD: {action}")

            @before_cmd
            @classmethod
            def before_cmd(cls, **args):
                print(before_cmd_ext_args_str(args))
                # if args[ext_req.name] == "update_args":
                #     ...
                action = args.get(req_opt, None)
                if action:
                    cls.do_action(action, "Before")

            #     # origen.current_command.args['input'] = ["hijack!"]
            #     print("before!!")
            #     args.pop("flag_extension")
            #     return args

            @classmethod
            def after_cmd(cls, **args):
                print(after_cmd_ext_args_str(args))
                action = args.get(req_opt, None)
                if action:
                    cls.do_action(action, "After")

            @classmethod
            def clean_up(cls, **args):
                print(clean_up_ext_args_str(args))
                action = args.get(req_opt, None)
                if action:
                    cls.do_action(action, "CleanUp")

            class subc:
                @classmethod
                def before_cmd(**args):
                    print(before_cmd_ext_args_str(args))

        class plugin_test_ext_stacking:
            pass
            # @before_cmd
            # @classmethod
            # def print_args_before(cls, **args):
            #     print(before_cmd_ext_args_str(args))

            # @after_cmd
            # @classmethod
            # def print_args_after(cls, **args):
            #     print(after_cmd_ext_args_str(args))

            # @clean_up
            # @classmethod
            # def print_args_clean(cls, **args):
            #     print(clean_up_ext_args_str(args))
