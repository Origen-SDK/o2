import pytest, origen, _origen, pathlib, getpass
from tests.shared import in_new_origen_proc
from configs import mailer as mailer_configs
from tests import om_shared

with om_shared():
    from om_tests.utils.test_mailer import Common  # type:ignore

class TestMailer(Common):
    def test_mailer_is_accessible(self):
        assert origen.mailer
        assert isinstance(origen.mailer, self.mailer_class)

    def test_mailer_defaults_are_set_from_config(self):
        assert origen.mailer.server == "smtp.origen.org"
        assert origen.mailer.port == 25
        assert origen.mailer.auth_method == None
        assert origen.mailer.domain == "origen.org"
        assert origen.mailer.timeout == 120
        assert origen.mailer.timeout_seconds == origen.mailer.timeout
        assert origen.mailer.__user__ == "dummy_ldap_read_only"
        assert origen.mailer.config == {
            "server": "smtp.origen.org",
            "port": 25,
            "domain": "origen.org",
            "auth_method": None,
            "user": "dummy_ldap_read_only",
            "timeout": 120,
        }

    # def test_default_body_contents(self):
    #     assert origen.mailer.test_body == ...
    #     assert origen.mailer.release_body == ...

    # def test_default_origen_sig(self):
    #     assert origen.mailer.__origen_signature__ == ...

    # def test_default_app_sig(self):
    #     assert origen.mailer.__app_signature__ == ...

    # def test_mailer_auth_scheme(self):
    #     assert 1 == 0

    class TestConfigs:
        def test_mailer_minimum(self):
            retn = in_new_origen_proc(mod=mailer_configs)
            assert retn["server"] == "smtp_minimum.origen.org"
            assert retn["port"] == None
            assert retn["auth_method"] == None
            assert retn["domain"] == None
            assert retn["timeout"] == 60
            assert retn["user"] == getpass.getuser()
            assert retn["dataset"] == None
            assert retn["username"] == "minimum"
            assert retn["password"] == "dummy"
            assert retn["sender"] == "minimum@origen.orgs"

        def test_mailer_empty(self, capfd):
            retn = in_new_origen_proc(mod=mailer_configs)
            assert retn["mailer"] is None
            assert retn["app_mailer"] is None
            assert capfd.readouterr().err == ""

        def test_tls_service_user(self):
            retn = in_new_origen_proc(mod=mailer_configs)
            assert retn["server"] == "smtp.origen.org"
            assert retn["auth_method"] == "TLS"
            assert retn["user"] == "mailer_service_user"
            assert retn["dataset"] is None
            assert retn["username"] == "mailer"
            assert retn["password"] == "test"
            assert retn["sender"] == "service@origen.org"

        def test_tls_dataset(self):
            retn = in_new_origen_proc(mod=mailer_configs)
            assert retn["server"] == "smtp.origen.org"
            assert retn["auth_method"] == "TLS"
            assert retn["user"] == getpass.getuser()
            assert retn["dataset"] == "for_mailer"
            assert retn["username"] == "mailer_name"
            assert retn["password"] == "mailer_pw"
            assert retn["sender"] == "mailer@origen.org"
            assert retn["for_mailer_dataset_email"] == "mailer@origen.org"
            assert retn["not_mailer_dataset_email"] == "not_mailer@origen.org"
            assert retn["hierarchy"] == ["not_mailer"]

        def test_tls_dataset_with_backup(self):
            retn = in_new_origen_proc(mod=mailer_configs)
            assert retn["server"] == "smtp.origen.org"
            assert retn["auth_method"] == "TLS"
            assert retn["user"] == getpass.getuser()
            assert retn["dataset"] == "for_mailer"
            assert retn["username"] == "mailer_name"
            assert retn["password"] == "mailer_pw"
            assert retn["sender"] == "not_mailer@origen.org"
            assert retn["for_mailer_dataset_email"] == None
            assert retn["not_mailer_dataset_email"] == "not_mailer@origen.org"
            assert retn["hierarchy"] == ["not_mailer"]

        def test_error_on_missing_server(self, capfd):
            retn = in_new_origen_proc(mod=mailer_configs, expect_fail=True)
            stdout = capfd.readouterr().out
            assert "Malformed config file" in stdout
            assert "missing field `server`" in stdout

        def test_error_on_bad_mailer_class(self, capfd):
            retn = in_new_origen_proc(mod=mailer_configs)
            assert retn["mailer"] is None
            assert retn["app_mailer"] is None
            stdout = capfd.readouterr().out
            assert "Unable to initialize mailer" in stdout
            assert "module 'builtins' has no attribute 'blah'" in stdout

        def test_error_on_tls_with_invalid_service_user(self, capfd):
            err_u = "No user 'blah' has been added"
            retn = in_new_origen_proc(mod=mailer_configs)
            assert retn["server"] == "smtp.origen.org"
            assert retn["auth_method"] == "TLS"
            assert isinstance(retn["user"], RuntimeError)
            assert err_u in str(retn["user"])
            assert isinstance(retn["username"], RuntimeError)
            assert err_u in str(retn["username"])
            assert isinstance(retn["password"], RuntimeError)
            assert err_u in str(retn["password"])
            assert isinstance(retn["test"], RuntimeError)
            assert err_u in str(retn["test"])

        def test_error_on_invalid_auth_method(self, capfd):
            retn = in_new_origen_proc(mod=mailer_configs)
            assert retn["mailer"] == None
            assert retn["app_mailer"] == None

            out = capfd.readouterr().out
            assert "Unable to initialize mailer" in out
            assert "Invalid auth method 'blah!' found in the mailer configuration" in out

        # def test_overriding_release_body(self):
        #     assert 1 == 0

        # def test_disabling_origen_sig(self):
        #     assert 1 == 0

        # def test_overriding_app_sig(self):
        #     assert 1 == 0

        # def test_disabling_app_sig(self):
        #     assert 1 == 0


