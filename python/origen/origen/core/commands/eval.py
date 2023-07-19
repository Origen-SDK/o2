import origen, pathlib

def run(args):
    code = list(args.get('code', []))
    files = list(args.get('scripts', []))
    code_indices = list(origen.current_command.arg_indices.get('code', []))
    file_indices = list(origen.current_command.arg_indices.get('scripts', []))
    if len(code_indices) > 0:
        code_idx = code_indices.pop()
    else:
        code_idx = None
    if len(file_indices) > 0:
        files_idx = file_indices.pop()
    else:
        files_idx = None

    # Build the run order based on where the given code or script file appear in the command line
    to_run = []
    while (code_idx is not None) or (files_idx is not None):
        if (code_idx or -1) > (files_idx or -1):
            to_run.append(code.pop())
            if len(code_indices) > 0:
                code_idx = code_indices.pop()
            else:
                code_idx = None
        else:
            p = pathlib.Path(files.pop())
            if not p.exists():
                msg = f"Could not find script file '{p}'"
                origen.logger.error(msg)
                exit(1)
            to_run.append(p)
            if len(file_indices) > 0:
                files_idx = file_indices.pop()
            else:
                files_idx = None

    # Decouple run environment from boot environment, but assume origen is already imported
    eval_locals = {"origen": origen}
    eval_globals = {}

    # Above is actually built with highest index first, so iterate through in reverse
    for code in reversed(to_run):
        if isinstance(code, pathlib.Path):
            c = open(code).read()
        else:
            c = code

        try:
            exec(c, eval_globals, eval_locals)
        except Exception as e:
            # Doctor the traceback to remove the references to boot.py
            # Cleans up the traceback to just what the user should care about
            import traceback, sys
            tb = e.__traceback__
            exc = traceback.format_exception(None, e, tb)
            exc = [exc[0]] + exc[2:-1] + [exc[-1].strip()]
            if isinstance(code, pathlib.Path):
                origen.logger.error(f"Exception occurred evaluating from script '{code}'")
            else:
                origen.logger.error(f"Exception occurred evaluating code:\n{c}")
            print(''.join(exc), file=sys.stderr)
            exit(1)
