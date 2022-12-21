'''
No routing to an external SMTP server, so just checking the setup, configuration options, etc.
'''

import pytest, re
from pathlib import Path
from origen_metal.utils.mailer import Mailer
from tests.framework.users.shared import Base as UsersBase
from tests.framework.users.shared import no_current_user_error_msg
from tests.shared.python_like_apis import Fixture_DictLikeAPI
import origen_metal as om

class Common(UsersBase):
    mailer_class = Mailer
    ml_class = om.utils.mailer.Maillist
    maillists_class = om.utils.mailer.Maillists
    mls_class = maillists_class

    ml_root = Path(__file__).parent.joinpath("maillists")
    mls_dir = ml_root.joinpath("mls")
    dev_ml_toml = mls_dir.joinpath("dev.maillist.toml")
    inner_mls_dir = mls_dir.joinpath("inner")
    custom_mls_dir = ml_root.joinpath("custom")
    custom1_ml = custom_mls_dir.joinpath("custom1.maillist")
    custom2_ml_toml = custom_mls_dir.joinpath("custom2.maillist.toml")
    invalid_toml_mls_dir = ml_root.joinpath("errors/invalid_toml")
    invalid_ml_toml1 = invalid_toml_mls_dir.joinpath("develop.maillist.toml")
    invalid_ml_toml2 = invalid_toml_mls_dir.joinpath("invalid_toml.maillist.toml")
    empty_ml_mls_dir = ml_root.joinpath("empty")
    empty_ml = empty_ml_mls_dir.joinpath("empty.maillist")
    empty_ml_toml =  empty_ml_mls_dir.joinpath("empty_toml.maillist.toml")
    empty_ml_dir = empty_ml_mls_dir.joinpath("empty")
    missing_ml_dir = ml_root.joinpath("missing")
    missing_ml = ml_root.joinpath("mia.maillist")
    missing_ml_toml = ml_root.joinpath("mia.maillist.toml")
    invalid_ml_ext = ml_root.joinpath("errors/empty.maillist.err")
    conflicting_aud_dev_ml_toml = ml_root.joinpath("errors/conflicting_audiences/dev.maillist.toml")
    conflicting_aud_develop_ml_toml = ml_root.joinpath("errors/conflicting_audiences/develop.maillist.toml")
    conflicting_aud_prod_ml_toml = ml_root.joinpath("errors/conflicting_audiences/prod.maillist.toml")
    redundant_mls_dir = ml_root.joinpath("redundant_audience")
    redundant_aud_dev_ml_toml = redundant_mls_dir.joinpath("development.maillist.toml")
    redundant_aud_rel_ml_toml = redundant_mls_dir.joinpath("release.maillist.toml")
    override_mls_dir = ml_root.joinpath("override")
    dev_ml_toml_override = override_mls_dir.joinpath("dev.maillist.toml")
    custom2_ml_override = override_mls_dir.joinpath("custom2.maillist")
    ext_overwrite_mls_dir = ml_root.joinpath("ext_overwrite")
    ext_overwrite_ml = ext_overwrite_mls_dir.joinpath("ext_overwrite.maillist")
    ext_overwrite_ml_toml = ext_overwrite_mls_dir.joinpath("ext_overwrite.maillist.toml")

    password_motive_str = "mailer"

    dummy_server = "dummy.smtp.origen.org"
    dummy_min_server = "min.smtp.origen.org"

    @classmethod
    def new_minimum_mailer(cls):
        return cls.mailer_class(server=cls.dummy_min_server)

    def new_ml(cls, *args, **kwargs):
        return cls.ml_class(*args, **kwargs)

    @classmethod
    def new_mls(cls, *args, **kwargs):
        return cls.mls_class(*args, **kwargs)

    @pytest.fixture
    def d_server(self):
        return self.dummy_server

    @staticmethod
    def invalid_auth_err(auth):
        return f"Invalid auth method '{auth}' found in the mailer configuration"

    @staticmethod
    def missing_mls_err_msg(f):
        return f"Cannot find maillist path '.*{f.name}'"

    @staticmethod
    def invalid_toml_err_msg(f):
        n = f.name.split('.')[0]
        return f"Unable to build maillist from '.*invalid_toml.*{n}.maillist.toml'. Encountered errors: unexpected eof encountered"

    @staticmethod
    def invalid_file_ext_err_msg(f):
        return f"Unsupported file extension for maillist '.*{f.name}'"

    @staticmethod
    def conflicting_aud_err_msg(f, given_aud, mapped_given_aud, mapped_aud):
        if isinstance(f, str):
            if mapped_given_aud:
                return f"Maillist '{f}' was given audience '{given_aud}' \\(maps to '{mapped_given_aud}'\\) but conflicts with the named audience '{mapped_aud}'."
            else:
                return f"Maillist '{f}' was given audience '{given_aud}' but conflicts with the named audience '{mapped_aud}'."
        else:
            return f"Maillist at '.*{f.name.split('.')[0]}.maillist.*' was given audience '{given_aud}' \\(maps to '{mapped_given_aud}'\\) but conflicts with the named audience '{mapped_aud}'."

    @staticmethod
    def missing_ml_err_msg(f):
        n = f.name.split('.')[0]
        return f"Unable to find maillist at: '.*{n}.maillist'"

    @staticmethod
    def missing_ml_toml_err_msg(f):
        n = f.name.split('.')[0]
        return f"Unable to build maillist from '.*{n}.maillist.toml'. Encountered errors: configuration file .*{n}.maillist.toml\" not found"

