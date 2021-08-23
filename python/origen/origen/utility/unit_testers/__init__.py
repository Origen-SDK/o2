import _origen
from abc import ABC, abstractmethod


class UnitTester(ABC):
    @abstractmethod
    def run(self, *args, **kwargs):
        pass


class RunResult(_origen.utility.unit_testers.RunResult):
    def __init__(self, passed):
        _origen.utility.unit_testers.RunResult.__init__(self, passed, None)
