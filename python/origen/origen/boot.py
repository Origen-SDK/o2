# These must come before all other imports.
from __future__ import print_function, unicode_literals, absolute_import

import pathlib
import importlib


def run_cmd(command,
            targets=None,
            verbosity=None,
            verbosity_keywords="",
            mode=None,
            files=None,
            output_dir=None,
            reference_dir=None,
            debug=False,
            args=None,
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

    if mode == None:
        origen.set_mode(_origen.app_config()["mode"])
    else:
        origen.set_mode(mode)

    if files is not None:
        _origen.file_handler().init(files)

    if verbosity is not None:
        origen_metal.framework.logger.set_verbosity(verbosity)
        origen_metal.framework.logger.set_verbosity_keywords(verbosity_keywords.split(","))

    if output_dir is not None:
        _origen.set_output_dir(output_dir)

    if reference_dir is not None:
        _origen.set_reference_dir(reference_dir)

    if debug:
        _origen.enable_debug()

    origen.target.setup(targets=targets)

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
        origen.target.load()

        from origen_metal._helpers import interactive
        from origen import dut, tester
        from origen.registers.actions import write, verify, write_transaction, verify_transaction
        interactive.prep_shell(origen.__console_history_file__)
        interactive.interact(banner=f"Origen {origen.version}",
                             context=origen.__interactive_context__())

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
            r = origen.utility.results.GenericResult(
                succeeded=False, message="No mailer available!")
        else:
            r = origen.app.mailer.test(args.get("to", None))
        r.summarize_and_exit()

    # TODO need to remove generic result
    elif command == "mailer:send":
        if origen.mailer is None:
            r = origen.utility.results.GenericResult(
                succeeded=False, message="No mailer available!")
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

    # Internal command to dispatch an app/plugin command
    elif command == "_dispatch_":
        path = f'{origen.app.name}.commands'
        for cmd in kwargs["commands"]:
            path += f'.{cmd}'
        m = importlib.import_module(path)
        m.run(**(args or {}))
        exit(0)

    else:
        print(f"Unknown command: {command}")
        exit(1)