class TestMailer(Common):
    def test_initializing_minimum_mailer(self, unload_users):
        min_server = "minimum.smtp.origen.org"
        m = self.mailer_class(server=min_server)
        assert isinstance(m, self.mailer_class)

        assert m.server == min_server
        assert m.port == None
        assert m.auth_method == None
        assert m.domain == None
        assert m.timeout == 60
        assert m.__user__ == None
        assert m.user == None
        with pytest.raises(RuntimeError, match=no_current_user_error_msg):
            assert m.username == None
        with pytest.raises(RuntimeError, match=no_current_user_error_msg):
            assert m.password == None
        with pytest.raises(RuntimeError, match=no_current_user_error_msg):
            assert m.dataset == None
        with pytest.raises(RuntimeError, match=no_current_user_error_msg):
            assert m.sender == None
        assert m.config == {
            "server": min_server,
            "port": None,
            "auth_method": None,
            "domain": None,
            "timeout": 60,
            "user": None,
        }

    def test_minimum_with_current_user(self, unload_users, users, u):
        users.set_current_user(u)
        u.password = "pwd"

        m = self.new_minimum_mailer()
        assert m.user == u
        assert m.__user__ == None
        assert m.username == u.id
        assert m.password == "pwd"
        assert m.dataset is None
        assert m.auth_method is None

        # Should get an error that the current user does not have an email set
        with pytest.raises(RuntimeError, match=self.missing_email_error_msg(u)):
            m.sender

        # Set the email and try again
        e = "u@origen.org"
        u.email = e
        assert m.sender == e

        assert m.config == {
            'server': self.dummy_min_server,
            'port': None,
            'auth_method': None,
            'domain': None,
            'timeout': 60,
            'user': None,
        }


    def test_initializing_mailer_with_options(self, unload_users):
        server = "with.options.origen.org"
        port = 25
        domain = "origen.org"
        timeout = 120
        auth_method = "TLS"
        m = self.mailer_class(server=server, port=port, domain=domain, timeout=timeout, auth_method=auth_method)

        assert m.server == server
        assert m.port == port
        assert m.auth_method == auth_method
        assert m.domain == domain
        assert m.timeout == timeout
        assert m.__user__ == None
        assert m.user == None
        assert m.config == {
            'server': server,
            'port': port,
            'auth_method': auth_method,
            'domain': domain,
            'timeout': 120,
            'user': None,
        }

    def test_tying_mailer_to_a_user(self, d_server, unload_users, users, u, u2):
        assert users.current is None
        u.password = "pw"
        u2.password = "u2_pw"

        m = self.mailer_class(server=d_server, user=u2.id)
        assert m.user == u2
        assert m.__user__ == u2.id
        assert m.username == u2.id
        assert m.password == "u2_pw"
        assert m.dataset is None
        assert m.config == {
            'server': d_server,
            'port': None,
            'auth_method': None,
            'domain': None,
            'timeout': 60,
            'user': u2.id,
        }

        users.set_current_user(u2)
        m = self.mailer_class(server=d_server, user=u.id)
        assert m.user == u
        assert m.__user__ == u.id
        assert m.username == u.id
        assert m.password == "pw"
        assert m.dataset is None
        assert m.config == {
            'server': d_server,
            'port': None,
            'auth_method': None,
            'domain': None,
            'timeout': 60,
            'user': u.id,
        }

    def test_tying_mailer_to_a_future_user(self, d_server, unload_users, users, u):
        fun = "future_user_name"
        assert users.current is None
        m = self.mailer_class(server=d_server, user=fun)
        assert m.__user__ == fun
        err_msg = f"No user '{fun}' has been added"
        with pytest.raises(RuntimeError, match=err_msg):
            assert m.user
        with pytest.raises(RuntimeError, match=err_msg):
            assert m.username
        with pytest.raises(RuntimeError, match=err_msg):
            assert m.password
        with pytest.raises(RuntimeError, match=err_msg):
            assert m.dataset
        assert m.config == {
            'server': d_server,
            'port': None,
            'auth_method': None,
            'domain': None,
            'timeout': 60,
            'user': fun,
        }

        fu = users.add(fun)
        fu.password = "fu_pw"
        assert m.user == fu
        assert m.__user__ == fun
        assert m.username == fun
        assert m.password == "fu_pw"
        assert m.dataset is None

    def test_mailer_password_motive_consistency(self):
        ''' If this changes in the backend, likely frontend documentation will need updating'''
        assert self.mailer_class.PASSWORD_MOTIVE == self.password_motive_str

    def test_tying_mailer_to_a_user_motive(self, d_server, unload_users, users, u):
        m = self.mailer_class(server=d_server)

        users.set_current_user(u)
        ds_nm = u.add_dataset("not_mailer")
        ds_nm.password = "not_mailer_pw"
        u.data_lookup_hierarchy = ["not_mailer"]
        assert u.password == "not_mailer_pw"
        assert m.username == u.id
        assert m.password == "not_mailer_pw"
        assert m.dataset is None

        ds_fm = u.add_dataset("for_mailer")
        ds_fm.password = "for_mailer_pw"

        u.add_motive(self.mailer_class.PASSWORD_MOTIVE, ds_fm)
        u.data_lookup_hierarchy = ["not_mailer", "for_mailer"]
        assert u.password == "not_mailer_pw"
        assert m.username == u.id
        assert m.password == "for_mailer_pw"
        assert m.dataset == ds_fm.dataset_name

        ds_fm.username = "for_mailer_un"
        assert m.username == "for_mailer_un"

        # TODO support custom motives?

    def test_invalid_auth_scheme(self, d_server):
        with pytest.raises(RuntimeError, match=self.invalid_auth_err("hi")):
            self.mailer_class(server=d_server, auth_method="hi")

    # def test_error_on_sending_email(self):
    #     fail

    # def test_test_email_body(self):
    #     fail
    
    # def test_test_email_recipients(self):
    #     fail

