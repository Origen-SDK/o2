"""Lightweight console launcher that avoids booting Origen twice."""

import os
import pathlib
import platform
import subprocess
import sys


def run_origen():
    on_windows = platform.system() == "Windows"
    executable_name = "origen.exe" if on_windows else "origen"
    package_root = pathlib.Path(__file__).resolve().parent
    packaged = package_root.joinpath("origen", "__bin__", "bin", executable_name)
    candidates = []

    if os.getenv("ORIGEN_CLI"):
        candidates.append(pathlib.Path(os.environ["ORIGEN_CLI"]))

    for parent in [pathlib.Path.cwd(), *pathlib.Path.cwd().parents]:
        if parent.joinpath(".origen_dev_workspace").exists():
            candidates.append(
                parent.joinpath("rust", "origen", "target", "debug", executable_name)
            )
            break

    candidates.append(packaged)

    # PATH is only a final compatibility fallback. In particular, an installed
    # O2 console script must not accidentally launch an Origen v1/Ruby binary
    # with the same name instead of the CLI bundled in its own wheel.
    wrapper = pathlib.Path(sys.argv[0]).resolve()
    for entry in os.getenv("PATH", "").split(os.pathsep):
        candidate = pathlib.Path(entry).joinpath(executable_name)
        if candidate.exists() and candidate.resolve() != wrapper:
            candidates.append(candidate)

    executable = next((path for path in candidates if path.is_file()), None)
    if executable is None:
        raise FileNotFoundError(
            "Could not locate the Origen CLI binary. Build it with Cargo or set ORIGEN_CLI."
        )

    return subprocess.run(
        [str(executable), *sys.argv[1:]],
        shell=on_windows,
        check=False,
    ).returncode
