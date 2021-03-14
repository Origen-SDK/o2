import pathlib
from tests._shared.for_proc import setenv

config_root = pathlib.Path(__file__).parent
err_root = config_root.joinpath("error_conditions")

def test_mailer_minimum(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    origen.current_user().password = "dummy"
    origen.current_user().username = "minimum"
    origen.current_user().password = "Mini"
    origen.current_user().email = "minimum@origen.orgs"
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

def test_tls_service_user(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    q.put(("server", origen.mailer.server))
    q.put(("auth_method", origen.mailer.auth_method))
    q.put(("service_user", origen.mailer.service_user))
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
    origen.current_user().datasets["for_mailer"].username = "mailer_name"
    origen.current_user().datasets["for_mailer"].password = "mailer_pw"
    origen.current_user().datasets["for_mailer"].email = "mailer@origen.org"
    origen.current_user().datasets["not_mailer"].email = "not_mailer@origen.org"

    q.put(("server", origen.mailer.server))
    q.put(("auth_method", origen.mailer.auth_method))
    q.put(("service_user", origen.mailer.service_user))
    q.put(("username", origen.mailer.username))
    q.put(("password", origen.mailer.password))
    q.put(("sender", origen.mailer.sender))
    q.put(("dataset", origen.mailer.dataset))
    q.put(("for_mailer_dataset_email", origen.current_user().datasets["for_mailer"].email))
    q.put(("not_mailer_dataset_email", origen.current_user().datasets["not_mailer"].email))
    q.put(("hierarchy", origen.current_user().data_lookup_hierarchy))


def test_tls_dataset_with_backup(q, options):
    setenv(config_root, config_name="test_tls_dataset", bypass_config_lookup=True)

    import origen
    origen.current_user().datasets["for_mailer"].username = "mailer_name"
    origen.current_user().datasets["for_mailer"].password = "mailer_pw"
    origen.current_user().datasets["not_mailer"].email = "not_mailer@origen.org"

    q.put(("server", origen.mailer.server))
    q.put(("auth_method", origen.mailer.auth_method))
    q.put(("service_user", origen.mailer.service_user))
    q.put(("username", origen.mailer.username))
    q.put(("password", origen.mailer.password))
    q.put(("sender", origen.mailer.sender))
    q.put(("dataset", origen.mailer.dataset))
    q.put(("for_mailer_dataset_email", origen.current_user().datasets["for_mailer"].email))
    q.put(("not_mailer_dataset_email", origen.current_user().datasets["not_mailer"].email))
    q.put(("hierarchy", origen.current_user().data_lookup_hierarchy))

def test_error_on_missing_server(q, options):
    setenv(err_root, bypass_config_lookup=True)
    import origen
    origen.current_user().email = "mailer@origen.org"

    try:
        q.put(("server", origen.mailer.server))
    except Exception as e:
        q.put(("server", e))

    try:
        q.put(("test", origen.mailer.test()))
    except Exception as e:
        q.put(("test", e))
    q.put(("port", origen.mailer.port))

def test_error_on_tls_with_invalid_service_user(q, options):
    setenv(err_root, bypass_config_lookup=True)
    import origen
    origen.current_user().email = "mailer@origen.org"

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
