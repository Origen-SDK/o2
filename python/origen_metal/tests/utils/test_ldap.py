import pytest
import origen_metal as om
from origen_metal.utils.ldap import LDAP
from origen_metal.frontend import DataStoreView

SERVER = "ldap://ldap.forumsys.com:389"
BASE = "dc=example,dc=com"
AUTH_TYPE = "simple_bind"
AUTH_USERNAME = "cn=read-only-admin,dc=example,dc=com"
USER_USERNAME = "uid=euler,dc=example,dc=com"
PASSWORD = "password"
NAME = "forumsys"

class TestStandaloneLDAP:
    def forumsys_ldap(self):
        return om._origen_metal.utils.ldap.LDAP(
            name=NAME,
            server=SERVER,
            base=BASE,
            auth=AUTH_TYPE,
            username=AUTH_USERNAME,
            password=PASSWORD
        )

    def test_ldap_parameters(self):
        ldap = self.forumsys_ldap()
        assert ldap.base == BASE
        assert ldap.server == SERVER
        assert ldap.name == NAME
        assert ldap.bound == False
        assert ldap.auth == {
            'type': AUTH_TYPE,
            'username': AUTH_USERNAME,
            'password': PASSWORD
        }

    def test_ldap_can_bind(self):
        ldap = self.forumsys_ldap()
        assert ldap.bind()
        assert ldap.bound == True

    def test_ldap_searching(self):
        ldap = self.forumsys_ldap()
        results = ldap.search("(uid=euler)", [])
        assert results == {
            'uid=euler,dc=example,dc=com': ({
                'cn': ['Leonhard Euler'],
                'sn': ['Euler'],
                'uid': ['euler'],
                'objectClass':
                ['inetOrgPerson', 'organizationalPerson', 'person', 'top'],
                'mail': ['euler@ldap.forumsys.com']
            }, {})
        }
        results = ldap.search("(|(uid=tesla)(uid=curie))", ["cn", "mail"])
        assert results == {
            'uid=tesla,dc=example,dc=com': ({
                'cn': ['Nikola Tesla'],
                'mail': ['tesla@ldap.forumsys.com']
            }, {}),
            'uid=curie,dc=example,dc=com': ({
                'mail': ['curie@ldap.forumsys.com'],
                'cn': ['Marie Curie']
            }, {})
        }
        results = ldap.search("(|(uid=tesla)(uid=curie))", ["BLAH"])
        assert results == {
            'uid=curie,dc=example,dc=com': ({}, {}),
            'uid=tesla,dc=example,dc=com': ({}, {})
        }
        results = ldap.search("(|(uid=blah)(uid=none))", ["BLAH"])
        assert results == {}

    def test_single_filter_search(self):
        ldap = self.forumsys_ldap()
        results = ldap.single_filter_search("(uid=tesla)", ["cn", "mail"])
        assert results == ({
            'mail': ['tesla@ldap.forumsys.com'],
            'cn': ['Nikola Tesla']
        }, {})
        results = ldap.single_filter_search("(uid=blah)", ["cn", "mail"])
        assert results == ({}, {})

    def test_error_if_single_filter_search_returns_multiple_dns(self):
        ldap = self.forumsys_ldap()
        with pytest.raises(RuntimeError,
                           match="expected a single DN result from filter"):
            ldap.single_filter_search("(|(uid=tesla)(uid=Curie))",
                                           ["mail"])

    def test_unbind_and_rebind(self):
        ldap = self.forumsys_ldap()
        assert ldap.bind()
        assert ldap.bound
        assert ldap.unbind()
        assert not ldap.bound
        assert ldap.bind()
        assert ldap.bound

    def test_validating_passwords(self):
        ldap = self.forumsys_ldap()
        assert ldap.bind()
        assert ldap.bound == True
        assert ldap.validate_credentials(USER_USERNAME, PASSWORD)
        assert not ldap.validate_credentials(USER_USERNAME, "?")
        # Should not effect the current LDAP
        assert ldap.bound == True

class TestLdapAsDataStore(DataStoreView):
    ''' The LDAP's only data store feature is populating users'''

    def parameterize(self):
        return {
            "init_args": [
                self.ds_test_name,
                SERVER,
                BASE,
                AUTH_TYPE,
                AUTH_USERNAME,
                PASSWORD
            ],
        }

    @property
    def data_store_class(self):
        return LDAP

    def test_underlying_ldap_search_works(self):
        assert self.ds.bind()
        assert self.ds.bound == True

        results = self.ds.single_filter_search("(uid=tesla)", ["cn", "mail"])
        assert results == ({
            'mail': ['tesla@ldap.forumsys.com'],
            'cn': ['Nikola Tesla']
        }, {})

    # TODO Revisit if this is needed vs. just creating a new one
    # def test_binding_with_a_different_user(self):
    #     ldap = self.forumsys_ldap()
    #     assert ldap.auth == {
    #         'type': AUTH_TYPE,
    #         'username': AUTH_USERNAME,
    #         'password': PASSWORD
    #     }
    #     assert ldap.bind()
    #     assert ldap.bound == True

    #     assert ldap.bind_as(USER_USERNAME, PASSWORD)
    #     assert ldap.bound == True
    #     assert ldap.auth == {
    #         'type': AUTH_TYPE,
    #         'username': USER_USERNAME,
    #         'password': PASSWORD
    #     }

    #     with pytest.raises(OSError, match="invalidCredentials"):
    #         ldap.bind_as(USER_USERNAME, "?")
    #     assert ldap.bound == False

    #     # Restore LDAP to previous settings
    #     ldap.bind_as(AUTH_USERNAME, PASSWORD)
    #     assert ldap.bound == True
