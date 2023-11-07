import sys
import subprocess
import pathlib

def run_origen():
    subprocess.run([
        pathlib.Path(__file__).parent.absolute().joinpath("bin").joinpath("origen"),
        *sys.argv[1:]
    ], shell=True)
