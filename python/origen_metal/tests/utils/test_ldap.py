# TODO need to clean this up

import pytest, copy
import origen_metal as om
from origen_metal.utils.ldap import LDAP
from origen_metal.frontend import DataStoreView
from tests.framework.users.shared import unload_users, users

# FORUMSYS LDAP
# https://www.forumsys.com/2022/05/10/online-ldap-test-server/
# Started with this one, but after using it for a bit, server started to have various timeout
# issues, eventually leading to just an inability to connect, despite other LDAPs working.
# Switched to the ZFLEX LLDAP but leaving the parameterized setup here in case its needed.
# Note: this setup was refactored due to the switch to ZFLEX, so the parameters have not been/cannot be tested.
FORUMSYS = {
    "name": "forumsys",
    "server": "ldap://ldap.forumsys.com:389",
    "base": "dc=example,dc=com",
    "auth_config": {
        "scheme": "simple_bind",
        "username": "cn=read-only-admin,dc=example,dc=com",
        "password": "zflexpass",
    },
    "dn_prefix": None,
    "populate_user_config": {
        "data_id": "uid",
        "mapping": {
            "email": "mail",
            "last_name": "sn",
            "full_name": "cn"
        }
    },
    "users": {
        "euler": {
            "fields": {
                'cn': ['Leonhard Euler'],
                'sn': ['Euler'],
                'uid': ['euler'],
                'objectClass': ['inetOrgPerson', 'organizationalPerson', 'person', 'top'],
                'mail': ['euler@ldap.forumsys.com']
            },
            "password": "password",
        },
        "curie": {
            "fields": {
                'mail': ['curie@ldap.forumsys.com'],
                'cn': ['Marie Curie']
            }
        }
    }
}

# ZFLEX test ldap
# https://www.zflexldapadministrator.com/index.php/component/content/article?id=82:free-online-ldap
ZFLEX = {
    "name": "zflex",
    "server": "ldap://zflexldap.com:389",
    "base": "dc=zflexsoftware,dc=com",
    "auth_config": {
        "scheme": "simple_bind",
        "username": "cn=ro_admin,ou=sysadmins,dc=zflexsoftware,dc=com",
        "password": "zflexpass",
    },
    "dn_prefix": "ou=users,ou=guests",
    "populate_user_config": {
        "data_id": "uid",
        "mapping": {
            "email": "mail",
            "last_name": "sn",
            "full_name": "cn"
        }
    },
    "users": {
        "guest1": {
            "fields": {
                'title': ['Contractor1'],
                'mail': ['guest1@zflexsoftware.com'],
                'facsimileTelephoneNumber': ['330-333-3342'],
                'employeetype': ['temp'],
                'employeeNumber': ['11003'],
                'cn': ['Guest Number One'],
                'l': ['Boston'],
                'mobile': ['909-983-4552'],
                'objectClass': ['top', 'person', 'organizationalPerson', 'inetOrgPerson'],
                'givenName': ['Guest'],
                'pager': ['303-223-9876'],
                'displayname': ['Guest1 NumberOne'],
                'uid': ['guest1'],
                'street': ['403 Anywhere Lane'],
                'postalCode': ['30994'],
                'postalAddress': ['3088 NewMain Street'],
                'departmentNumber': ['0001'],
                'sn': ['Number One']
            },
            "password": "guest1password",
        },
        "guest2": {
            "fields": {
                'mail': ['guest2@zflexsoftware.com'],
                'sn': ['NumberTwo'],
                'cn': ['guest2 NumberTwo'],
            }
        }
    }
}

class Common:
    def dummy_ldap(self, name=None, timeout=5, continuous_bind=False, populate_user_config=False):
        if populate_user_config is True:
            pop_config = self.config["populate_user_config"]
        elif populate_user_config is False:
            pop_config = None
        else:
            pop_config = populate_user_config

        return om._origen_metal.utils.ldap.LDAP(
            name=(name or self.config["name"]),
            server=self.config["server"],
            base=self.config["base"],
            auth=self.config["auth_config"],
            timeout=timeout,
            continuous_bind=continuous_bind,
            populate_user_config=pop_config,
        )

    @property
    def ldap_class(self):
        return LDAP

    class DummyLDAPConfig:
        class User:
            def __init__(self, id, parent):
                self.id = id
                self.parent = parent

            @property
            def fields(self):
                return self.parent["users"][self.id]["fields"]

            def __getattr__(self, name):
                if name in self.fields:
                    f = self.fields[name]
                    return f[0] if len(f) == 1 else f
                else:
                    return object.__getattribute__(self, name)

            @property
            def qualified_id(self):
                return f"uid={self.id}{(',' + self.parent.dn_prefix) if self.parent.dn_prefix else ''},{self.parent.base}"

            @property
            def password(self):
                return self.parent["users"][self.id]["password"]

        def __init__(self, config):
            self.config = config
            self._users = []
            for n, c in config["users"].items():
                self._users.append(self.User(n, self))

        def as_params_list(self):
            return [
                self.server,
                self.base,
                self.auth_config,
                False,
                self.populate_user_config,
                5,
            ]

        @property
        def server(self):
            return self.config["server"]

        @property
        def base(self):
            return self.config["base"]

        @property
        def dn_prefix(self):
            return self.config["dn_prefix"]

        @property
        def name(self):
            return self.config["name"]

        @property
        def auth_config(self):
            return self.config["auth_config"]

        @property
        def auth_scheme(self):
            return self.config["auth_config"]["scheme"]

        @property
        def auth_username(self):
            return self.config["auth_config"]["username"]

        @property
        def auth_password(self):
            return self.config["auth_config"]["password"]

        @property
        def populate_user_config(self):
            return self.config["populate_user_config"]

        @property
        def users(self):
            return self._users

        @property
        def u1(self):
            return self._users[0]

        @property
        def u2(self):
            return self._users[1]

        def __getitem__(self, key):
            return self.config[key]

    @classmethod
    def get_dummy_config(cls):
        return cls.DummyLDAPConfig(ZFLEX)

    @property
    def config(self):
        if not hasattr(self, "_config"):
            self._config = self.get_dummy_config()
        return self._config

    @pytest.fixture
    def dummy_config(self):
        return self.config

    @pytest.fixture
    def u1(self, dummy_config):
        return dummy_config.u1

    @pytest.fixture
    def u2(self, dummy_config):
        return dummy_config.u2

