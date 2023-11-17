from pathlib import Path

project_dir = Path(__file__).parent.parent.parent.parent.parent
o2_root = project_dir
cli_dir = project_dir.joinpath(f"python/origen/origen/__bin__/bin")
rust_build_cli_dir = project_dir.joinpath(f"rust/origen/target/debug")
test_apps_dir = project_dir.joinpath("test_apps")
plugins_dir = test_apps_dir # Currently the same but may change if test_apps dir is re-organized