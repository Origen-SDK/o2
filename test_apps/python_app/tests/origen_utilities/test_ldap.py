# '''
# Testing that the LDAP driver can do 'ldap things' are handled by Origen Metal

# The tests here are just checking the interface between Origen and Origen Metal
# '''

import pytest, origen, pathlib
from tests.shared import in_new_origen_proc
from tests import om_shared
from configs import ldap as ldap_configs

# Grab the dummy RC from origen_metal's tests
with om_shared():
    from om_tests import test_frontend  # type:ignore
    from om_tests.utils.test_ldap import Common as LdapCommon  # type:ignore

@pytest.mark.ldap
class TestLDAPs(LdapCommon, test_frontend.Common):
    def test_ldaps_are_accessible(self, dummy_config):
        n = "test_ldap"
        assert isinstance(origen.ldaps, self.cat_class)
        num_ldaps = len(origen.ldaps)

        assert isinstance(
            origen.ldaps.add(n, self.ldap_class,
                             ["t", dummy_config.server, dummy_config.base, dummy_config.auth_config]), self.ldap_class)
        assert len(origen.ldaps) == num_ldaps + 1
        assert n in origen.ldaps
        assert isinstance(origen.ldaps[n], self.ldap_class)

    def test_simple_ldap(self):
        retn = in_new_origen_proc(mod=ldap_configs)

    def test_fully_configured_ldap(self):
        retn = in_new_origen_proc(mod=ldap_configs)

    def test_multiple_ldaps(self):
        retn = in_new_origen_proc(mod=ldap_configs)

    def test_bad_ldap_config(self, capfd):
        retn = in_new_origen_proc(mod=ldap_configs, expect_fail=True)
        out = capfd.readouterr().out
        assert "Malformed config file" in out
        p = pathlib.Path("tests/origen_utilities/configs/ldap/test_bad_ldap_config.toml")
        assert f"invalid type: string \"hi\", expected an integer for key `ldaps.bad.timeout` in {str(p)}" in out

    def test_empty_config(self):
        retn = in_new_origen_proc(mod=ldap_configs)

    def test_empty_ldaps(self):
        retn = in_new_origen_proc(mod=ldap_configs)

    def test_empty_ldap(self, capfd):
        retn = in_new_origen_proc(mod=ldap_configs, expect_fail=True)
        out = capfd.readouterr().out
        assert "Malformed config file" in out
        assert "missing field `server`" in out

    @pytest.mark.skip
    def test_adding_ldaps_with_non_ldap_types(self):
        raise NotImplementedError