class TestMaillist(Common):
    def test_creating_minimal_maillist(self):
        n = "min_ml"
        ml = self.new_ml(n, [])
        assert isinstance(ml, self.ml_class)
        assert ml.name == n
        assert ml.recipients == []
        assert ml.signature is None
        assert ml.audience is None
        assert ml.domain is None
        assert ml.file is None
        assert ml.config == {
            "name": n,
            "recipients": [],
            "signature": None,
            "audience": None,
            "domain": None,
            "file": None,
        }
        assert ml.resolve_recipients() == []

    def test_creating_a_maillist_with_recipients(self):
        n = "ml_with_recipients"
        r = [
            "u0@origen.org",
            "u20@origen.org",
            "o0@o2.org",
        ]

        ml = self.new_ml(n, recipients=r)
        assert isinstance(ml, self.ml_class)
        assert ml.name == n
        assert ml.recipients == r
        assert ml.signature is None
        assert ml.audience is None
        assert ml.domain is None
        assert ml.file is None
        assert ml.config == {
            "name": n,
            "recipients": r,
            "signature": None,
            "audience": None,
            "domain": None,
            "file": None,
        }
        assert ml.resolve_recipients() == r

    def test_creating_a_maillist_with_options(self):
        n = "ml_with_opts"
        r = [
            "u0@origen.org",
            "u20@origen.org",
            "o0@o2.org",
        ]
        sig = "Sig from Origen!"
        aud = "dev"
        mapped_aud = "development"
        domain = "origen.org"

        ml = self.new_ml(n, recipients=r, signature=sig, audience=aud, domain=domain)
        assert isinstance(ml, self.ml_class)
        assert ml.name == n
        assert ml.recipients == r
        assert ml.signature == sig
        assert ml.audience == mapped_aud
        assert ml.domain == domain
        assert ml.file is None
        assert ml.config == {
            "name": n,
            "recipients": r,
            "signature": sig,
            "audience": mapped_aud,
            "domain": domain,
            "file": None,
        }
        assert ml.resolve_recipients() == r

    def test_appending_domain(self):
        r = [
            "u",
            "u2",
            "u3",
        ]
        domain = "origen.org"
        ml = self.new_ml("ml", recipients=r, domain=domain)
        assert ml.recipients == r
        assert ml.domain == domain
        assert ml.config["recipients"] == r
        assert ml.resolve_recipients() == [
            f"u@{domain}",
            f"u2@{domain}",
            f"u3@{domain}",
        ]

    def test_appending_domain_when_applicable(self):
        r = [
            "u@o.org",
            "u2",
            "u3@o3.com",
            "u4",
        ]
        domain = "origen.org"
        ml = self.new_ml("ml", recipients=r, domain=domain)
        assert ml.recipients == r
        assert ml.domain == domain
        assert ml.config["recipients"] == r
        assert ml.resolve_recipients() == [
            "u@o.org",
            f"u2@{domain}",
            "u3@o3.com",
            f"u4@{domain}",
        ]

    def test_invalid_recipient_resolution(self):
        invalid_email_err_msg = "Missing domain or user"
        invalid_domain_err_msg = "Invalid email domain"
        r = ["u", "u2"]
        ml = self.new_ml("ml", recipients=r)
        assert ml.recipients == r
        assert ml.domain is None
        with pytest.raises(RuntimeError, match=invalid_email_err_msg):
            ml.resolve_recipients()

        r = ["u", "u2@origen.org"]
        ml = self.new_ml("ml", recipients=r)
        assert ml.recipients == r
        assert ml.domain is None
        with pytest.raises(RuntimeError, match=invalid_email_err_msg):
            ml.resolve_recipients()

        r = ["u", "u2@origen.org"]
        domain= "blah!"
        ml = self.new_ml("ml", recipients=r, domain=domain)
        assert ml.recipients == r
        assert ml.domain == domain
        with pytest.raises(RuntimeError, match=invalid_domain_err_msg):
            ml.resolve_recipients()

        r = ["u@origen!", "u2@origen.org"]
        domain= "blah!"
        ml = self.new_ml("ml", recipients=r, domain=domain)
        assert ml.recipients == r
        assert ml.domain == domain
        with pytest.raises(RuntimeError, match=invalid_domain_err_msg):
            ml.resolve_recipients()

    def test_audience_resolution(self):
        ml = self.new_ml("t", [], audience="prod")
        assert ml.audience == "production"
        assert ml.is_production
        assert not ml.is_development
        ml = self.new_ml("t", [], audience="production")
        assert ml.audience == "production"
        assert ml.is_production
        assert not ml.is_development
        ml = self.new_ml("t", [], audience="release")
        assert ml.audience == "production"
        assert ml.is_production
        assert not ml.is_development

        ml = self.new_ml("t", [], audience="dev")
        assert ml.audience == "development"
        assert not ml.is_production
        assert ml.is_development
        ml = self.new_ml("t", [], audience="develop")
        assert ml.audience == "development"
        assert not ml.is_production
        assert ml.is_development
        ml = self.new_ml("t", [], audience="development")
        assert ml.audience == "development"
        assert not ml.is_production
        assert ml.is_development

        ml = self.new_ml("t", [], audience="Other")
        assert ml.audience == "Other"
        assert not ml.is_production
        assert not ml.is_development

    def test_audience_resolution_by_name(self):
        ml = self.new_ml("prod", [])
        assert ml.name == "prod"
        assert ml.audience == "production"
        ml = self.new_ml("production", [])
        assert ml.name == "production"
        assert ml.audience == "production"
        ml = self.new_ml("release", [])
        assert ml.name == "release"
        assert ml.audience == "production"

        ml = self.new_ml("dev", [])
        assert ml.name == "dev"
        assert ml.audience == "development"
        ml = self.new_ml("develop", [])
        assert ml.name == "develop"
        assert ml.audience == "development"
        ml = self.new_ml("development", [])
        assert ml.name == "development"
        assert ml.audience == "development"

    def test_conflicting_name_and_audience_resolution(self):
        n = "prod"
        with pytest.raises(RuntimeError, match=self.conflicting_aud_err_msg(n, "dev", "development", "production")):
            ml = self.new_ml(n, [], audience="dev")

        n = "release"
        with pytest.raises(RuntimeError, match=self.conflicting_aud_err_msg(n, "other", None, "production")):
            ml = self.new_ml(n, [], audience="other")

    def test_maillist_from_file(self):
        f = self.custom1_ml
        ml = self.ml_class.from_file(f)

        r = ["u1@custom1.org", "u2@custom2.org"]
        assert isinstance(ml, self.ml_class)
        assert ml.name == "custom1"
        assert ml.recipients == r
        assert ml.signature is None
        assert ml.audience is None
        assert ml.domain is None
        assert ml.file == f
        assert isinstance(ml.file, Path)
        assert ml.config == {
            "name": "custom1",
            "recipients": r,
            "signature": None,
            "audience": None,
            "domain": None,
            "file": f,
        }
        assert isinstance(ml.config["file"], Path)
        assert ml.resolve_recipients() == r

        # str should also be acceptable
        ml = self.ml_class.from_file(str(f))
        assert isinstance(ml, self.ml_class)
        assert ml.name == "custom1"
        assert ml.recipients == r
        assert ml.signature is None
        assert ml.audience is None
        assert ml.domain is None
        assert ml.file == f
        assert isinstance(ml.file, Path)
        assert ml.config == {
            "name": "custom1",
            "recipients": r,
            "signature": None,
            "audience": None,
            "domain": None,
            "file": f,
        }
        assert isinstance(ml.config["file"], Path)
        assert ml.resolve_recipients() == r

    def test_maillist_from_toml_file(self):
        f = self.custom2_ml_toml
        ml = self.ml_class.from_file(f)

        r = ["u1", "u2"]
        domain = "custom2.org"
        assert isinstance(ml, self.ml_class)
        assert ml.name == "custom2"
        assert ml.recipients == r
        assert ml.signature is None
        assert ml.audience == "development"
        assert ml.domain == domain
        assert ml.file == f
        assert isinstance(ml.file, Path)
        assert ml.config == {
            "name": "custom2",
            "recipients": r,
            "signature": None,
            "audience": "development",
            "domain": domain,
            "file": f,
        }
        assert isinstance(ml.config["file"], Path)
        assert ml.resolve_recipients() == [f"u1@{domain}", f"u2@{domain}"]

    def test_invalid_toml(self):
        f = self.invalid_ml_toml1
        with pytest.raises(RuntimeError, match=self.invalid_toml_err_msg(f)):
            ml = self.ml_class.from_file(f)

        f = self.invalid_ml_toml2
        with pytest.raises(RuntimeError, match=self.invalid_toml_err_msg(f)):
            ml = self.ml_class.from_file(f)

    def test_empty_maillist(self):
        f = self.empty_ml
        ml = self.ml_class.from_file(f)
        assert ml.config == {
            "name": "empty",
            "recipients": [],
            "signature": None,
            "audience": None,
            "domain": None,
            "file": f,
        }

        f = self.empty_ml_toml
        ml = self.ml_class.from_file(f)
        assert ml.config == {
            "name": "empty_toml",
            "recipients": [],
            "signature": None,
            "audience": None,
            "domain": None,
            "file": f,
        }

    def test_missing_file(self):
        f = self.missing_ml
        with pytest.raises(RuntimeError, match=self.missing_ml_err_msg(f)):
            ml = self.ml_class.from_file(f)

        f = self.missing_ml_toml
        with pytest.raises(RuntimeError, match=self.missing_ml_toml_err_msg(f)):
            ml = self.ml_class.from_file(f)

    def test_invalid_file_ext(self):
        f = self.invalid_ml_ext
        with pytest.raises(RuntimeError, match=self.invalid_file_ext_err_msg(f)):
            ml = self.ml_class.from_file(f)

    def test_conflicting_audience(self):
        f = self.conflicting_aud_dev_ml_toml
        with pytest.raises(RuntimeError, match=self.conflicting_aud_err_msg(f, "other", "other", "development")):
            ml = self.ml_class.from_file(f)

        f = self.conflicting_aud_develop_ml_toml
        with pytest.raises(RuntimeError, match=self.conflicting_aud_err_msg(f, "prod", "production", "development")):
            ml = self.ml_class.from_file(f)

        f = self.conflicting_aud_prod_ml_toml
        with pytest.raises(RuntimeError, match=self.conflicting_aud_err_msg(f, "development", "development", "production")):
            ml = self.ml_class.from_file(f)

    def test_redundant_audience(self):
        f = self.redundant_aud_dev_ml_toml
        ml = self.ml_class.from_file(f)
        assert ml.config == {
            "name": "development",
            "recipients": ["u1_dev@redundant.org"],
            "signature": None,
            "audience": "development",
            "domain": None,
            "file": f,
        }

        f = self.redundant_aud_rel_ml_toml
        ml = self.ml_class.from_file(f)
        assert ml.config == {
            "name": "release",
            "recipients": ["u1_release@redundant.org"],
            "signature": None,
            "audience": "production",
            "domain": None,
            "file": f,
        }

