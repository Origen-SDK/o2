# FOR_PR need to clean up print statemnents
import pathlib
from builtins import exit as exit_proc

# TEST_NEEDED
# Or needed at all?
def unsupported_command(base_cmd, sub_command=None):
    if sub_command is None:
        raise RuntimeError(f"Unsupported command '{base_cmd}'")
    else:
        raise RuntimeError(f"Unsupported sub-command '{sub_command}' for '{base_cmd}'")

dispatch_plugin_cmd = "_plugin_dispatch_"
dispatch_aux_cmd = "_dispatch_aux_cmd_"
dispatch_app_cmd = "_dispatch_app_cmd_"

exit = True

def run_cmd(command,
            targets=None,
            verbosity=None,
            verbosity_keywords=None,
            mode=None,
            files=None,
            output_dir=None,
            reference_dir=None,
            debug=False,
            args=None,
            ext_args=None,
            extensions=None,
            dispatch_root=None,
            dispatch_src=None,
            plugins=None,
            subcmds=None,
            exit=None,
            **kwargs):
    ''' Run an Origen command. This is the main entry method for the CLI, but it can also
        be used in application commands to invoke Origen commands within the same thread instead of
        making system calls.
        
        See Also
        --------
        * :link-to:`Example Application Commands <src_code:example_commands>`
    '''

    import origen
    import _origen
    import origen_metal

    import origen.application
    import origen.target

    if command == dispatch_plugin_cmd:
        cmd_src = "plugin"
    elif command == dispatch_aux_cmd:
        cmd_src = "aux_ns"
    elif command == dispatch_app_cmd:
        cmd_src = "app"
    else:
        cmd_src = "core"

    def wrap_mod_from_file(path):
        try:
            return origen.helpers.mod_from_file(path)
        except Exception as e:
            return [path, e]

    def mod_from_modulized_path(root, sub_parts):
        root = pathlib.Path(root)
        if not root.exists():
            return [f"Root directory '{root}' does not exists or is not accessible"]
        path = pathlib.Path(f"{root.joinpath('.'.join(sub_parts))}.py")
        if not path.exists():
            paths = [path]
            modulized_path = pathlib.Path(root)
            if len(sub_parts) > 1:
                for i, sub in enumerate(sub_parts[:-1]):
                    modulized_path = modulized_path.joinpath(sub)
                    if modulized_path.exists():
                        path = pathlib.Path(f"{modulized_path}/{'.'.join(sub_parts[(i+1):])}.py")
                        if path.exists():
                            return wrap_mod_from_file(path)
                        else:
                            paths.append(path)
                    else:
                        return [f"From root '{root}', searched:", *[p.relative_to(root) for p in paths]]
                return [f"From root '{root}', searched:", *[p.relative_to(root) for p in paths]]
            else:
                path = pathlib.Path(f"{modulized_path}/{sub_parts[-1]}.py")
                if not path.exists():
                    paths.append(path)
                    return [f"From root '{root}', searched:", *[p.relative_to(root) for p in paths]]
        return wrap_mod_from_file(path)

    def call_user_cmd(cmd_type):
        m = mod_from_modulized_path(dispatch_root, subcmds)

        if isinstance(m, list):
            if isinstance(m[1], Exception):
                origen.log.error(f"Could not load {cmd_type} command implementation from '{('.').join(subcmds)}' ({m[0]})")
                origen.log.error(f"Received exception:\n{m[1]}")
            else:
                origen.log.error(f"Could not find implementation for {cmd_type} command '{('.').join(subcmds)}'")
                for msg in m:
                    origen.log.error(f"  {msg}")
            exit_proc(1)

        if hasattr(m, 'run'):
            m.run(**(args or {}))
        else:
            origen.logger.error(f"Could not find 'run' function in module '{m.__file__}'")
            exit_proc(1)

    def is_subcmd(*subcs):
        return list(subcs) == subcmds

    def unsupported_subcmd(subcmd=None):
        if subcmd is None:
            print(f"Unsupported sub-command '{subcmds.join(' -> ')}' for base command '{command}'")
        else:
            print(f"Unsupported sub-command '{subcmd}' for '{command}'")
        exit_proc(1)

    if mode == None:
        if _origen.is_app_present():
            origen.set_mode(_origen.app_config()["mode"])
    else:
        origen.set_mode(mode)

    if files is not None:
        _origen.file_handler().init(files)

    if verbosity is not None:
        origen_metal.framework.logger.set_verbosity(verbosity)

    if verbosity_keywords is not None:
        origen_metal.framework.logger.set_verbosity_keywords(verbosity_keywords)

    if output_dir is not None:
        _origen.set_output_dir(output_dir)

    if reference_dir is not None:
        _origen.set_reference_dir(reference_dir)

    if debug:
        _origen.enable_debug()

    from origen.core.plugins import from_origen_cli
    from_origen_cli(plugins)

    if _origen.is_app_present():
        origen.target.setup(targets=([] if targets is False else targets))

    if args is None:
        args = {}
    if subcmds is None:
        subcmds = []
    if ext_args is None:
        ext_args = {}
    if extensions is None:
        extensions = []
    current_ext = None

    def before_cmd(func):
        current_ext["before_cmd"] = func.__name__
        return func
    setattr(origen.boot, "before_cmd", before_cmd)

    def after_cmd(func):
        current_ext["after_cmd"] = func.__name__
        return func
    setattr(origen.boot, "after_cmd", after_cmd)

    def clean_up(func):
        current_ext["clean_up"] = func.__name__
        return func
    setattr(origen.boot, "clean_up", clean_up)

    def on_load(func):
        if current_ext:
            current_ext["on_load"] = func.__name__
        else:
            dispatch['on_load'] = func
        return func
    setattr(origen.boot, "on_load", on_load)

    dispatch = {}
    def run(func):
        dispatch['run_func'] = func
        return func
    setattr(origen.boot, "run", run)

    # FOR_PR need to test with app subcmds

    for ext in extensions:
        current_ext = ext
        if cmd_src == "core":
            _dispatch_src = [command]
        elif cmd_src == "app":
            _dispatch_src = []
        else:
            _dispatch_src = [dispatch_src]
        m = mod_from_modulized_path(ext['root'], [cmd_src, *_dispatch_src, *subcmds])
        if isinstance(m, list):
            if len(m) == 2 and isinstance(m[1], Exception):
                origen.log.error(f"Could not load {ext['source']} extension implementation from '{ext['name']}' ({m[0]})")
                origen.log.error(f"Received exception:\n{m[1]}")
            else:
                if ext['source'] == "app":
                    n = ''
                else:
                    n = f"'{ext['name']}'"
                origen.log.error(f"Could not find implementation for {ext['source']} extension{n}")
                for msg in m:
                    origen.log.error(f"  {msg}")
            ext['mod'] = None
        else:
            ext['mod'] = m
        
            if "on_load" in ext:
                getattr((ext["mod"]), ext["on_load"])(ext["mod"])
    current_ext = None
    _origen.current_command.set_command(command, subcmds, args, ext_args, extensions)

    def run_ext(phase, continue_on_fail=False):
        for ext in extensions:
            if phase in ext:
                if ext['source'] == "app":
                    this_ext_args = ext_args["app"]
                else:
                    this_ext_args = ext_args[ext['source']][ext['name']]

                try:
                    getattr(ext["mod"], ext[phase])(**this_ext_args)
                except Exception as e:
                    if continue_on_fail:
                        origen.log.error(f"Error running {ext['source']} extension{'' if ext['source'] == 'app' else ' ' + ext['name']}")
                        origen.log.error(e)
                    else:
                        raise(e)

    try:
        run_ext("before_cmd")

        # The generate command handles patterns and flows.
        # Future: Add options to generate patterns concurrently, or send them off to LSF.
        # For now, just looping over the patterns.
        if command == "generate":
            origen.producer.generate(*[f for f in _origen.file_handler()])

            # Alway print a summary when initiated from the CLI
            origen.producer.summarize()

        elif command == "compile":
            _origen.set_operation("compile")
            for file in _origen.file_handler():
                origen.app.compile(pathlib.Path(file))

        elif command == "interactive":
            _origen.set_operation("interactive")
            origen.logger.trace("Starting interactive session (on Python side)")
            if _origen.is_app_present:
                origen.target.load()

            from origen_metal._helpers import interactive
            from origen import dut, tester
            from origen.registers.actions import write, verify, write_transaction, verify_transaction
            interactive.prep_shell(origen.__console_history_file__)
            interactive.interact(banner=f"Origen {origen.version}",
                                context=origen.__interactive_context__())

        elif command == "credentials":
            _origen.set_operation("credentials")
            datasets = args.get("datasets", None)
            all = args.get("all", False)
            if is_subcmd("set"):
                if all:
                    origen.logger.display("Setting passwords for all available datasets...")
                    for d in origen.current_user.datasets.values():
                        d.password = None
                        d.password()
                    origen.logger.display("Done!")
                elif datasets is None:
                    origen.current_user.password = None
                    origen.current_user.password
                else:
                    for d in datasets:
                        d.password = None
                        d.password()
            elif is_subcmd("clear"):
                if all:
                    origen.logger.display("Clearing all cached passwords...")
                    origen.current_user.clear_cached_passwords()
                elif datasets is None:
                    origen.logger.display("Clearing cached password for topmost dataset...")
                    origen.current_user.clear_cached_password()
                else:
                    for d in datasets:
                        origen.logger.display(f"Clearing cached password for dataset '{d}'")
                        origen.current_user.datasets[d].clear_cached_password()
                origen.logger.display("Done!")
            else:
                unsupported_subcmd()

        elif command == "eval":
            for c in args['code']:
                exec(c)

        elif command == "web:build":
            _origen.set_operation("web")
            from origen.web import run_cmd
            return run_cmd("build", args)

        elif command == "web:view":
            _origen.set_operation("web")
            from origen.web import run_cmd
            return run_cmd("view", args)

        elif command == "web:clean":
            _origen.set_operation("web")
            from origen.web import run_cmd
            return run_cmd("clean", args)

        elif command == "app:publish":
            _origen.set_operation("app")
            origen.app.__publish__(**args).summarize_and_exit()

        elif command == "app:package":
            _origen.set_operation("app")
            origen.app.build_package(args)

        elif command == "app:run_publish_checks":
            _origen.set_operation("app")
            origen.app.__run_publish_checks__(args).summarize_and_exit()

        elif command == "app:init":
            _origen.set_operation("app")
            r = origen.app.__rc_init__()
            r.summarize_and_exit()

        elif command == "app:status":
            _origen.set_operation("app")
            r = origen.app.__rc_status__()
            r.summarize()

        elif command == "app:checkin":
            _origen.set_operation("app")
            checkin_all = args.pop("all", False)
            args["dry_run"] = args.pop("dry-run", False)
            if 'pathspecs' in args and not checkin_all:
                r = origen.app.__rc_checkin__(**args)
            else:
                r = origen.app.__rc_checkin__(pathspecs=None, **args)
            r.gist()

        # TODO need to remove generic result
        elif command == "mailer:test":
            if origen.mailer is None:
                from origen_metal.framework import Outcome
                r = Outcome(succeeded=False, message="No mailer available!")
            else:
                r = origen.app.mailer.test(args.get("to", None))
            r.summarize_and_exit()

        # TODO need to remove generic result
        elif command == "mailer:send":
            if origen.mailer is None:
                from origen_metal.framework import Outcome
                r = Outcome(succeeded=False, message="No mailer available!")
            else:
                r = origen.app.mailer.send(subject=args.get("subject", None),
                                        to=args.get("to", None),
                                        body=args["body"])
            r.summarize_and_exit()

        # Internal command to give the Origen version loaded by the application to the CLI
        elif command == "_version_":
            import importlib_metadata

            def tabify(message):
                return "\n".join([f"\t{l}" for l in message.split("\n")])

            try:
                if origen.app:
                    print(f"App\nSuccess\n{tabify(origen.app.version)}")
            except Exception as e:
                print("App")
                print("Error")
                print(tabify(repr(e)))

            if origen.__in_origen_core_app:
                origen.logger.info("Running in Origen core application")
            else:
                print("Origen")
                try:
                    print(
                        f"Success\n{tabify(importlib_metadata.version('origen'))}")
                except Exception as e:
                    print("Error")
                    print(tabify(repr(e)))

            print("_ CLI")
            try:
                print(f"Success\n{tabify(origen.status['cli_version'])}")
            except Exception as e:
                print("Error")
                print(tabify(repr(e)))

            print("_ PyAPI")
            try:
                print(
                    f"Success\n{tabify(origen.status['other_build_info']['pyapi_version'])}"
                )
            except Exception as e:
                print("Error")
                print(tabify(repr(e)))

            print("_ Origen (Rust Backend)")
            try:
                print(f"Success\n{tabify(origen.status['origen_version'])}")
            except Exception as e:
                print("Error")
                print(tabify(repr(e)))

            print("_ Origen-Core-Support")
            try:
                print(
                    f"Success\n{tabify(origen.status['origen_core_support_version'])}"
                )
            except Exception as e:
                print("Error")
                print(tabify(repr(e)))

            print("_ OrigenMetal (Rust Backend - Origen)")
            try:
                print(
                    f"Success\n{tabify(origen.status['origen_metal_backend_version'])}"
                )
            except Exception as e:
                print("Error")
                print(tabify(repr(e)))

            print("_ origen_metal")
            try:
                print(
                    f"Success\n{tabify(importlib_metadata.version('origen_metal'))}"
                )
            except Exception as e:
                print("Error")
                print(tabify(repr(e)))

            print("_ _origen_metal (PyAPI Metal)")
            try:
                print(f"Success\n{tabify(origen_metal._origen_metal.__version__)}")
            except Exception as e:
                print("Error")
                print(tabify(repr(e)))

            print("_ OrigenMetal (Rust Backend - PyAPI Metal)")
            try:
                print(
                    f"Success\n{tabify(origen_metal._origen_metal.__origen_metal_backend_version__)}"
                )
            except Exception as e:
                print("Error")
                print(tabify(repr(e)))


        elif command == dispatch_app_cmd:
            call_user_cmd("app")

        elif command == dispatch_plugin_cmd:
            call_user_cmd("plugin")

        elif command == dispatch_aux_cmd:
            call_user_cmd("aux")

        else:
            unsupported_command(command)

        run_ext("after_cmd")
    finally:
        run_ext("clean_up", continue_on_fail=True)

    if exit is None:
        if origen.boot.exit:
            exit_proc(0)
    elif exit is False:
        pass
    else:
        exit_proc(0)
