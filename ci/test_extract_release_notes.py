import pathlib
import subprocess
import sys


SCRIPT = pathlib.Path(__file__).with_name("extract_release_notes.py")


def test_extracts_only_requested_entry(tmp_path):
    history = tmp_path / "history"
    history.write_text("(release-example-2-0-0)=\n# Example 2.0.0\n\nNew\n\n---\n\n(release-example-1-0-0)=\n# Old\n")
    output = tmp_path / "body.md"
    result = subprocess.run([sys.executable, str(SCRIPT), "--history", str(history),
                             "--tag-prefix", "example", "--version", "2.0.0",
                             "--output", str(output)], capture_output=True, text=True)
    assert result.returncode == 0, result.stderr
    assert output.read_text() == "# Example 2.0.0\n\nNew\n"
