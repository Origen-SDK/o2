import pathlib
from tests._shared.for_proc import setenv

config_root = pathlib.Path(__file__).parent
err_root = config_root.joinpath("error_conditions")


def test_mailer_minimum(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    origen.current_user.password = "dummy"
    origen.current_user.username = "minimum"
    origen.current_user.password = "Mini"
    origen.current_user.email = "minimum@origen.orgs"
    q.put(("server", origen.mailer.server))
    q.put(("port", origen.mailer.port))
    q.put(("auth_method", origen.mailer.auth_method))
    q.put(("domain", origen.mailer.domain))
    q.put(("timeout", origen.mailer.timeout))
    q.put(("service_user", origen.mailer.service_user))
    try:
        q.put(("username", origen.mailer.username))
    except Exception as e:
        q.put(("username", e))
    try:
        q.put(("password", origen.mailer.password))
    except Exception as e:
        q.put(("password", e))
    q.put(("sender", origen.mailer.sender))
    q.put(("dataset", origen.mailer.dataset))


def test_mailer_empty(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    q.put(("mailer", origen.mailer))
    q.put(("app_mailer", origen.app.mailer))


def test_tls_service_user(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    q.put(("server", origen.mailer.server))
    q.put(("auth_method", origen.mailer.auth_method))
    q.put(("service_user", origen.mailer.service_user.id))
    q.put(("username", origen.mailer.username))
    q.put(("password", origen.mailer.password))
    q.put(("sender", origen.mailer.sender))
    try:
        q.put(("dataset", origen.mailer.dataset))
    except Exception as e:
        q.put(("dataset", e))


def test_tls_dataset(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    origen.current_user.datasets["for_mailer"].username = "mailer_name"
    origen.current_user.datasets["for_mailer"].password = "mailer_pw"
    origen.current_user.datasets["for_mailer"].email = "mailer@origen.org"
    origen.current_user.datasets["not_mailer"].email = "not_mailer@origen.org"

    q.put(("server", origen.mailer.server))
    q.put(("auth_method", origen.mailer.auth_method))
    q.put(("service_user", origen.mailer.service_user))
    q.put(("username", origen.mailer.username))
    q.put(("password", origen.mailer.password))
    q.put(("sender", origen.mailer.sender))
    q.put(("dataset", origen.mailer.dataset))
    q.put(("for_mailer_dataset_email",
           origen.current_user.datasets["for_mailer"].email))
    q.put(("not_mailer_dataset_email",
           origen.current_user.datasets["not_mailer"].email))
    q.put(("hierarchy", origen.current_user.data_lookup_hierarchy))


def test_tls_dataset_with_backup(q, options):
    setenv(config_root,
           config_name="test_tls_dataset",
           bypass_config_lookup=True)

    import origen
    origen.current_user.datasets["for_mailer"].username = "mailer_name"
    origen.current_user.datasets["for_mailer"].password = "mailer_pw"
    origen.current_user.datasets["not_mailer"].email = "not_mailer@origen.org"

    q.put(("server", origen.mailer.server))
    q.put(("auth_method", origen.mailer.auth_method))
    q.put(("service_user", origen.mailer.service_user))
    q.put(("username", origen.mailer.username))
    q.put(("password", origen.mailer.password))
    q.put(("sender", origen.mailer.sender))
    q.put(("dataset", origen.mailer.dataset))
    q.put(("for_mailer_dataset_email",
           origen.current_user.datasets["for_mailer"].email))
    q.put(("not_mailer_dataset_email",
           origen.current_user.datasets["not_mailer"].email))
    q.put(("hierarchy", origen.current_user.data_lookup_hierarchy))


def test_error_on_missing_server(q, options):
    setenv(err_root, bypass_config_lookup=True)
    import origen
    origen.current_user.email = "mailer@origen.org"
    q.put(("mailer", origen.mailer))
    q.put(("app_mailer", origen.app.mailer))


def test_error_on_bad_system(q, options):
    setenv(err_root, bypass_config_lookup=True)
    import origen
    q.put(("mailer", origen.mailer))
    q.put(("app_mailer", origen.app.mailer))


def test_error_on_tls_with_invalid_service_user(q, options):
    setenv(err_root, bypass_config_lookup=True)
    import origen
    origen.current_user.email = "mailer@origen.org"

    q.put(("server", origen.mailer.server))
    q.put(("auth_method", origen.mailer.auth_method))
    try:
        q.put(("service_user", origen.mailer.service_user))
    except Exception as e:
        q.put(("service_user", e))

    try:
        q.put(("username", origen.mailer.username))
    except Exception as e:
        q.put(("username", e))

    try:
        q.put(("password", origen.mailer.password))
    except Exception as e:
        q.put(("password", e))

    try:
        q.put(("test", origen.mailer.test()))
    except Exception as e:
        q.put(("test", e))


def test_error_on_invalid_auth_method(q, options):
    setenv(err_root, bypass_config_lookup=True)
    import origen
    q.put(("server", origen.mailer.server))
    q.put(("auth_method", origen.mailer.auth_method))
    try:
        q.put(("username", origen.mailer.username))
    except Exception as e:
        q.put(("username", e))
    try:
        q.put(("password", origen.mailer.password))
    except Exception as e:
        q.put(("password", e))
    try:
        q.put(("test", origen.mailer.test))
    except Exception as e:
        q.put(("test", e))


def dump_ml(ml):
    return {
        "name": ml.name,
        "recipients": ml.recipients,
        "resolve_recipients": ml.resolve_recipients(),
        "audience": ml.audience,
        "file": ml.file,
    }


# --- Maillists ---


def test_adding_custom_mailist_directories(q, options):
    setenv(config_root)
    import origen
    q.put(("maillists", list(origen.maillists.keys())))
    q.put(("custom1", dump_ml(origen.maillists["custom1"])))
    q.put(("custom2", dump_ml(origen.maillists["custom2"])))
    q.put(("other", dump_ml(origen.maillists["other"])))
    q.put(("dev_maillists", list(origen.maillists.dev_maillists.keys())))


def test_mailists_overwrite_lower_priority_ones(q, options):
    setenv(config_root)
    import origen
    q.put(("maillists", list(origen.maillists.keys())))
    q.put(("custom2", dump_ml(origen.maillists["custom2"])))
    q.put(("develop", dump_ml(origen.maillists["develop"])))


def test_maillist_toml_ext_overwrite_maillist_ext(q, options):
    setenv(config_root)
    import origen
    q.put(("maillists", list(origen.maillists.keys())))
    q.put(("ext_overwrite", dump_ml(origen.maillists["ext_overwrite"])))


def test_error_nessage_on_invalid_toml(q, options):
    setenv(config_root)
    import origen
    q.put(("maillists", list(origen.maillists.keys())))
    q.put(("develop", dump_ml(origen.maillists["develop"])))


def test_error_message_on_conflicting_audiences(q, options):
    setenv(config_root)
    import origen
    q.put(("maillists", list(origen.maillists.keys())))
    q.put(("dev", dump_ml(origen.maillists["dev"])))
    q.put(("develop", dump_ml(origen.maillists["develop"])))
    q.put(("prod", dump_ml(origen.maillists["prod"])))


def test_redundant_audience_parameter(q, options):
    setenv(config_root)
    import origen
    q.put(("maillists", list(origen.maillists.keys())))
    q.put(("development", dump_ml(origen.maillists["development"])))
    q.put(("release", dump_ml(origen.maillists["release"])))
