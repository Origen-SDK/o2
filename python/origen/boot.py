# Called by the Origen CLI to boot the Origen Python env, not for application use
# Any target/env overrides given to the command line will be passed in here
def __origen__(command, target=None, environment=None, mode=None):
    import origen
    import _origen
    import origen.application
    import origen.target

    origen.application.load()

    if mode == None:
        origen.set_mode(_origen.app_config()["mode"])
    else:
        origen.set_mode(mode)

    origen.target.load(target=target, environment=environment)

    if command == "generate":
        print("Generate command called!")

    elif command == "interactive":
        import atexit
        import os
        import readline
        import rlcompleter

        historyPath = origen.root.joinpath(".origen").joinpath("console_history")

        def save_history(historyPath=historyPath):
            import readline
            readline.write_history_file(historyPath)

        if os.path.exists(historyPath):
            readline.read_history_file(historyPath)

        atexit.register(save_history)
        del os, atexit, readline, rlcompleter, save_history, historyPath

        import code
        from origen import dut, tester
        code.interact(banner=f"Origen {origen.version}", local=locals(), exitmsg="")

    else:
        print(f"Unknown command: {command}")
        exit(1)
