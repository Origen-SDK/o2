# TODO add tests and remove equivalents
import inspect, subprocess, pathlib, os
from origen_metal._helpers import in_new_proc

def in_new_origen_proc(func=None, mod=None, *, func_kwargs=None, with_configs=None, bypass_config_lookup=False):
    if isinstance(with_configs, str) or isinstance(with_configs, pathlib.Path):
        with_configs=[with_configs]

    if func is None:
        func = getattr(mod, inspect.stack()[1].function)
    return in_new_proc(func, mod, func_kwargs=func_kwargs)

# TODO support options
def run_cli_cmd(cmd, *, with_env=None, with_configs=None, bypass_config_lookup=False, input=None, expect_fail=False, return_details=False):
    if isinstance(cmd, str):
        cmd = ["origen", cmd]
    else:
        cmd = ["origen", *cmd]

    subp_env = os.environ.copy()
    if isinstance(with_configs, str) or isinstance(with_configs, pathlib.Path):
        with_configs=[with_configs]
    if with_configs:
        subp_env["origen_config_paths"] = ';'.join([str(c) for c in with_configs])

    if with_env:
        subp_env.update(with_env)

    if bypass_config_lookup:
        subp_env["origen_bypass_config_lookup"] = "1"

    if expect_fail:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, input=input, env=subp_env)
        if result.returncode == 0:
            cmd = ' '.join(cmd)
            raise RuntimeError(f"Expected cmd '{cmd}' to fail but received return code 0")
    else:
        result = subprocess.run(cmd, shell=True, check=False, capture_output=True, text=True, input=input, env=subp_env)
        # FOR_PR take these out
        print(result.stdout)
        print(result.stderr)
    if return_details:
        return {
            # FOR_PR take these out
            # "stderr": result.stderr.decode("utf-8"),
            # "stdout": result.stdout.decode("utf-8"),
            "stderr": result.stderr,
            "stdout": result.stdout,
            "returncode": result.returncode
        }
    else:
        # FOR_PR take these out
        # return result.stdout.decode("utf-8")
        return result.stdout
