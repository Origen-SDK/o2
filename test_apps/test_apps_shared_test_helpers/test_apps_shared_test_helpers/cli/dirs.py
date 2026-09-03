import os
from pathlib import Path

def _find_o2_root():
    if os.getenv("O2_ROOT"):
        return Path(os.environ["O2_ROOT"]).resolve()
    for start in [Path.cwd(), Path(__file__).resolve()]:
        for candidate in [start, *start.parents]:
            if candidate.joinpath(".origen_dev_workspace").is_file():
                return candidate
    raise RuntimeError("Could not locate the O2 workspace; set O2_ROOT")


o2_root = _find_o2_root()
project_dir = o2_root

# Rust Directories
rust_dir = project_dir.joinpath("rust")
rust_origen_dir = rust_dir.joinpath("origen")
rust_cli_dir = rust_origen_dir.joinpath("cli")
rust_cli_toml = rust_cli_dir.joinpath("Cargo.toml")
rust_build_cli_dir = project_dir.joinpath(f"rust/origen/target/debug")
cli_dir = rust_build_cli_dir

test_apps_dir = project_dir.joinpath("test_apps")
plugins_dir = test_apps_dir # Currently the same but may change if test_apps dir is re-organized
