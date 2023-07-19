# TODO add tests and remove equivalents
import inspect, subprocess, pathlib, os
from origen_metal._helpers import in_new_proc
from origen import running_on_windows

def in_new_origen_proc(func=None, mod=None, *, func_kwargs=None, with_configs=None, expect_fail=None, bypass_config_lookup=False):
    if isinstance(with_configs, str) or isinstance(with_configs, pathlib.Path):
        with_configs=[with_configs]

    if func is None:
        func = getattr(mod, inspect.stack()[1].function)
    return in_new_proc(func, mod, func_kwargs=func_kwargs, expect_fail=expect_fail)

def run_cli_cmd(cmd, *,
    with_env=None,
    with_configs=None,
    bypass_config_lookup=False,
    input=None,
    expect_fail=False,
    return_details=False,
    shell=None,
    targets=None,
    check=True,
    poetry_run=False,
    origen_exe=None
):
    if isinstance(cmd, str):
        cmd = [cmd]
    else:
        def to_cmd(c):
            if isinstance(c, pathlib.Path):
                return c.as_posix()
            else:
                return c
        cmd = list(map(to_cmd, cmd))

    if (origen_exe is None) or isinstance(origen_exe, str):
        origen_exe = [origen_exe or 'origen']
    if poetry_run:
        origen_exe = ["poetry", "run", *origen_exe]
    cmd = [*origen_exe, *cmd]

    subp_env = os.environ.copy()
    if isinstance(with_configs, str) or isinstance(with_configs, pathlib.Path):
        with_configs=[with_configs]
    if with_configs:
        subp_env["origen_config_paths"] = os.pathsep.join([str(c) for c in with_configs])

    if with_env:
        subp_env.update(with_env)

    if bypass_config_lookup:
        subp_env["origen_bypass_config_lookup"] = "1"

    if shell is None:
        shell = running_on_windows

    if targets is False:
        cmd.append("--no_targets")
    elif targets:
        if isinstance(targets, str):
            targets = [targets]
        cmd += ["-t", *targets]

    if expect_fail:
        result = subprocess.run(cmd, shell=shell, capture_output=True, text=True, input=input, env=subp_env)
        if result.returncode == 0:
            cmd = ' '.join(cmd)
            raise RuntimeError(f"Expected cmd '{cmd}' to fail but received return code 0")
    else:
        result = subprocess.run(cmd, shell=shell, check=check, capture_output=True, text=True, input=input, env=subp_env)
    if return_details:
        return {
            "stderr": result.stderr,
            "stdout": result.stdout,
            "returncode": result.returncode
        }
    else:
        return result.stdout
