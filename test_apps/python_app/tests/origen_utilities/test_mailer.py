import pytest, origen, _origen, pathlib, re
from tests.shared.python_like_apis import Fixture_DictLikeAPI
from tests.shared import in_new_origen_proc
from configs import mailer as mailer_configs

class TestMailer:
    def test_mailer_is_accessible(self):
        assert origen.mailer
        assert isinstance(origen.mailer, _origen.utility.mailer.Mailer)

    def test_mailer_defaults_are_set_from_config(self):
        assert origen.mailer.server == "smtp.origen.org"
        assert origen.mailer.port == 25
        assert origen.mailer.auth_method == "None"
        assert origen.mailer.domain == "origen.org"
        assert origen.mailer.timeout == 120
        assert origen.mailer.timeout_seconds == origen.mailer.timeout

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
            assert retn["auth_method"] == "None"
            assert retn["domain"] == None
            assert retn["timeout"] == 60
            assert retn["service_user"] == None
            assert retn["dataset"] == None
            assert isinstance(retn["username"], OSError)
            assert "Cannot retrieve username when using auth method 'None'" in str(retn["username"])
            assert isinstance(retn["password"], OSError)
            assert "Cannot retrieve password when using auth method 'None'" in str(retn["password"])
            assert retn["sender"] == "minimum@origen.orgs"

        def test_tls_service_user(self):
            retn = in_new_origen_proc(mod=mailer_configs)
            assert retn["server"] == "smtp.origen.org"
            assert retn["auth_method"] == "TLS"
            assert retn["service_user"] == "mailer_service_user"
            assert isinstance(retn["dataset"], OSError)
            assert "Cannot query the user dataset for the mailer when specifying a service user" in str(retn["dataset"])
            assert retn["username"] == "mailer"
            assert retn["password"] == "test"
            assert retn["sender"] == "service@origen.org"

        def test_tls_dataset(self):
            retn = in_new_origen_proc(mod=mailer_configs)
            assert retn["server"] == "smtp.origen.org"
            assert retn["auth_method"] == "TLS"
            assert retn["service_user"] == None
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
            assert retn["service_user"] == None
            assert retn["dataset"] == "for_mailer"
            assert retn["username"] == "mailer_name"
            assert retn["password"] == "mailer_pw"
            assert retn["sender"] == "not_mailer@origen.org"
            assert retn["for_mailer_dataset_email"] == None
            assert retn["not_mailer_dataset_email"] == "not_mailer@origen.org"
            assert retn["hierarchy"] == ["not_mailer"]

        def test_error_on_missing_server(self, capfd):
            err = "Mailer's 'server' parameter has not been set. Please update config parameter 'mailer__server' to enable use of the mailer"
            retn = in_new_origen_proc(mod=mailer_configs)
            assert isinstance(retn["server"], OSError)
            assert err in str(retn["server"])
            assert isinstance(retn["test"], OSError)
            assert err in str(retn["test"])
            assert retn["port"] == 123
            assert err in capfd.readouterr().out

        def test_error_on_tls_with_invalid_service_user(self, capfd):
            err = "Invalid service user 'blah' provided in mailer configuration"
            retn = in_new_origen_proc(mod=mailer_configs)
            assert retn["server"] == "smtp.origen.org"
            assert retn["auth_method"] == "TLS"
            assert isinstance(retn["service_user"], OSError)
            assert err in str(retn["service_user"])
            assert isinstance(retn["username"], OSError)
            assert err in str(retn["username"])
            assert isinstance(retn["password"], OSError)
            assert err in str(retn["password"])
            assert isinstance(retn["test"], OSError)
            assert err in str(retn["test"])
            assert err in capfd.readouterr().out

        def test_error_on_invalid_auth_method(self, capfd):
            retn = in_new_origen_proc(mod=mailer_configs)
            assert retn["server"] == "smtp.origen.org"
            assert retn["auth_method"] == "None"
            out = capfd.readouterr().out
            assert "Invalid auth method 'blah!' found in the mailer configuration" in out
            assert "Unable to fully configure mailer from config!" in out
            assert "Forcing no authentication (mailer__auth_method = 'None')" in out

        # def test_overriding_release_body(self):
        #     assert 1 == 0

        # def test_disabling_origen_sig(self):
        #     assert 1 == 0
        
        # def test_overriding_app_sig(self):
        #     assert 1 == 0

        # def test_disabling_app_sig(self):
        #     assert 1 == 0

