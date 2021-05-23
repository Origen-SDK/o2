# These must come before all other imports.
from __future__ import print_function, unicode_literals, absolute_import

import sys
import pathlib
import importlib

if sys.platform == "win32":
    # The below is needed only for pyreadline, which is needed only for Windows support.
    # This setup was taken from the pyreadline documentation:
    #   https://pythonhosted.org/pyreadline/introduction.html
    # The below was copied directly from the above source.
    #
    #this file is needed in site-packages to emulate readline
    #necessary for rlcompleter since it relies on the existance
    #of a readline module
    from pyreadline.rlmain import Readline
    __all__ = [
        'parse_and_bind',
        'get_line_buffer',
        'insert_text',
        'clear_history',
        'read_init_file',
        'read_history_file',
        'write_history_file',
        'get_current_history_length',
        'get_history_length',
        'get_history_item',
        'set_history_length',
        'set_startup_hook',
        'set_pre_input_hook',
        'set_completer',
        'get_completer',
        'get_begidx',
        'get_endidx',
        'set_completer_delims',
        'get_completer_delims',
        'add_history',
        'callback_handler_install',
        'callback_handler_remove',
        'callback_read_char',
    ]  #Some other objects are added below

    # create a Readline object to contain the state
    rl = Readline()

    if rl.disable_readline:

        def dummy(completer=""):
            pass

        for funk in __all__:
            globals()[funk] = dummy
    else:

        def GetOutputFile():
            '''Return the console object used by readline so that it can be used for printing in color.'''
            return rl.console

        __all__.append("GetOutputFile")

        import pyreadline.console as console

        # make these available so this looks like the python readline module
        read_init_file = rl.read_init_file
        parse_and_bind = rl.parse_and_bind
        clear_history = rl.clear_history
        add_history = rl.add_history
        insert_text = rl.insert_text

        write_history_file = rl.write_history_file
        read_history_file = rl.read_history_file

        get_completer_delims = rl.get_completer_delims
        get_current_history_length = rl.get_current_history_length
        get_history_length = rl.get_history_length
        get_history_item = rl.get_history_item
        get_line_buffer = rl.get_line_buffer
        set_completer = rl.set_completer
        get_completer = rl.get_completer
        get_begidx = rl.get_begidx
        get_endidx = rl.get_endidx

        set_completer_delims = rl.set_completer_delims
        set_history_length = rl.set_history_length
        set_pre_input_hook = rl.set_pre_input_hook
        set_startup_hook = rl.set_startup_hook

        callback_handler_install = rl.callback_handler_install
        callback_handler_remove = rl.callback_handler_remove
        callback_read_char = rl.callback_read_char

        console.install_readline(rl.readline)

    __all__ += ["rl", "run_cmd"]


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

    import origen.application
    import origen.target

    if mode == None:
        origen.set_mode(_origen.app_config()["mode"])
    else:
        origen.set_mode(mode)

    if files is not None:
        _origen.file_handler().init(files)

    if verbosity is not None:
        _origen.logger.set_verbosity(verbosity)
        _origen.logger.set_verbosity_keywords(verbosity_keywords.split(","))

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
        import atexit, os, sys, colorama, termcolor, readline, rlcompleter

        # Colorama init only required on windows, but place it here to keep consistent with all platforms, or in case options
        # need to be added
        # Also, its a known issue that powershell doesn't display yellow text correctly. The standard command prompt will
        # though.
        colorama.init()
        historyPath = origen.root.joinpath(".origen").joinpath(
            "console_history")

        def save_history(historyPath=historyPath):
            import readline
            readline.write_history_file(historyPath)

        if os.path.exists(historyPath):
            readline.read_history_file(historyPath)

        atexit.register(save_history)
        del os, atexit, readline, rlcompleter, sys, colorama, termcolor, save_history, historyPath

        import code
        from origen import dut, tester
        from origen.registers.actions import write, verify, write_transaction, verify_transaction
        code.interact(banner=f"Origen {origen.version}",
                      local=locals(),
                      exitmsg="")

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
        origen.app.publish(args)

    # Internal command to give the Origen version loaded by the application to the CLI
    elif command == "_version_":
        print(f"{origen.status['origen_version']}")

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
