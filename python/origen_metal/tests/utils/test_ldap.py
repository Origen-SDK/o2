import pytest
import origen_metal as om
from origen_metal.utils.ldap import LDAP
from origen_metal.frontend import DataStoreView
from tests.framework.users.shared import unload_users, users

SERVER = "ldap://ldap.forumsys.com:389"
BASE = "dc=example,dc=com"
AUTH_TYPE = "simple_bind"
AUTH_USERNAME = "cn=read-only-admin,dc=example,dc=com"
USER_USERNAME = "uid=euler,dc=example,dc=com"
PASSWORD = "password"
NAME = "forumsys"
TIMEOUT = 5
CONTINUOUS_BIND = False
AUTH_SETUP = {
    "scheme": AUTH_TYPE,
    "username": AUTH_USERNAME,
    "password": PASSWORD
}
POPULATE_USER_CONFIG = {
    "data_id": "uid",
    "mapping": {
        "email": "mail",
        "last_name": "sn",
        "full_name": "cn"
    }
}

INIT_PARAMS = [
    SERVER,
    BASE,
    AUTH_SETUP,
    CONTINUOUS_BIND,
    None,
    TIMEOUT,
]


class Common:
    def forumsys_ldap(self, timeout=5, continuous_bind=False, populate_user_config=False):
        if populate_user_config is True:
            pop_config = POPULATE_USER_CONFIG
        elif populate_user_config is False:
            pop_config = None
        else:
            pop_config = populate_user_config
        return om._origen_metal.utils.ldap.LDAP(
            name=NAME,
            server=SERVER,
            base=BASE,
            auth=AUTH_SETUP,
            timeout=timeout,
            continuous_bind=continuous_bind,
            populate_user_config=pop_config,
        )

    @property
    def ldap_class(self):
        return LDAP

    @property
    def init_params(self):
        return INIT_PARAMS


class TestStandaloneLDAP(Common):
    def test_ldap_parameters(self):
        ldap = self.forumsys_ldap()
        assert ldap.base == BASE
        assert ldap.server == SERVER
        assert ldap.name == NAME
        assert ldap.bound == False
        assert ldap.auth_config == {
            'scheme': AUTH_TYPE,
            'username': AUTH_USERNAME,
            'password': PASSWORD,
            'allow_default_password': True,
            'use_default_motives': True,
            'priority_motives': [],
            'backup_motives': [],
        }
        assert ldap.continuous_bind == False
        assert ldap.timeout == 5

    def test_ldap_minimum_parameters(self):
        ldap = om._origen_metal.utils.ldap.LDAP(
            name="min",
            base=BASE,
            server=SERVER,
        )
        assert ldap.name == "min"
        assert ldap.base == BASE
        assert ldap.server == SERVER
        assert ldap.auth_config == {
            'scheme': AUTH_TYPE,
            'username': None,
            'password': None,
            'allow_default_password': True,
            'use_default_motives': True,
            'priority_motives': [],
            'backup_motives': [],
        }
        assert ldap.continuous_bind == False
        assert ldap.timeout == 60
        assert ldap.populate_user_config == None

    def test_ldap_can_bind(self):
        ldap = self.forumsys_ldap(continuous_bind=True)
        assert ldap.continuous_bind == True
        assert ldap.bind()
        assert ldap.bound == True

        # If continuous bind is disabled, ldap.bound should reflect this
        # 'ldap.bind' in this context is more like a 'try to bind' than an actual bind
        ldap = self.forumsys_ldap()
        assert ldap.continuous_bind == False
        assert ldap.bind()
        assert ldap.bound == False

    def test_ldap_timeout_settings(self):
        # Default timeout should be 60
        ldap = om._origen_metal.utils.ldap.LDAP(
            name=NAME,
            server=SERVER,
            base=BASE,
            auth=AUTH_SETUP,
            continuous_bind=False,
        )
        assert ldap.timeout == 60

        # Using 'False' should result in no timeout (will wait indefinitely)
        # This is returned as a "None"
        ldap = self.forumsys_ldap(timeout=False)
        assert ldap.timeout == None

        # Likewise, "True" will just apply the default
        ldap = self.forumsys_ldap(timeout=True)
        assert ldap.timeout == 60

        # Otherwise, timeout option should be used
        ldap = self.forumsys_ldap()
        assert ldap.timeout == 5

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
            ldap.single_filter_search("(|(uid=tesla)(uid=Curie))", ["mail"])

    def test_unbind_and_rebind(self):
        ldap = self.forumsys_ldap(continuous_bind=True)
        assert ldap.bind()
        assert ldap.bound
        assert ldap.unbind()
        assert not ldap.bound
        assert ldap.bind()
        assert ldap.bound

        ldap = self.forumsys_ldap()
        assert ldap.bind()
        assert ldap.bound == False
        assert ldap.unbind() == False

    def test_validating_passwords(self):
        ldap = self.forumsys_ldap(continuous_bind=True)
        assert ldap.bind()
        assert ldap.bound == True
        assert ldap.validate_credentials(USER_USERNAME, PASSWORD)
        assert not ldap.validate_credentials(USER_USERNAME, "?")
        # Should not effect the current LDAP
        assert ldap.bound == True

    def test_populate_user_config(self):
        ldap = self.forumsys_ldap(populate_user_config=True)
        assert ldap.populate_user_config == {
            "data_id": "uid",
            "mapping": {
                "email": "mail",
                "last_name": "sn",
                "full_name": "cn"
            }
        }
    
    def test_timeout_can_be_set(self):
        ldap = self.forumsys_ldap(populate_user_config=True)
        assert ldap.timeout == 5
        ldap.timeout = 10
        assert ldap.timeout == 10
        ldap.timeout = 0
        assert ldap.timeout == 0
        ldap.timeout = None
        assert ldap.timeout is None

