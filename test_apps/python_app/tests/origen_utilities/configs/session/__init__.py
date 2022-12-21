import pathlib
from tests._shared.for_proc import setenv

config_root = pathlib.Path(__file__).parent

def test_user_session_root_can_be_updated_from_config(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    q.put(("root", origen.sessions.user_sessions.path))

def test_app_session_root_can_be_updated_from_app_config(q, options):
    # App config is not as involved, so just use an environment variable
    import os
    os.environ["origen_app_app_session_root"] = str(config_root.joinpath("app_session_test_root"))

    import origen
    q.put(("root", origen.sessions.app_sessions.path))
