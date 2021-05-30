import pathlib, shutil
current = pathlib.Path(__file__).parent.absolute()

# Package the CLI
cli_src = current.joinpath("../rust/origen/target/debug")
if cli_src.joinpath("origen.exe").exists():
    # Windows
    cli_src = cli_src.joinpath("origen.exe")
elif cli_src.joinpath("origen").exists():
    cli_src = cli_src.joinpath("origen")
else:
    raise RuntimeError(f"Could not locate built CLI in {cli_src}")
cli_pkg_dir = current.joinpath("origen/__bin__/bin")
cli_pkg_dir.mkdir(parents=True, exist_ok=True)
print(f"Copying CLI for packaging ({cli_src} to {cli_pkg_dir})")
shutil.copy2(cli_src, cli_pkg_dir)

# Package the _origen compiled library
_origen_src = current.joinpath("../rust/pyapi/target/debug")
if _origen_src.joinpath("_origen.dll").exists():
    # Windows
    _origen_src = _origen_src.joinpath("_origen.dll")
elif _origen_src.joinpath("lib_origen.so").exists():
    _origen_src = _origen_src.joinpath("lib_origen.so")
else:
    raise RuntimeError(f"Could not locate compiled library in {_origen_src}")
_origen_pkg = current.joinpath("_origen.pyd")
print(f"Copying _origen library for packaging ({_origen_src} to {_origen_pkg})")
shutil.copy2(_origen_src, _origen_pkg)
