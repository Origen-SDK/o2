import importlib, origen

creds = "credentials"
eval = "eval"

_subcmds = None
_base_cmd = None

def import_cmd(cmd):
    return importlib.import_module(f"origen.core.commands.{cmd}")

def is_subcmd(*subcs):
    return list(subcs) == _subcmds

def unsupported_subcmd(subcmd=None):
    if subcmd is None:
        print(f"Unsupported sub-command '{(' -> ').join(_subcmds)}' for base command '{_base_cmd}'")
    else:
        print(f"Unsupported sub-command '{subcmd}' for '{_base_cmd}'")
    exit(1)

def run_core_cmd(base_cmd, sub_cmds, args):
    origen.core.commands._base_cmd = base_cmd
    origen.core.commands._subcmds = sub_cmds
    try:
        if base_cmd == creds:
            import_cmd(creds).run(args)
        elif base_cmd == eval:
            import_cmd(eval).run(args)
        else:
            return False
        return True
    except Exception as e:
        raise e
    finally:
        origen.core.commands._base_cmd = None
        origen.core.commands._subcmds = None