class TestMaillists(Common):
    def test_empty_maillists(self):
        n = "mls_test"
        mls = self.new_mls(n)
        assert isinstance(mls, self.mls_class)
        assert mls.name == n
        assert len(mls) == 0
        assert mls.maillists == {}
        assert mls.directories == []
        assert mls.production_maillists == {}
        assert mls.development_maillists == {}

    def test_smaller_maillists(self):
        n = "mls_test"
        mls = self.new_mls(n, self.custom_mls_dir)
        assert mls.name == n
        assert len(mls) == 2
        assert list(mls.keys()) == ["custom1", "custom2"]
        assert list(mls.maillists.keys()) == ["custom1", "custom2"]
        assert isinstance(mls.maillists["custom1"], self.ml_class)
        assert isinstance(mls.maillists["custom2"], self.ml_class)
        assert mls.directories == [self.custom_mls_dir]
        assert mls.production_maillists == {}
        assert list(mls.development_maillists.keys()) == ["custom2"]
        assert isinstance(mls.development_maillists["custom2"], self.ml_class)

        # Should also work with str
        mls = self.new_mls(n, str(self.custom_mls_dir))
        assert mls.name == n
        assert len(mls) == 2
        assert list(mls.keys()) == ["custom1", "custom2"]
        assert list(mls.maillists.keys()) == ["custom1", "custom2"]
        assert isinstance(mls.maillists["custom1"], self.ml_class)
        assert isinstance(mls.maillists["custom2"], self.ml_class)
        assert mls.directories == [self.custom_mls_dir]
        assert mls.production_maillists == {}
        assert list(mls.development_maillists.keys()) == ["custom2"]
        assert isinstance(mls.development_maillists["custom2"], self.ml_class)
    
    def test_larger_maillists(self):
        n = "mls_larger_test"
        mls = self.new_mls(n, self.mls_dir, self.custom_mls_dir, str(self.empty_ml_mls_dir), self.redundant_mls_dir)
        assert mls.name == n
        assert len(mls) == 9
        assert list(mls.keys()) == [
            "production", "dev", "weekly",
            "custom1", "custom2",
            "empty", "empty_toml",
            "development", "release"
        ]
        assert mls.directories == [
            self.mls_dir,
            self.custom_mls_dir,
            self.empty_ml_mls_dir,
            self.redundant_mls_dir
        ]
        assert list(mls.production_maillists.keys()) == ["production", "release"]
        assert list(mls.development_maillists.keys()) == ["dev", "custom2", "development"]

    def test_maillists_dirs_are_not_searched_recursively(self):
        n = "mls_top"
        mls = self.new_mls(n, self.mls_dir)
        assert list(mls.keys()) == ["production", "dev", "weekly"]
        assert mls.directories == [self.mls_dir]

        n = "mls_top_and_inner"
        mls = self.new_mls(n, self.mls_dir, self.inner_mls_dir)
        assert list(mls.keys()) == ["production", "dev", "weekly", "example"]
        assert mls.directories == [self.mls_dir, self.inner_mls_dir]

    def test_empty_mls_dir(self):
        mls = self.new_mls("empty_mls_dir", self.empty_ml_dir)
        assert len(mls) == 0
        assert list(mls.keys()) == []
        assert mls.directories == [self.empty_ml_dir]

    def test_audience_filtering(self):
        mls = self.new_mls("aud_test", self.mls_dir, self.custom_mls_dir)
        assert list(mls.keys()) == ["production", "dev", "weekly", "custom1", "custom2"]
        prod_mls = ["production"]
        dev_mls = ["dev", "custom2"]
        weekly_mls = ["weekly"]

        assert list(mls.production_maillists) == prod_mls
        assert list(mls.prod_maillists) == prod_mls
        assert list(mls.release_maillists) == prod_mls
        assert list(mls.maillists_for("prod")) == prod_mls
        assert list(mls.maillists_for("production")) == prod_mls
        assert list(mls.maillists_for("release")) == prod_mls

        assert list(mls.dev_maillists) == dev_mls
        assert list(mls.develop_maillists) == dev_mls
        assert list(mls.development_maillists) == dev_mls
        assert list(mls.maillists_for("dev")) == dev_mls
        assert list(mls.maillists_for("develop")) == dev_mls
        assert list(mls.maillists_for("development")) == dev_mls

        assert list(mls.maillists_for("weekly")) == weekly_mls
        assert mls.maillists_for("nothing") == {}

    def test_overriding_maillists(self):
        mls = self.new_mls("aud_test", self.mls_dir, self.custom_mls_dir)
        assert list(mls.keys()) == ["production", "dev", "weekly", "custom1", "custom2"]
        ml = mls["dev"]
        assert ml.resolve_recipients() == ["dev@test_apps.origen.org"]
        assert ml.file == self.dev_ml_toml
        ml = mls["custom2"]
        assert ml.resolve_recipients() == ["u1@custom2.org", "u2@custom2.org"]
        assert ml.file == self.custom2_ml_toml

        mls = self.new_mls("aud_test", self.mls_dir, self.custom_mls_dir, self.override_mls_dir)
        assert list(mls.keys()) == ["production", "dev", "weekly", "custom1", "custom2"]
        ml = mls["dev"]
        assert ml.resolve_recipients() == ["u1@override.origen.org", "u2@override.origen.org"]
        assert ml.file == self.dev_ml_toml_override
        ml = mls["custom2"]
        assert ml.resolve_recipients() == ["u1@custom2.override.org", "u2@custom2.override.org"]
        assert ml.file == self.custom2_ml_override

    def test_maillist_toml_ext_overwrite_maillist_ext(self):
        ml_ex = self.ml_class.from_file(self.ext_overwrite_ml)
        ml_ex_toml= self.ml_class.from_file(self.ext_overwrite_ml_toml)
        assert ml_ex.resolve_recipients() == ["ext@origen.org"]
        assert ml_ex.file == self.ext_overwrite_ml
        assert ml_ex_toml.resolve_recipients() == ["ext@overwrite.origen.org"]
        assert ml_ex_toml.file == self.ext_overwrite_ml_toml
        assert ml_ex.file.parent == ml_ex_toml.file.parent

        mls = self.new_mls("ext_override_test", self.ext_overwrite_mls_dir)
        assert list(mls.keys()) == ["ext_overwrite"]
        ml = mls["ext_overwrite"]
        assert ml.name == "ext_overwrite"
        assert ml.resolve_recipients() == ["ext@overwrite.origen.org"]
        assert ml.file == self.ext_overwrite_ml_toml

    def test_error_importing_maillists(self, capfd):
        # Exception
        with pytest.raises(RuntimeError, match=self.invalid_toml_err_msg(self.invalid_ml_toml1)):
            self.new_mls("err", self.mls_dir, self.invalid_toml_mls_dir, self.custom_mls_dir)

        # Log fail
        mls = self.new_mls("err", self.mls_dir, self.invalid_toml_mls_dir, self.custom_mls_dir, continue_on_error=True)
        stdout = capfd.readouterr().out
        assert list(mls.keys()) == [
            "production", "dev", "weekly",
            "custom1", "custom2",
        ]
        assert re.search(self.invalid_toml_err_msg(self.invalid_ml_toml1), stdout)
        assert re.search(self.invalid_toml_err_msg(self.invalid_ml_toml2), stdout)
        assert mls.directories == [self.mls_dir, self.invalid_toml_mls_dir, self.custom_mls_dir]

    def test_missing_directories(self, capfd):
        # Exception
        with pytest.raises(RuntimeError, match=self.missing_mls_err_msg(self.missing_ml_dir)):
            self.new_mls("err", self.missing_ml_dir)

        # Log fail
        mls = self.new_mls("err", self.missing_ml_dir, continue_on_error=True)
        stdout = capfd.readouterr().out
        assert re.search(self.missing_mls_err_msg(self.missing_ml_dir), stdout)
        assert list(mls.keys()) == []
        assert mls.directories == [self.missing_ml_dir]

    def test_continuing_on_multiple_fails(self, capfd):
        mls = self.new_mls("err_tests", self.missing_ml_dir, self.mls_dir, self.invalid_toml_mls_dir, continue_on_error=True)
        stdout = capfd.readouterr().out
        assert re.search(self.invalid_toml_err_msg(self.invalid_ml_toml1), stdout)
        assert re.search(self.invalid_toml_err_msg(self.invalid_ml_toml2), stdout)
        assert re.search(self.missing_mls_err_msg(self.missing_ml_dir), stdout)
        assert list(mls.keys()) == ["production", "dev", "weekly"]
        assert mls.directories == [self.missing_ml_dir, self.mls_dir, self.invalid_toml_mls_dir]

    def test_given_direct_maillist_path(self):
        # Direct ML path
        mls = self.new_mls("mls", self.custom1_ml)
        assert list(mls.keys()) == ["custom1"]
        assert mls.directories == []

        # Direct ML path
        mls = self.new_mls("mls", self.mls_dir, self.custom1_ml)
        assert list(mls.keys()) == [
            "production", "dev", "weekly",
            "custom1",
        ]
        assert mls.directories == [self.mls_dir]

    def test_given_direct_file_with_invalid_ext(self, capfd):
        f = Path(__file__)
        with pytest.raises(RuntimeError, match=self.invalid_file_ext_err_msg(f)):
            self.new_mls("err", self.mls_dir, f)

        mls = self.new_mls("err", self.mls_dir, f, continue_on_error=True)
        assert list(mls.keys()) == [
            "production", "dev", "weekly",
        ]
        assert mls.directories == [self.mls_dir]
        stdout = capfd.readouterr().out
        assert re.search(self.invalid_file_ext_err_msg(f), stdout)

    # MLS dict-like API
    class TestMallistsDictLikeAPI(Fixture_DictLikeAPI, Common):
        def parameterize(self):
            return {
                "keys": ["production", "dev", "weekly", "custom1", "custom2"],
                "klass": self.ml_class,
            }

        def boot_dict_under_test(self):
            return self.new_mls("aud_test", self.mls_dir, self.custom_mls_dir)
