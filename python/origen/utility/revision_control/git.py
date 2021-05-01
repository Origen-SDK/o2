import _origen
from abc import ABC

class Git(ABC, _origen.utility.revision_control.git.Git):
    def __init__(self, config):
        _origen.utility.revision_control.git.Git.__init__(self, config)
