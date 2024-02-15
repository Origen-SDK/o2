from pathlib import Path

project_dir = Path(__file__).parent.parent.parent.parent.parent
o2_root = project_dir
cli_dir = project_dir.joinpath("python/origen/origen/__bin__/bin")

# Rust Directories
rust_dir = project_dir.joinpath("rust")
rust_origen_dir = rust_dir.joinpath("origen")
rust_cli_dir = rust_origen_dir.joinpath("cli")
rust_cli_toml = rust_cli_dir.joinpath("cargo.toml")
rust_build_cli_dir = project_dir.joinpath(f"rust/origen/target/debug")

test_apps_dir = project_dir.joinpath("test_apps")
plugins_dir = test_apps_dir # Currently the same but may change if test_apps dir is re-organized