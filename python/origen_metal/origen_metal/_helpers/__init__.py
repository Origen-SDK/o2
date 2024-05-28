import inspect
import multiprocessing as mp
import subprocess as sp
from typing import Any

PACKAGE_NOT_FOUND_MESSAGE = "Package(s) not found:"

# TODO pip_show: add some basic tests
class PipShowReturn:
    def __init__(self, output):
        self.fields = dict([l.strip().split(":", 1) for l in output.split("\n")[:-1]])
        self.fields = dict([k.strip(), v.strip()] for k, v in self.fields.items())
        self._field_names = dict([f.lower().replace('-', '_'), f] for f in self.fields.keys())

    def __getattr__(self, name: str) -> Any:
        if name in self._field_names:
            return self.fields[self._field_names[name]]
        return object.__getattribute__(self, name)

def pip_show(package, *, no_parse=False, wrap_poetry=False):
    if wrap_poetry:
        cmd = ["poetry",  "run"]
    else:
        cmd = []
    cmd += ["pip", "show", package]
    result = sp.run(cmd, capture_output=True, text=True)
    if no_parse:
        return result
    if PACKAGE_NOT_FOUND_MESSAGE in result.stderr:
        return None
    return PipShowReturn(result.stdout)

# TODO swap out assert?

def in_new_proc(func=None, mod=None, func_kwargs=None, expect_fail=False):
    if func is None:
        func = getattr(mod, inspect.stack()[1].function)
    context = mp.get_context("spawn")
    q = context.Queue()

    args=(q, func_kwargs)
    proc = context.Process(target=func, args=args)
    proc.start()
    proc.join()
    results = {}
    while not q.empty():
        # Convert the populated Queue to a dictionary
        obj = q.get()
        results[obj[0]] = obj[1]
    if expect_fail:
        assert proc.exitcode == 1
    else:
        assert proc.exitcode == 0
    return results
