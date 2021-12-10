import origen_metal
from abc import ABC, abstractclassmethod

Base = origen_metal._origen_metal.utils.revision_control.Base


class RevisionControlAPI(ABC):
    @abstractclassmethod
    def system(self):
        ...

    @abstractclassmethod
    def populate(self, version):
        ...

    @abstractclassmethod
    def revert(self, path):
        ...

    @abstractclassmethod
    def checkout(self, force, path, version):
        ...

    @abstractclassmethod
    def status(self, path):
        ...

    @abstractclassmethod
    def tag(self, force, path, version):
        ...

    @abstractclassmethod
    def is_initialized(self, tagname, force, message):
        ...

    @abstractclassmethod
    def init(self):
        ...
