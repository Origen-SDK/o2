import pathlib
from tests._shared.for_proc import setenv

config_root = pathlib.Path(__file__).parent
err_root = config_root.joinpath("error_conditions")


def test_mailer_minimum(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    origen.current_user.password = "dummy"
    origen.current_user.username = "minimum"
    origen.current_user.email = "minimum@origen.orgs"
    q.put(("server", origen.mailer.server))
    q.put(("port", origen.mailer.port))
    q.put(("auth_method", origen.mailer.auth_method))
    q.put(("domain", origen.mailer.domain))
    q.put(("timeout", origen.mailer.timeout))
    q.put(("user", origen.mailer.user.id))
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
    q.put(("user", origen.mailer.user.id))
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
    q.put(("user", origen.mailer.user.id))
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
    q.put(("user", origen.mailer.user.id))
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

def test_error_on_bad_mailer_class(q, options):
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
        q.put(("user", origen.mailer.user.id))
    except Exception as e:
        q.put(("user", e))

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
    q.put(("mailer", origen.mailer))
    q.put(("app_mailer", origen.app.mailer))

# --- Maillists ---

def dump_ml(ml):
    return {
        "name": ml.name,
        "recipients": ml.recipients,
        "resolve_recipients": ml.resolve_recipients(),
        "audience": ml.audience,
        "file": ml.file,
    }

def test_adding_custom_maillist_directories(q, options):
    setenv(config_root)
    import origen
    q.put(("maillists", list(origen.maillists.keys())))
    q.put(("directories", list(origen.maillists.directories)))
    q.put(("custom1", dump_ml(origen.maillists["custom1"])))
    q.put(("custom2", dump_ml(origen.maillists["custom2"])))
    q.put(("other", dump_ml(origen.maillists["other"])))
    q.put(("dev_maillists", list(origen.maillists.dev_maillists.keys())))

def test_invalid_maillists_toml(q, options):
    setenv(config_root)
    import origen

def test_missing_maillists_dir(q, options):
    setenv(config_root)
    import origen
    q.put(("maillists", list(origen.maillists.keys())))
    q.put(("directories", list(origen.maillists.directories)))

def test_maillists_outside_of_app(q, options):
    setenv(None, cd="../../../../../")
    import origen
    q.put(("maillists", list(origen.maillists.keys())))
    q.put(("directories", list(origen.maillists.directories)))

def test_error_on_bad_maillists_class(q, options):
    setenv(err_root, bypass_config_lookup=True)
    import origen
    q.put(("maillists", origen.maillists))

def test_empty_maillists_config(q, options):
    setenv(config_root)
    import origen
    q.put(("maillists", list(origen.maillists.keys())))
    q.put(("directories", list(origen.maillists.directories)))

def test_error_on_loading_a_bad_maillist(q, options):
    setenv(err_root)
    import origen
    q.put(("maillists", list(origen.maillists.keys())))
    q.put(("directories", list(origen.maillists.directories)))