class TestMaillist(Common):
    origen_mls_dir = pathlib.Path(__file__).parent.joinpath("configs/mailer/maillists")
    others_dir = origen_mls_dir.joinpath("others")
    raw_custom_mls_dir = origen_mls_dir.parent.joinpath("../../../../../../python/origen_metal/tests/utils/maillists/custom")
    raw_invalid_mls_dir = origen_mls_dir.parent.joinpath("error_conditions/../../../../../../../python/origen_metal/tests/utils/maillists/errors/invalid_toml")

    @property
    def default_dirs(self):
        return [
            # Note that pytest isn't launched via the Origen CLI, so the CLI maillists dir is not added.
            origen.root.joinpath("config"),
            origen.root.joinpath("config/maillists"),
        ]

    @property
    def mls(self):
        return origen.maillists

    @property
    def app_ml_base(self):
        return origen.app.root.joinpath("config/maillists")

    @property
    def available_maillists(self):
        return set([
            # Development
            *self.dev_maillists,

            # Release
            *self.prod_maillists,

            # Empty
            "empty_toml",
            "empty",

            # Other
            "example",
            "config_level"
        ])

    @property
    def dev_maillists(self):
        return set(["develop", "dev", "development"])

    @property
    def prod_maillists(self):
        return set(["release", "release2", "prod", "production"])

    def test_maillists_are_available(self):
        assert isinstance(self.mls, self.mls_class)
        assert set(self.mls.keys()) == set(self.available_maillists)
        assert isinstance(self.mls["develop"], self.ml_class)
        assert self.mls.directories == self.default_dirs
        assert set(self.mls.keys()) == self.available_maillists

    def test_adding_custom_maillist_directories(self):
        retn = in_new_origen_proc(mod=mailer_configs)
        assert set(retn["maillists"]) == set(
            [*self.available_maillists, "custom1", "custom2", "other"])
        assert retn["directories"] == [
            *self.default_dirs,
            self.raw_custom_mls_dir,
            self.others_dir
        ]
        assert retn["directories"][2].resolve() == self.custom_mls_dir

        ml = retn["custom1"]
        assert ml["name"] == "custom1"
        assert ml["resolve_recipients"] == ["u1@custom1.org", "u2@custom2.org"]
        assert ml["audience"] == None
        assert ml["file"] == self.raw_custom_mls_dir.joinpath("custom1.maillist")

        ml = retn["custom2"]
        assert ml["name"] == "custom2"
        assert ml["resolve_recipients"] == ["u1@custom2.org", "u2@custom2.org"]
        assert ml["audience"] == "development"
        assert ml["file"] == self.raw_custom_mls_dir.joinpath("custom2.maillist.toml")
        assert set(retn["dev_maillists"]) == set(
            [*self.dev_maillists, "other", "custom2"])

    def test_invalid_maillists_toml(self, capfd):
        retn = in_new_origen_proc(mod=mailer_configs, expect_fail=True)
        stdout = capfd.readouterr().out
        assert "Malformed config file" in stdout
        assert 'invalid type: string "hi!", expected a sequence' in stdout

    def test_missing_maillists_dir(self, capfd):
        retn = in_new_origen_proc(mod=mailer_configs)
        assert retn["directories"] == [
            *self.default_dirs,
            self.origen_mls_dir.joinpath("./missing"),
            self.others_dir
        ]
        assert set(retn["maillists"]) == set([*self.available_maillists, "other"])

        stdout = capfd.readouterr().out
        split = str(self.origen_mls_dir.parent.joinpath('maillists')).rsplit('maillists', 1)
        assert f"Cannot find maillist path '{split[0]}./maillists/missing'" in stdout

    def test_maillists_outside_of_app(self, capfd):
        retn = in_new_origen_proc(mod=mailer_configs)
        assert retn["directories"] == []
        assert retn["maillists"] == []
        assert capfd.readouterr().out == ""
        assert capfd.readouterr().err == ""

    def test_error_on_bad_maillists_class(self, capfd):
        retn = in_new_origen_proc(mod=mailer_configs)
        assert retn["maillists"] is None
        stdout = capfd.readouterr().out
        assert "Unable to initialize maillists" in stdout
        assert "module 'builtins' has no attribute 'blah'" in stdout

    def test_empty_maillists_config(self):
        retn = in_new_origen_proc(mod=mailer_configs)
        assert set(retn["maillists"]) == self.available_maillists
        assert retn["directories"] == [*self.default_dirs]

    def test_error_on_loading_a_bad_maillist(self, capfd):
        retn = in_new_origen_proc(mod=mailer_configs)
        assert retn["directories"] == [
            *self.default_dirs,
            self.raw_invalid_mls_dir,
            self.others_dir.parent.parent.joinpath("error_conditions/../maillists/others")
        ]
        assert set(retn["maillists"]) == set([*self.available_maillists, "other"])

        stdout = capfd.readouterr().out
        assert f"Unable to build maillist from '{self.raw_invalid_mls_dir.joinpath('develop.maillist.toml')}'" in stdout
        assert f"Unable to build maillist from '{self.raw_invalid_mls_dir.joinpath('invalid_toml.maillist.toml')}'" in stdout
