import pytest, origen, _origen
from tests.shared.python_like_apis import Fixture_DictLikeAPI

SERVER = "ldap://ldap.forumsys.com:389"
BASE = "dc=example,dc=com"
AUTH_TYPE = "simple_bind"
AUTH_USERNAME = "cn=read-only-admin,dc=example,dc=com"
USER_USERNAME = "uid=euler,dc=example,dc=com"
PASSWORD = "password"
NAME = "forumsys"


class TestLDAPs:
    @property
    def ldap(self):
        return origen.ldaps[NAME]

    class TestLDAPsDictLike(Fixture_DictLikeAPI):
        def parameterize(self):
            return {
                "keys": [NAME],
                "klass": _origen.utility.ldap.LDAP,
                "not_in_dut": "Blah"
            }

        def boot_dict_under_test(self):
            return origen.ldaps

    def test_ldap_parameters(self):
        assert self.ldap.base == BASE
        assert self.ldap.server == SERVER
        assert self.ldap.name == NAME
        assert self.ldap.bound == False
        assert self.ldap.auth == {
            'type': AUTH_TYPE,
            'username': AUTH_USERNAME,
            'password': PASSWORD
        }

    def test_ldap_can_bind(self):
        assert self.ldap.bind()
        assert self.ldap.bound == True

    def test_ldap_searching(self):
        results = self.ldap.search("(uid=euler)", [])
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
        results = self.ldap.search("(|(uid=tesla)(uid=curie))", ["cn", "mail"])
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
        results = self.ldap.search("(|(uid=tesla)(uid=curie))", ["BLAH"])
        assert results == {
            'uid=curie,dc=example,dc=com': ({}, {}),
            'uid=tesla,dc=example,dc=com': ({}, {})
        }
        results = self.ldap.search("(|(uid=blah)(uid=none))", ["BLAH"])
        assert results == {}

    def test_single_filter_search(self):
        results = self.ldap.single_filter_search("(uid=tesla)", ["cn", "mail"])
        assert results == ({
            'mail': ['tesla@ldap.forumsys.com'],
            'cn': ['Nikola Tesla']
        }, {})
        results = self.ldap.single_filter_search("(uid=blah)", ["cn", "mail"])
        assert results == ({}, {})

    def test_error_if_single_filter_search_returns_multiple_dns(self):
        with pytest.raises(OSError,
                           match="expected a single DN result from filter"):
            self.ldap.single_filter_search("(|(uid=tesla)(uid=Curie))",
                                           ["mail"])

    def test_unbind_and_rebind(self):
        assert self.ldap.bound
        assert self.ldap.unbind()
        assert not self.ldap.bound
        assert self.ldap.bind()
        assert self.ldap.bound

    def test_validating_passwords(self):
        assert self.ldap.bound == True
        assert self.ldap.validate_credentials(USER_USERNAME, PASSWORD)
        assert not self.ldap.validate_credentials(USER_USERNAME, "?")
        # Should not effect the current LDAP
        assert self.ldap.bound == True

    def test_binding_with_a_different_user(self):
        assert self.ldap.auth == {
            'type': AUTH_TYPE,
            'username': AUTH_USERNAME,
            'password': PASSWORD
        }
        assert self.ldap.bound == True

        assert self.ldap.bind_as(USER_USERNAME, PASSWORD)
        assert self.ldap.bound == True
        assert self.ldap.auth == {
            'type': AUTH_TYPE,
            'username': USER_USERNAME,
            'password': PASSWORD
        }

        with pytest.raises(OSError, match="invalidCredentials"):
            self.ldap.bind_as(USER_USERNAME, "?")
