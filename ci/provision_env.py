#!/usr/bin/env python3
"""Install a project's locked environment, for projects that are not Origen apps.

`origen env setup` does this for applications, including the Windows/Python 3.7
workaround described below. The no-application and Origen Metal projects are not
Origen applications, so CI provisions them with this script instead of calling
`uv sync` directly, to keep both paths behaving the same.

Why the workaround exists: UV writes Windows console scripts as a trampoline
with a zip archive appended, and the trampoline runs `python.exe <the .exe
itself>`, relying on the interpreter to execute that archive. CPython's old C
`zipimport`, used up to and including 3.7, rejects it and tries to parse the
executable as source, so every console script in the environment is unusable.
CPython 3.8 replaced `zipimport` with the pure-Python implementation, which
reads it correctly. pip's launchers work on 3.7, so pip installs the environment
there. The contents still come from `uv.lock` by way of `uv export`, so the
resolution is identical either way.

Retire this when O2 drops Python 3.7, or when UV's launcher becomes readable
by it.
"""

import argparse
import pathlib
import re
import subprocess
import sys
import tempfile


def run(command, cwd):
    print(f"+ {' '.join(str(part) for part in command)}", flush=True)
    subprocess.run(command, cwd=cwd, check=True)


def needs_pip_provisioning():
    return sys.platform == "win32" and sys.version_info < (3, 8)


def is_installable_project(project):
    """Does UV install this project itself, or only its dependencies?

    A project marked `[tool.uv] package = false` is virtual: `uv sync` installs
    its dependencies and never builds the project. Installing it with pip anyway
    makes setuptools guess at the layout and publish whatever directories it
    finds, which is how `tests` ended up shadowing real modules. Parsed by hand
    because this has to run on Python 3.7, which has no tomllib.
    """
    section = None
    for line in project.joinpath("pyproject.toml").read_text().splitlines():
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            section = stripped[1:-1]
            continue
        if section == "tool.uv" and re.match(r"^package\s*=\s*false\b", stripped):
            return False
    return True


def provision_with_pip(project):
    print(
        f"Python {sys.version.split()[0]} on Windows cannot execute UV's console-script "
        "launchers; installing with pip from the exported lock instead.",
        flush=True,
    )
    venv = project / ".venv"
    # --clear replaces any existing environment: unlike `uv sync`, installing
    # with pip cannot prune packages that are no longer in the lock, so starting
    # clean is what keeps this path equivalent to a sync. --python pins the
    # interpreter, which UV would otherwise choose for itself.
    run(
        ["uv", "venv", "--seed", "--clear", "--python", sys.executable, str(venv)],
        cwd=project,
    )

    with tempfile.TemporaryDirectory() as scratch:
        requirements = pathlib.Path(scratch) / "requirements.txt"
        run(
            [
                "uv", "export",
                "--frozen",
                "--all-groups",
                "--no-hashes",
                "--no-emit-project",
                "-o", str(requirements),
            ],
            cwd=project,
        )
        python = venv / "Scripts" / "python.exe"
        run([str(python), "-m", "pip", "install", "-r", str(requirements)], cwd=project)
        if is_installable_project(project):
            run([str(python), "-m", "pip", "install", "--no-deps", "."], cwd=project)
        else:
            print("Project is virtual (tool.uv package = false); dependencies only.", flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("project", type=pathlib.Path)
    project = parser.parse_args().project.resolve()

    if needs_pip_provisioning():
        provision_with_pip(project)
    else:
        run(["uv", "sync", "--all-groups", "--no-editable"], cwd=project)


if __name__ == "__main__":
    main()
