from origen_metal.frontend import DataStoreAPI
from origen_metal import _origen_metal
_LDAP = _origen_metal.utils.ldap.LDAP

class LDAP(_LDAP, DataStoreAPI):
    def __init__(self, *args, **kwargs):
        _LDAP.__init__(self)
        DataStoreAPI.__init__(self)

    @DataStoreAPI.populate_user
    def populate_user(self, *args, **kwargs):
        _LDAP.populate_user(self, *args, **kwargs)