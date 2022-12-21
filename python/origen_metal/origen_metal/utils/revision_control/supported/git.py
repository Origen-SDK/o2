from origen_metal import _origen_metal
from origen_metal.utils.revision_control import RevisionControlAPI
from origen_metal.frontend import DataStoreAPI

_Git = _origen_metal.utils.revision_control.supported.Git


class Git(_Git, RevisionControlAPI, DataStoreAPI):
    def __init__(self, config):
        _Git.__init__(self, config)
        DataStoreAPI.__init__(self)

    @DataStoreAPI.populate_user
    def populate_user(self, *args, **kwargs):
        _Git.populate_user(self, *args, **kwargs)
