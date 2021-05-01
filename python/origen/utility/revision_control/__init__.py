import _origen
from abc import ABC

class RevisionControl(ABC, _origen.utility.revision_control.Base):
    def __init__(self):
        _origen.utility.revision_control.Base.__init__(self)
