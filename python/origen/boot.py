# Called by the Origen CLI to boot the Origen Python env, not for application use
# Any target/env overrides given to the command line will be passed in here
def __origen__(command, target=None, environment=None, mode=None):
    import origen
    import _origen
    import origen.application
    import origen.target

    if mode == None:
        origen.set_mode(_origen.app_config()["mode"])
    else:
        origen.set_mode(mode)

    origen.target.load(target=target, environment=environment)

    if command == "generate":
        print("Generate command called!")

    elif command == "interactive":
        import atexit, os, rlcompleter, sys, colorama, termcolor
        if sys.platform == "win32":
            import pyreadline as readline
        else:
            import readline

        # Colorama init only required on windows, but place it here to keep consistent with all platforms, or in case options
        # need to be added
        # Also, its a known issue that powershell doesn't display yellow text correctly. The standard command prompt will
        # though.
        colorama.init()
        historyPath = origen.root.joinpath(".origen").joinpath("console_history")

        def save_history(historyPath=historyPath):
            import sys, colorama, termcolor
            colorama.init()
            if sys.platform == "win32":
                import pyreadline as readline # This isn't necssary but left it in, in case we want to experiment with it.
                print(termcolor.colored("Origen: Warning: origen currently does not support history files on Windows. :(", 'yellow'))
            else:
                import readline
                readline.write_history_file(historyPath)

        if os.path.exists(historyPath):
            if sys.platform == "win32":
                print(termcolor.colored("Origen: Warning: origen currently does not support history files on Windows. :(", 'yellow'))
            else:
                readline.read_history_file(historyPath)

        atexit.register(save_history)
        del os, atexit, readline, rlcompleter, sys, colorama, termcolor, save_history, historyPath

        import code
        from origen import dut, tester
        code.interact(banner=f"Origen {origen.version}", local=locals(), exitmsg="")

    else:
        print(f"Unknown command: {command}")
        exit(1)
