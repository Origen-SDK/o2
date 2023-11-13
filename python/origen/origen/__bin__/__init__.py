import sys
import subprocess
import pathlib
import platform
on_windows = platform.system() == 'Windows'

def run_origen():
    if on_windows:
        subprocess.run([
            str(pathlib.Path(__file__).parent.absolute().joinpath("bin").joinpath("origen")),
            *sys.argv[1:]
        ], shell=on_windows)
    else:
        subprocess.run(str(
            pathlib.Path(__file__).parent.absolute().joinpath("bin").joinpath(
                "origen")) + " " + " ".join(sys.argv[1:]),
                    shell=True)
