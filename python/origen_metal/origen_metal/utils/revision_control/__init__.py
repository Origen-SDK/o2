import origen_metal
from abc import ABC, abstractclassmethod


class RevisionControl(ABC,
                      origen_metal._origen_metal.utils.revision_control.Base):
    def __init__(self):
        origen_metal._origen_metal.utils.revision_control.Base.__init__(self)

    @abstractclassmethod
    def init(self):
        ...