class TestLdapAsDataStore(DataStoreView):
    ''' The LDAP's only data store feature is populating users'''
    def parameterize(self):
        return {
            "init_args": [
                self.ds_test_name,
                SERVER,
                BASE,
                AUTH_SETUP,
            ],
        }

    @property
    def data_store_class(self):
        return LDAP

    def test_underlying_ldap_search_works(self):
        results = self.ds.single_filter_search("(uid=tesla)", ["cn", "mail"])
        assert results == ({
            'mail': ['tesla@ldap.forumsys.com'],
            'cn': ['Nikola Tesla']
        }, {})


class TestAuthSetups:
    class TestSimpleBind:
        @pytest.fixture
        def min_auth(self):
            return om._origen_metal.utils.ldap.LDAP(
                name="min",
                base=BASE,
                server=SERVER,
            )

        @pytest.fixture
        def u(self, users):
            if "ldap_user" in users:
                return users["ldap_user"]
            else:
                u = users.add("ldap_user")
                users.set_current_user(u)
                return u

        @pytest.fixture
        def cu(self, users, u):
            return users.current_user

        def test_default_simple_bind_setup(self, unload_users, u, cu):
            u.password = "top_pwd"
            # No auth given but username and password provided assumes 'simple bind'
            ldap = om._origen_metal.utils.ldap.LDAP(
                name="min",
                base=BASE,
                server=SERVER,
            )
            auth = ldap.auth
            assert auth["scheme"] == "simple_bind"
            assert auth["username"] == cu.id
            assert auth["motives"] == ["min", "ldap"]

            assert ldap.auth_config == {
                'scheme': AUTH_TYPE,
                'username': None,
                'password': None,
                'allow_default_password': True,
                'use_default_motives': True,
                'priority_motives': [],
                'backup_motives': [],
            }

        def test_username_and_password_from_current_user(
                self, min_auth, u, cu):
            # Should return the default password
            assert cu.password == "top_pwd"
            assert min_auth.auth["password"] == "top_pwd"

            u.password = "updated_top_pwd"
            assert cu.password == "updated_top_pwd"
            assert min_auth.auth["password"] == "updated_top_pwd"

        def test_using_default_motives(self, min_auth, u):

            # Should return the password for the motive
            l_str = "ldap"
            ds = u.register_dataset(l_str)
            u.add_motive("ldap", "ldap")
            u.datasets[l_str].password = "generic_ldap_pw"
            assert min_auth.auth["password"] == "generic_ldap_pw"

            # Should return the password for the motive
            lname = "min"
            ds = u.register_dataset(lname)
            u.datasets[lname].password = "ldap_min_pw"

            u.add_motive(lname, lname)
            assert min_auth.auth["password"] == "ldap_min_pw"

        def test_using_custom_motives(self, unload_users, u, cu):
            ldap = om._origen_metal.utils.ldap.LDAP(
                name="custom_motives",
                base=BASE,
                server=SERVER,
                auth={
                    "priority_motives": ["ldap_pw"],
                    "backup_motives": ["ldap_pw_backup"],
                    "allow_default_password": False,
                })
            assert ldap.auth_config == {
                'scheme': AUTH_TYPE,
                'username': None,
                'password': None,
                'allow_default_password': False,
                'use_default_motives': True,
                'priority_motives': ["ldap_pw"],
                'backup_motives': ["ldap_pw_backup"],
            }
            motives = ["ldap_pw", "custom_motives", "ldap", "ldap_pw_backup"]

            motives_str = (', ').join([f"'{m}'" for m in motives])
            with pytest.raises(
                    RuntimeError,
                    match=
                    f"No password found for user '{cu.id}' matching motives {motives_str}"
            ):
                ldap.auth

            u.register_dataset("backup")
            u.add_motive("ldap_pw_backup", "backup")
            u.datasets["backup"].password = "backup_pw"
            assert ldap.auth["password"] == "backup_pw"

            u.register_dataset("generic")
            u.add_motive("ldap", "generic")
            u.datasets["generic"].password = "generic_pw"
            assert ldap.auth["password"] == "generic_pw"

            u.register_dataset("primary")
            u.add_motive("ldap_pw", "primary")
            u.datasets["primary"].password = "primary_pw"
            assert ldap.auth["password"] == "primary_pw"

            assert ldap.auth == {
                "scheme": "simple_bind",
                "motives": motives,
                "username": cu.id,
                "password": "primary_pw"
            }

        @pytest.mark.skip
        def test_error_when_no_password_given_and_no_motive_allowed(self):
            with pytest.raises(RuntimeError, match="???"):
                ldap = om._origen_metal.utils.ldap.LDAP(
                    name="custom_motives",
                    base=BASE,
                    server=SERVER,
                    auth={
                        "allow_default_password": False,
                        "use_default_motives": False,
                    })

        def test_custom_motives_only(self, unload_users, users, u, cu):
            # Auth and username given, but no password, looks up user with same password motives as before
            # Mimics how a service user may be used
            su = users.add("mimic_service_user")
            su.register_dataset("for_ldap")
            su.add_motive("ldap_name_only", "for_ldap")
            su.datasets["for_ldap"].password = "service_user_pw"
            ldap = om._origen_metal.utils.ldap.LDAP(
                name="su_ldap",
                base=BASE,
                server=SERVER,
                auth={
                    "username": su.id,
                    # TODO accept a string here? "priority_motives": "ldap_name_only"
                    "priority_motives": ["ldap_name_only"],
                    "allow_default_password": False,
                    "use_default_motives": False,
                })
            assert ldap.auth == {
                "scheme": "simple_bind",
                "motives": ["ldap_name_only"],
                "username": "mimic_service_user",
                "password": "service_user_pw"
            }
            assert cu.id != su.id

        def test_given_password_without_username(self, cu):
            # Auth and password given but no username assumes current user
            # Can be used a hard-code/shared/common password without needing to explicitly add a dataset
            ldap = om._origen_metal.utils.ldap.LDAP(name="ldap_static_pw",
                                                    base=BASE,
                                                    server=SERVER,
                                                    auth={
                                                        "password":
                                                        "static_pw",
                                                    })
            assert ldap.auth == {
                "scheme": "simple_bind",
                "motives": ["ldap_static_pw", "ldap"],
                "username": cu.id,
                "password": "static_pw"
            }

            cu.register_dataset("generic")
            cu.add_motive("ldap", "generic")
            cu.datasets["generic"].password = "generic_pw"
            assert cu.password_for("ldap") == "generic_pw"
            assert ldap.auth["password"] == "static_pw"

        def test_given_username_and_password(self):
            ''' A user does not need to exists if both username and password are given'''
            ldap = om._origen_metal.utils.ldap.LDAP(
                name="ldap_static_user_and_pw",
                base=BASE,
                server=SERVER,
                auth={
                    "username": "static_user",
                    "password": "static_pw",
                })
            assert ldap.auth == {
                "scheme": "simple_bind",
                "motives": ["ldap_static_user_and_pw", "ldap"],
                "username": "static_user",
                "password": "static_pw"
            }

        def test_given_user_does_not_exists(self):
            static_n = "static_user"
            ldap = om._origen_metal.utils.ldap.LDAP(
                name="ldap_static_user_and_pw",
                base=BASE,
                server=SERVER,
                auth={
                    "username": static_n,
                })
            with pytest.raises(RuntimeError,
                               match=f"No user '{static_n}' has been added"):
                ldap.auth
