import _origen

from origen.web import run_cmd
from . import is_subcmd, unsupported_subcmd


def run(args):
    _origen.set_operation("web")
    if is_subcmd("build"):
        return run_cmd("build", args)
    if is_subcmd("view"):
        return run_cmd("view", args)
    if is_subcmd("serve"):
        return run_cmd("serve", args)
    if is_subcmd("clean"):
        return run_cmd("clean", args)
    unsupported_subcmd()
