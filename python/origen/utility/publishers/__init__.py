import _origen
from abc import ABC, abstractmethod


class Publisher(ABC):
    def __init__(self, **config):
        pass

    @abstractmethod
    def build_package(self, *args, **kwargs):
        pass

    @abstractmethod
    def upload(self, build_result):
        pass
