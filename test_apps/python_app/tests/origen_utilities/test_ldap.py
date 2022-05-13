# '''
# Testing that the LDAP driver can do 'ldap things' are handled by Origen Metal

# The tests here are just checking the interface between Origen and Origen Metal
# '''

import pytest, origen
from tests.shared import in_new_origen_proc
from tests import om_shared
from configs import ldap as ldap_configs

# Grab the dummy RC from origen_metal's tests
with om_shared():
    from om_tests import test_frontend  # type:ignore
    from om_tests.utils.test_ldap import Common as LdapCommon  # type:ignore
    from om_tests.utils.test_ldap import SERVER, BASE, AUTH_SETUP  # type:ignore


class TestLDAPs(LdapCommon, test_frontend.Common):
    def test_ldaps_are_accessible(self):
        n = "test_ldap"
        assert isinstance(origen.ldaps, self.cat_class)
        num_ldaps = len(origen.ldaps)

        assert isinstance(
            origen.ldaps.add(n, self.ldap_class,
                             ["t", SERVER, BASE, AUTH_SETUP]), self.ldap_class)
        assert len(origen.ldaps) == num_ldaps + 1
        assert n in origen.ldaps
        assert isinstance(origen.ldaps[n], self.ldap_class)

    def test_simple_ldap(self):
        retn = in_new_origen_proc(mod=ldap_configs)

    def test_fully_configured_ldap(self):
        retn = in_new_origen_proc(mod=ldap_configs)

    def test_multiple_ldaps(self):
        retn = in_new_origen_proc(mod=ldap_configs)

    # TODO
    @pytest.mark.xfail
    def test_bad_ldap_config(self):
        retn = in_new_origen_proc(mod=ldap_configs)

    def test_empty_config(self):
        retn = in_new_origen_proc(mod=ldap_configs)

    def test_empty_ldaps(self):
        retn = in_new_origen_proc(mod=ldap_configs)

    # TODO
    @pytest.mark.xfail
    def test_empty_ldap(self):
        retn = in_new_origen_proc(mod=ldap_configs)

    @pytest.mark.skip
    def test_adding_ldaps_with_non_ldap_types(self):
        raise NotImplementedError
