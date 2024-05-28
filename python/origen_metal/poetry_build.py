import pathlib, shutil, os, sys

sys.path.insert(0, str(pathlib.Path(os.path.abspath(__file__)).parent.parent.joinpath("origen")))
from build_helpers import windows, compile_rust, publish_step, ls_dir

if __name__ == '__main__':
    if publish_step:
        print("Building libraries for packaging...")
        om_pkg_root = pathlib.Path(__file__).parent.absolute()
        om_lib = om_pkg_root.joinpath('origen_metal').joinpath(f"_origen_metal.{'pyd' if windows else 'so'}")

        rust_build_target = os.getenv("ORIGEN_METAL__BUILD_TARGET", "release")
        copy_build_target = os.getenv("ORIGEN_METAL__COPY_BUILD_TARGET", True)
        if copy_build_target == "0":
            copy_build_target = False
        om_lib_src = om_pkg_root.joinpath("../../rust/pyapi_metal")
        om_lib_target = om_lib_src.joinpath(f"target/{rust_build_target}")
        om_lib_target = om_lib_target.joinpath(f"{'' if windows else 'lib'}origen_metal.{'dll' if windows else 'so'}")
        compile_lib = os.getenv("ORIGEN_METAL__COMPILE_LIB", True)

        if compile_lib:
            compile_rust(om_lib_src, rust_build_target, False)
        if copy_build_target:
            print(f"Copying compiled library '{om_lib_target}' to '{om_lib}'")
            ls_dir(om_lib_target.parent)
            shutil.copy2(om_lib_target, om_lib)

        # Final check that compiled library is present
        if not om_lib.exists():
            print(f"No OM library found. Expected: {om_lib}")
            exit(1)
    else:
        print("Skipping library build steps when run as a dependency")

def build(arg):
    # This method is called during install. Very important this is defined and
    # does nothing.
    pass
