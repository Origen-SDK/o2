from origen_metal import _origen_metal
from origen_metal.utils.revision_control import RevisionControlAPI

_Git = _origen_metal.utils.revision_control.supported.Git


class Git(_Git, RevisionControlAPI):
    def __init__(self, config):
        _Git.__init__(self, config)