@pytest.mark.ldap
class TestStandaloneLDAP(Common):
    def test_ldap_parameters(self, dummy_config):
        ldap = self.dummy_ldap()
        assert ldap.base == dummy_config.base
        assert ldap.server == dummy_config.server
        assert ldap.name == dummy_config.name
        assert ldap.bound == False
        assert ldap.auth_config == {
            'scheme': dummy_config.auth_scheme,
            'username': dummy_config.auth_username,
            'password': dummy_config.auth_password,
            'allow_default_password': True,
            'use_default_motives': True,
            'priority_motives': [],
            'backup_motives': [],
        }
        assert ldap.continuous_bind == False
        assert ldap.timeout == 5
        assert ldap.populate_user_config == None

    def test_ldap_minimum_parameters(self, dummy_config):
        ldap = om._origen_metal.utils.ldap.LDAP(
            name="min",
            base=dummy_config.base,
            server=dummy_config.server,
        )
        assert ldap.name == "min"
        assert ldap.base == dummy_config.base
        assert ldap.server == dummy_config.server
        assert ldap.auth_config == {
            'scheme': dummy_config.auth_scheme,
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
        ldap = self.dummy_ldap(continuous_bind=True)
        assert ldap.continuous_bind == True
        assert ldap.bind()
        assert ldap.bound == True

        # If continuous bind is disabled, ldap.bound should reflect this
        # 'ldap.bind' in this context is more like a 'try to bind' than an actual bind
        ldap = self.dummy_ldap()
        assert ldap.continuous_bind == False
        assert ldap.bind()
        assert ldap.bound == False

    def test_ldap_timeout_settings(self, dummy_config):
        # Default timeout should be 60
        ldap = om._origen_metal.utils.ldap.LDAP(
            name=dummy_config.name,
            base=dummy_config.base,
            server=dummy_config.server,
            continuous_bind=False,
        )
        assert ldap.timeout == 60

        # Using 'False' should result in no timeout (will wait indefinitely)
        # This is returned as a "None"
        ldap = self.dummy_ldap(timeout=False)
        assert ldap.timeout == None

        # Likewise, "True" will just apply the default
        ldap = self.dummy_ldap(timeout=True)
        assert ldap.timeout == 60

        # Otherwise, timeout option should be used
        ldap = self.dummy_ldap()
        assert ldap.timeout == 5

    def test_ldap_searching(self, u1, u2):
        ldap = self.dummy_ldap()
        results = ldap.search(f"(uid={u1.id})", [])
        assert results == {
            u1.qualified_id: (u1.fields, {})
        }
        results = ldap.search(f"(|(uid={u1.id})(uid={u2.id}))", ["cn", "mail"])
        assert results == {
            u1.qualified_id: ({
                'cn': [u1.cn],
                'mail': [u1.mail]
            }, {}),
            u2.qualified_id: ({
                'mail': [u2.mail],
                'cn': [u2.cn]
            }, {})
        }

        results = ldap.search(f"(|(uid={u1.id})(uid={u2.id}))", ["BLAH"])
        assert results == {
            u1.qualified_id: ({}, {}),
            u2.qualified_id: ({}, {})
        }
        results = ldap.search("(|(uid=blah)(uid=none))", ["BLAH"])
        assert results == {}


    def test_single_filter_search(self, u1):
        ldap = self.dummy_ldap()
        results = ldap.single_filter_search(f"(uid={u1.id})", ["cn", "mail"])
        assert results == ({
            'mail': [u1.mail],
            'cn': [u1.cn]
        }, {})
        results = ldap.single_filter_search("(uid=blah)", ["cn", "mail"])
        assert results == ({}, {})

    def test_error_if_single_filter_search_returns_multiple_dns(self, u1, u2):
        ldap = self.dummy_ldap()
        with pytest.raises(RuntimeError,
                           match="expected a single DN result from filter"):
            ldap.single_filter_search(f"(|(uid={u1.id})(uid={u2.id}))", ["mail"])

    def test_unbind_and_rebind(self):
        ldap = self.dummy_ldap(continuous_bind=True)
        assert ldap.bind()
        assert ldap.bound
        assert ldap.unbind()
        assert not ldap.bound
        assert ldap.bind()
        assert ldap.bound

        ldap = self.dummy_ldap()
        assert ldap.bind()
        assert ldap.bound == False
        assert ldap.unbind() == False

    def test_validating_passwords(self, dummy_config):
        ldap = self.dummy_ldap(continuous_bind=True)
        assert ldap.bind()
        assert ldap.bound == True
        assert ldap.validate_credentials(dummy_config.users[0].qualified_id, dummy_config.users[0].password)
        assert not ldap.validate_credentials(dummy_config.users[0].qualified_id, "?")
        # Should not effect the current LDAP
        assert ldap.bound == True

    def test_populate_user_config(self):
        ldap = self.dummy_ldap(populate_user_config=True)
        assert ldap.populate_user_config == {
            "data_id": "uid",
            "mapping": {
                "email": "mail",
                "last_name": "sn",
                "full_name": "cn"
            }
        }
    
    def test_timeout_can_be_set(self):
        ldap = self.dummy_ldap(populate_user_config=True)
        assert ldap.timeout == 5
        ldap.timeout = 10
        assert ldap.timeout == 10
        ldap.timeout = 0
        assert ldap.timeout == 0
        ldap.timeout = None
        assert ldap.timeout is None

@pytest.mark.ldap
class TestLdapAsDataStore(DataStoreView):
    ''' The LDAP's only data store feature is populating users'''
    def parameterize(self):
        config = Common.get_dummy_config()
        return {
            "init_args": [
                self.ds_test_name,
                config.server,
                config.base,
                config.auth_config,
            ],
        }

    @property
    def data_store_class(self):
        return LDAP

    def test_underlying_ldap_search_works(self):
        u = Common.get_dummy_config().u1
        results = self.ds.single_filter_search(f"(uid={u.id})", ["cn", "mail"])
        assert results == ({
            'mail': [u.mail],
            'cn': [u.cn]
        }, {})


@pytest.mark.ldap
class TestAuthSetups:
    class TestSimpleBind:
        @pytest.fixture
        def min_auth(self):
            config = Common.get_dummy_config()
            return om._origen_metal.utils.ldap.LDAP(
                name="min",
                base=config.base,
                server=config.server,
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
            config = Common.get_dummy_config()
            u.password = "top_pwd"
            # No auth given but username and password provided assumes 'simple bind'
            ldap = om._origen_metal.utils.ldap.LDAP(
                name="min",
                base=config.base,
                server=config.server,
            )
            auth = ldap.auth
            assert auth["scheme"] == "simple_bind"
            assert auth["username"] == cu.id
            assert auth["motives"] == ["min", "ldap"]

            assert ldap.auth_config == {
                'scheme': config.auth_scheme,
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
            config = Common.get_dummy_config()
            ldap = om._origen_metal.utils.ldap.LDAP(
                name="custom_motives",
                base=config.base,
                server=config.server,
                auth={
                    "priority_motives": ["ldap_pw"],
                    "backup_motives": ["ldap_pw_backup"],
                    "allow_default_password": False,
                })
            assert ldap.auth_config == {
                'scheme': config.auth_scheme,
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
            config = Common.get_dummy_config()
            with pytest.raises(RuntimeError, match="???"):
                ldap = om._origen_metal.utils.ldap.LDAP(
                    name="custom_motives",
                    base=config.base,
                    server=config.server,
                    auth={
                        "allow_default_password": False,
                        "use_default_motives": False,
                    })

        def test_custom_motives_only(self, unload_users, users, u, cu):
            # Auth and username given, but no password, looks up user with same password motives as before
            # Mimics how a service user may be used
            config = Common.get_dummy_config()
            su = users.add("mimic_service_user")
            su.register_dataset("for_ldap")
            su.add_motive("ldap_name_only", "for_ldap")
            su.datasets["for_ldap"].password = "service_user_pw"
            ldap = om._origen_metal.utils.ldap.LDAP(
                name="su_ldap",
                base=config.base,
                server=config.server,
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
            config = Common.get_dummy_config()
            ldap = om._origen_metal.utils.ldap.LDAP(name="ldap_static_pw",
                                                    base=config.base,
                                                    server=config.server,
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
            config = Common.get_dummy_config()
            ldap = om._origen_metal.utils.ldap.LDAP(
                name="ldap_static_user_and_pw",
                base=config.base,
                server=config.server,
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
            config = Common.get_dummy_config()
            ldap = om._origen_metal.utils.ldap.LDAP(
                name="ldap_static_user_and_pw",
                base=config.base,
                server=config.server,
                auth={
                    "username": static_n,
                })
            with pytest.raises(RuntimeError,
                               match=f"No user '{static_n}' has been added"):
                ldap.auth
