import sys, code


def prep_shell(history_file):
    if sys.platform == "win32":
        # The below is needed only for pyreadline, which is needed only for Windows support.
        # This setup was taken from the pyreadline documentation:
        #   https://pythonhosted.org/pyreadline/introduction.html
        # The below was copied directly from the above source.
        #
        #this file is needed in site-packages to emulate readline
        #necessary for rlcompleter since it relies on the existance
        #of a readline module
        from pyreadline3.rlmain import Readline

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

            import pyreadline3.console as console

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

        __all__ += ["rl"]

    import atexit, readline, os, colorama

    # Colorama init only required on windows, but place it here to keep consistent with all platforms, or in case options
    # need to be added
    # Also, its a known issue that powershell doesn't display yellow text correctly. The standard command prompt will
    # though.
    colorama.init()

    # Make sure the tmp directory exists
    history_file.parent.mkdir(exist_ok=True)

    def save_history(history=history_file):
        import readline

        readline.write_history_file(history)

    if os.path.exists(history_file):
        readline.read_history_file(str(history_file))

    atexit.register(save_history)


def metal_context():
    import origen_metal
    return {"origen_metal": origen_metal, "om": origen_metal}


def interact(banner=None, context=None):
    code.interact(
        banner=banner,
        local=context or metal_context(),
    )
