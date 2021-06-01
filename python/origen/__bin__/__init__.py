import sys
import subprocess
import pathlib


def run_origen():
    subprocess.run(
        str(pathlib.Path(__file__).parent.absolute().joinpath("bin").joinpath("origen")) + " " + " ".join(sys.argv[1:]),
        shell=True
    )
