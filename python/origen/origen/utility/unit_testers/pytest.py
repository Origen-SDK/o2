from . import UnitTester, RunResult
import subprocess


class PyTest(UnitTester):
    def __init__(self, config):
        UnitTester.__init__(self, **config)
        self.exe = config.get("exe", ["poetry", "run", "pytest"])

    def run(self):
        r = subprocess.run(self.exe, shell=True)
        return RunResult(passed=(r.returncode == 0))
