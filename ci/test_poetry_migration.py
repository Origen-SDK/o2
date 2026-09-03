#!/usr/bin/env python3
"""Exercise a legacy Poetry application through O2's complete UV migration flow."""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE_APP = ROOT / "test_apps" / "python_app"
POETRY_MANIFEST = (
    ROOT / "rust" / "origen" / "cli" / "tests" / "fixtures" / "poetry" / "python_app.toml"
)


def run(
    *args: str,
    cwd: Path,
    capture: bool = False,
    pin_python: bool = False,
) -> subprocess.CompletedProcess[str]:
    environment = os.environ.copy()
    if pin_python:
        environment["UV_PYTHON"] = sys.executable
    return subprocess.run(
        args,
        cwd=str(cwd),
        env=environment,
        check=True,
        text=True,
        capture_output=capture,
    )


def main() -> None:
    with tempfile.TemporaryDirectory(
        prefix=".poetry-migration-", dir=str(ROOT / "test_apps")
    ) as project_directory, tempfile.TemporaryDirectory(
        prefix=".poetry-dependencies-", dir=str(ROOT / "test_apps")
    ) as dependency_directory:
        project = Path(project_directory)
        workspace = Path(dependency_directory)
        shutil.copytree(
            SOURCE_APP,
            project,
            dirs_exist_ok=True,
            ignore=shutil.ignore_patterns(
                ".venv",
                "pyproject.toml",
                "uv.lock",
                "__pycache__",
                ".pytest_cache",
                "output",
            ),
        )
        # Copy path dependencies into an isolated tree. Their project-local UV
        # source declarations are root-only configuration; remove entries for
        # packages already sourced by the historical application's root so UV
        # sees one authoritative editable URL for each package.
        dependencies = workspace / "dependencies"
        dependency_sources = {
            "origen": ROOT / "python" / "origen",
            "python_plugin": ROOT / "test_apps" / "python_plugin",
            "test_apps_shared_test_helpers": (
                ROOT / "test_apps" / "test_apps_shared_test_helpers"
            ),
            "python_plugin_no_cmds": ROOT / "test_apps" / "python_plugin_no_cmds",
            "pl_ext_cmds": ROOT / "test_apps" / "pl_ext_cmds",
        }
        for name, source in dependency_sources.items():
            destination = dependencies / name
            shutil.copytree(
                source,
                destination,
                ignore=shutil.ignore_patterns(
                    ".venv",
                    "uv.lock",
                    "__pycache__",
                    ".pytest_cache",
                    "dist",
                    "output",
                ),
            )
            manifest_path = destination / "pyproject.toml"
            manifest_lines = manifest_path.read_text().splitlines(keepends=True)
            manifest_path.write_text(
                "".join(
                    line
                    for line in manifest_lines
                    if not line.startswith(("origen = {", "origen-metal = {"))
                )
            )
        os.symlink(ROOT / "rust", workspace / "rust", target_is_directory=True)

        original_manifest_text = POETRY_MANIFEST.read_text()
        replacements = {
            "../../python/origen": dependencies.joinpath("origen").as_posix(),
            "../../python/origen_metal": (
                ROOT / "python" / "origen_metal"
            ).as_posix(),
            "../python_plugin": dependencies.joinpath("python_plugin").as_posix(),
            "../test_apps_shared_test_helpers": dependencies.joinpath(
                "test_apps_shared_test_helpers"
            ).as_posix(),
            "../python_plugin_no_cmds": dependencies.joinpath(
                "python_plugin_no_cmds"
            ).as_posix(),
        }
        for old, new in replacements.items():
            assert old in original_manifest_text
            original_manifest_text = original_manifest_text.replace(old, new, 1)
        original_manifest = original_manifest_text.encode()
        original_poetry_lock = b"historical Poetry lock sentinel\n"
        pyproject = project / "pyproject.toml"
        poetry_lock = project / "poetry.lock"
        uv_lock = project / "uv.lock"
        pyproject.write_bytes(original_manifest)
        poetry_lock.write_bytes(original_poetry_lock)

        preview = run("origen", "env", "migrate", "--dry-run", cwd=project, capture=True)
        assert "--- a/pyproject.toml" in preview.stdout
        assert "+++ b/pyproject.toml" in preview.stdout
        assert pyproject.read_bytes() == original_manifest
        assert poetry_lock.read_bytes() == original_poetry_lock
        assert not uv_lock.exists()

        run("origen", "env", "migrate", cwd=project, pin_python=True)
        assert b"[tool.poetry]" not in pyproject.read_bytes()
        assert uv_lock.is_file()
        assert not poetry_lock.exists()
        migrated_manifest = pyproject.read_bytes()
        migrated_lock = uv_lock.read_bytes()
        repeated = run("origen", "env", "migrate", cwd=project, capture=True)
        assert "already migrated" in repeated.stdout
        assert pyproject.read_bytes() == migrated_manifest
        assert uv_lock.read_bytes() == migrated_lock

        run("uv", "lock", "--check", cwd=project, pin_python=True)
        run("origen", "env", "setup", cwd=project, pin_python=True)
        run("origen", "-v", cwd=project)
        run(
            "origen",
            "exec",
            "python",
            "-c",
            "import origen; import origen_metal",
            cwd=project,
        )
        run("origen", "exec", "pytest", "-q", cwd=project)


if __name__ == "__main__":
    main()