class TestMaillist:
    @property
    def mls(self):
        return origen.mailer.maillists

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
            "empty_toml", "empty",

            # Other
            "example", "config_level"
        ])

    @property
    def dev_maillists(self):
        return set(["develop", "dev", "development"])
    
    @property
    def prod_maillists(self):
        return set(["release", "release2", "prod", "production"])

    def test_maillists_are_available(self):
        assert isinstance(self.mls, dict)
        assert set(self.mls.keys()) == set(self.available_maillists)
        assert isinstance(self.mls["develop"], _origen.utility.mailer.Maillist)

    def test_maillists_can_be_filtered_by_audience(self):
        mls = origen.mailer.maillists_for("release")
        assert isinstance(mls, dict)
        assert set(mls.keys()) == self.prod_maillists
        assert isinstance(mls["release"], _origen.utility.mailer.Maillist)

        mls = origen.mailer.maillists_for("example")
        assert isinstance(mls, dict)
        assert set(mls.keys()) == set(["example"])

        mls = origen.mailer.maillists_for("None")
        assert isinstance(mls, dict)
        assert set(mls.keys()) == set([])

    def test_enumerated_maillist_filters(self):
        mls = origen.mailer.development_maillists
        assert set(mls.keys()) == self.dev_maillists

        mls = origen.mailer.develop_maillists
        assert set(mls.keys()) == self.dev_maillists

        mls = origen.mailer.dev_maillists
        assert set(mls.keys()) == self.dev_maillists

        mls = origen.mailer.release_maillists
        assert set(mls.keys()) == self.prod_maillists

        mls = origen.mailer.prod_maillists
        assert set(mls.keys()) == self.prod_maillists

        mls = origen.mailer.production_maillists
        assert set(mls.keys()) == self.prod_maillists

    def test_non_toml_maillists_parameters_can_be_queried(self):
        ml = self.mls["develop"]
        assert ml.name == "develop"
        assert ml.recipients == [
            "d1@test_apps.origen.org",
            "d2@test_apps.origen.org",
            "d3@test_apps.origen.org",
            "d4@test_apps.origen.org",
        ]
        assert ml.resolve_recipients() == [
            "d1@test_apps.origen.org",
            "d2@test_apps.origen.org",
            "d3@test_apps.origen.org",
            "d4@test_apps.origen.org",
        ]
        assert ml.signature is None
        assert ml.audience == "development"
        assert ml.domain is None
        assert isinstance(ml.file, pathlib.Path)
        assert ml.file == origen.app.root.joinpath("config/maillists/develop.maillist")

    def test_toml_maillists_parameters_can_be_queried(self):
        ml = self.mls["release2"]
        assert ml.name == "release2"
        assert ml.recipients == [
            "or1",
            "or2",
            "or3",
            "or4",
        ]
        assert ml.domain == "other_release_domain.origen.org"
        assert ml.resolve_recipients() == [
            "or1@other_release_domain.origen.org",
            "or2@other_release_domain.origen.org",
            "or3@other_release_domain.origen.org",
            "or4@other_release_domain.origen.org",
        ]
        assert ml.signature == "You are received this as a recipient of the 'release2' maillist!"
        assert ml.audience == "production"
        assert ml.file == origen.app.root.joinpath("config/maillists/release2.maillist.toml")

    def test_empty_maillists_do_not_crash_anything(self):
        ml = self.mls["empty"]
        assert ml.name == "empty"
        assert ml.recipients == []
        assert ml.resolve_recipients() == []
        assert ml.signature is None
        assert ml.audience is None
        assert ml.domain is None
        assert ml.file == origen.app.root.joinpath("config/maillists/empty.maillist")

        ml = self.mls["empty_toml"]
        assert ml.name == "empty_toml"
        assert ml.recipients == []
        assert ml.resolve_recipients() == []
        assert ml.signature is None
        assert ml.audience is None
        assert ml.domain is None
        assert ml.file == origen.app.root.joinpath("config/maillists/empty_toml.maillist.toml")

    @property
    def config_tests_maillists_root(self):
        return pathlib.Path(__file__).parent.joinpath("configs/mailer/maillists")

    def test_adding_custom_mailist_directories(self):
        retn = in_new_origen_proc(mod=mailer_configs)
        assert set(retn["maillists"]) == set([*self.available_maillists, "custom1", "custom2", "other"])
        ml = retn["custom1"]
        assert ml["name"] == "custom1"
        assert ml["resolve_recipients"] == ["u1@custom1.org", "u2@custom2.org"]
        assert ml["audience"] == None
        assert ml["file"] == self.config_tests_maillists_root.joinpath("custom/custom1.maillist")
        ml = retn["custom2"]
        assert ml["name"] == "custom2"
        assert ml["resolve_recipients"] == ["u1@custom2.org", "u2@custom2.org"]
        assert ml["audience"] == "development"
        assert ml["file"] == self.config_tests_maillists_root.joinpath("custom/custom2.maillist.toml")
        assert set(retn["dev_maillists"]) == set([*self.dev_maillists, "other", "custom2"])

    def test_mailists_overwrite_lower_priority_ones(self):
        retn = in_new_origen_proc(mod=mailer_configs)
        assert set(retn["maillists"]) == set([*self.available_maillists, "custom1", "custom2"])
        ml = retn["custom2"]
        assert ml["name"] == "custom2"
        assert ml["resolve_recipients"] == ["u1@custom2.override.org", "u2@custom2.override.org"]
        assert ml["audience"] == None
        assert ml["file"] == self.config_tests_maillists_root.joinpath("override/custom2.maillist")
        ml = retn["develop"]
        assert ml["name"] == "develop"
        assert ml["resolve_recipients"] == ["u1@override.origen.org", "u2@override.origen.org"]
        assert ml["audience"] == "development"
        assert ml["file"] == self.config_tests_maillists_root.joinpath("override/develop.maillist.toml")

    def test_maillist_toml_ext_overwrite_maillist_ext(self):
        ''' Confirm that, within the same directory, a .maillist.toml overwrites a .maillist '''
        retn = in_new_origen_proc(mod=mailer_configs)
        ml_file = self.config_tests_maillists_root.joinpath("ext_overwrite/ext_overwrite.maillist")
        assert ml_file.exists
        assert set(retn["maillists"]) == set([*self.available_maillists, "ext_overwrite"])
        ml = retn["ext_overwrite"]
        assert ml["resolve_recipients"] == ["ext@overwrite.origen.org"]
        assert ml["file"] == self.config_tests_maillists_root.joinpath("ext_overwrite/ext_overwrite.maillist.toml")

    def test_error_nessage_on_invalid_toml(self, capfd):
        retn = in_new_origen_proc(mod=mailer_configs)
        assert set(retn["maillists"]) == self.available_maillists

        # The original develop maillist should persist
        ml = retn["develop"]
        assert ml["name"] == "develop"
        assert ml["resolve_recipients"] == [
            "d1@test_apps.origen.org",
            "d2@test_apps.origen.org",
            "d3@test_apps.origen.org",
            "d4@test_apps.origen.org",
        ]
        stdout = capfd.readouterr().out
        assert r"Errors encountered building maillist 'develop' from C:\Users\nxa13790\Documents\origen\o2\test_apps\python_app\tests\origen_utilities\configs\mailer\.\maillists\errors\invalid_toml\develop.maillist.toml: unexpected eof encountered" in stdout
        assert r"Errors encountered building maillist 'invalid_toml' from C:\Users\nxa13790\Documents\origen\o2\test_apps\python_app\tests\origen_utilities\configs\mailer\.\maillists\errors\invalid_toml\invalid_toml.maillist.toml: unexpected eof encountered" in stdout

    def test_error_message_on_conflicting_audiences(self, capfd):
        retn = in_new_origen_proc(mod=mailer_configs)

        def err_msg_regex(f, given_aud, mapped_given_aud, mapped_aud):
            return f"Maillist at '.*{f}.maillist.*' was given audience '{given_aud}' \\(maps to '{mapped_given_aud}'\\) but conflicts with the named audience '{mapped_aud}'. Maillist not added."
        stdout = capfd.readouterr().out
        assert set(retn["maillists"]) == self.available_maillists

        # Original maillists should be maintained
        n = "dev"
        ml = retn[n]
        f = self.app_ml_base.joinpath("dev.maillist.toml")
        assert ml["name"] == n
        assert ml["resolve_recipients"] == ["dev@test_apps.origen.org"]
        assert ml["audience"] == "development"
        assert ml["file"] == f
        assert re.search(err_msg_regex(n, "other", "other", "development"), stdout)

        n = "develop"
        ml = retn[n]
        f = self.app_ml_base.joinpath("develop.maillist")
        assert ml["name"] == n
        assert ml["resolve_recipients"] == [
            "d1@test_apps.origen.org",
            "d2@test_apps.origen.org",
            "d3@test_apps.origen.org",
            "d4@test_apps.origen.org"
        ]
        assert ml["audience"] == "development"
        assert ml["file"] == f
        assert re.search(err_msg_regex(n, "prod", "production", "development"), stdout)

        n = "prod"
        ml = retn[n]
        f = self.app_ml_base.joinpath("prod.maillist.toml")
        assert ml["name"] == n
        assert ml["resolve_recipients"] == ['prod1@test_apps.origen.org', 'prod2@test_apps.origen.org']
        assert ml["audience"] == "production"
        assert ml["file"] == f
        assert re.search(err_msg_regex(n, "development", "development", "production"), stdout)

    def test_redundant_audience_parameter(self):
        # This should be fine, albeit redundant
        # prod.toml -> set to release
        retn = in_new_origen_proc(mod=mailer_configs)

        # release.toml -> set to develop
        ml = retn["development"]
        assert ml["name"] == "development"
        assert ml["resolve_recipients"] == ["u1_dev@redundant.org"]
        assert ml["audience"] == "development"
        assert ml["file"] == self.config_tests_maillists_root.joinpath("redundant_audience/development.maillist.toml")

        # dev.toml -> set to production
        ml = retn["release"]
        assert ml["name"] == "release"
        assert ml["resolve_recipients"] == ["u1_release@redundant.org"]
        assert ml["audience"] == "production"
        assert ml["file"] == self.config_tests_maillists_root.joinpath("redundant_audience/release.maillist.toml")
