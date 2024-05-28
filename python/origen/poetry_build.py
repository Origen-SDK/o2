import pathlib, shutil, os
from build_helpers import windows, compile_rust, publish_step, ls_dir

if __name__ == '__main__':
    if publish_step:
        print("Building libraries for packaging...")
        origen_pkg_root = pathlib.Path(__file__).parent.absolute()
        origen_pkg_lib = origen_pkg_root.joinpath(f"_origen.{'pyd' if windows else 'so'}")

        rust_build_target = os.getenv("ORIGEN__BUILD_TARGET", "release")
        build_lib = os.getenv("ORIGEN__BUILD_LIB", True)
        copy_lib = os.getenv("ORIGEN__COPY_LIB", True)
        if copy_lib == "0":
            copy_lib = False
        origen_lib_src = origen_pkg_root.joinpath("../../rust/pyapi")
        origen_lib_target = origen_lib_src.joinpath(f"target/{rust_build_target}/{'' if windows else 'lib'}_origen.{'dll' if windows else 'so'}")
        build_cli = os.getenv("ORIGEN__BUILD_CLI", True)
        copy_cli = os.getenv("ORIGEN__COPY_CLI", True)
        if copy_cli == "0":
            copy_cli = False
        origen_cli_src = origen_pkg_root.joinpath("../../rust/origen")
        origen_cli_target = origen_cli_src.joinpath(f"target/{rust_build_target}/origen{'.exe' if windows else ''}")
        origen_cli_pkg_dir = origen_pkg_root.joinpath("origen/__bin__/bin")

        # Build and copy Origen CLI
        if build_cli:
            compile_rust(origen_cli_src, rust_build_target, True)
        if copy_cli:
            print(f"Copying CLI for packaging from ({origen_cli_target} to {origen_cli_pkg_dir})")
            ls_dir(origen_cli_target.parent)
            shutil.copy2(origen_cli_target, origen_cli_pkg_dir)

        # Build and copy Origen library
        if build_lib:
            compile_rust(origen_lib_src, rust_build_target, False)
        if copy_lib:
            print(f"Copying CLI for packaging ({origen_lib_target} to {origen_pkg_lib})")
            ls_dir(origen_lib_target.parent)
            shutil.copy2(origen_lib_target, origen_pkg_lib)
        print("Libraries built!")
    else:
        print("Skipping library build steps when run as a dependency")

def build(arg):
    # This method is called during install. Very important this is defined and
    # does nothing.
    pass
