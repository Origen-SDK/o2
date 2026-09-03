import email.message
import pathlib
import subprocess
import sys
import zipfile


SCRIPT = pathlib.Path(__file__).with_name("validate_release.py")


def run(*args):
    return subprocess.run([sys.executable, str(SCRIPT), *map(str, args)], capture_output=True, text=True)


def test_source_validation(tmp_path):
    manifest = tmp_path / "pyproject.toml"
    manifest.write_text('[project]\nname = "example"\nversion = "1.2.3"\n')
    history = tmp_path / "history"
    history.write_text("(release-example-1-2-3)=\n")
    result = run("source", "--version", "1.2.3", "--tag-prefix", "example",
                 "--tags", "example-v1.2.3", "--history", history,
                 "--manifest", manifest)
    assert result.returncode == 0, result.stderr


def test_source_validation_accepts_semver_form_of_pep440_prerelease(tmp_path):
    manifest = tmp_path / "Cargo.toml"
    manifest.write_text('[package]\nname = "example"\nversion = "2.0.0-dev.9"\n')
    history = tmp_path / "history"
    history.write_text("(release-example-2-0-0-dev9)=\n")
    result = run("source", "--version", "2.0.0.dev9", "--tag-prefix", "example",
                 "--tags", "example-v2.0.0.dev9", "--history", history,
                 "--manifest", manifest)
    assert result.returncode == 0, result.stderr


def test_wheel_validation_rejects_wrong_version(tmp_path):
    wheel = tmp_path / "example-1.0.0-py3-none-any.whl"
    metadata = email.message.Message()
    metadata["Name"] = "example"
    metadata["Version"] = "1.0.0"
    with zipfile.ZipFile(wheel, "w") as archive:
        archive.writestr("example-1.0.0.dist-info/METADATA", metadata.as_bytes())
    result = run("wheels", "--package", "example", "--version", "2.0.0",
                 "--directory", tmp_path)
    assert result.returncode == 1
    assert "Unexpected metadata" in result.stderr


def _wheel(directory, filename, version="1.0.0", name="example"):
    metadata = email.message.Message()
    metadata["Name"] = name
    metadata["Version"] = version
    with zipfile.ZipFile(directory / filename, "w") as archive:
        archive.writestr(f"{name}-{version}.dist-info/METADATA", metadata.as_bytes())


def test_wheel_validation_accepts_expected_python_tag(tmp_path):
    _wheel(tmp_path, "example-1.0.0-cp311-cp311-win_amd64.whl")
    result = run("wheels", "--package", "example", "--version", "1.0.0",
                 "--directory", tmp_path, "--expect-python-versions", "3.11")
    assert result.returncode == 0, result.stderr


def test_wheel_validation_rejects_wrong_python_tag(tmp_path):
    # The failure this guards against: uv resolves a different interpreter for
    # the isolated build environment, so the matrix cell emits the wrong ABI.
    _wheel(tmp_path, "example-1.0.0-cp311-cp311-win_amd64.whl")
    result = run("wheels", "--package", "example", "--version", "1.0.0",
                 "--directory", tmp_path, "--expect-python-versions", "3.9")
    assert result.returncode == 1
    assert "Missing cp39" in result.stderr


def test_wheel_validation_reports_missing_version_per_platform(tmp_path):
    _wheel(tmp_path, "example-1.0.0-cp38-cp38-win_amd64.whl")
    _wheel(tmp_path, "example-1.0.0-cp39-cp39-win_amd64.whl")
    result = run("wheels", "--package", "example", "--version", "1.0.0",
                 "--directory", tmp_path, "--expect-python-versions", "3.8,3.9,3.10")
    assert result.returncode == 1
    assert "Missing cp310" in result.stderr


def test_wheel_validation_rejects_malformed_filename(tmp_path):
    _wheel(tmp_path, "example-1.0.0.whl")
    result = run("wheels", "--package", "example", "--version", "1.0.0",
                 "--directory", tmp_path)
    assert result.returncode == 1
    assert "Malformed wheel filename" in result.stderr
