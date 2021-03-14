import pytest, origen, _origen, pathlib
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

# class TestMaillist:
#     def test_maillist_are_read_from_the_config(self):
#         assert origen.config.maillists == ["develop", "release", "inside_app", "outside_app"]

#     class TestMaillistDictLikeAPI(Fixutre_DictLikeAPI):
#         ...

#     def test_maillist_can_be_read(self):
#         assert origen.config.maillist["develop"].emails == ""
#         assert origen.config.maillist["develop"].type == ""
#         assert origen.config.maillist["develop"].signature == ""

#     def test_maillist_can_be_filter_by_type(self):
#         assert origen.config.maillist.by_type("release") == ["release", "outside_app_release"]

#     def test_maillists_can_be_combined(self):
#         ...

#     def test_maillist_flag_duplicate_recipients(self):
#         ...
