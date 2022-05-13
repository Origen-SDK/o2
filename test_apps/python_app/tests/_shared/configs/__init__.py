import pathlib
from tests._shared.for_proc import setenv


def func__test_users_are_reset(q, options):
    import origen
    origen.users.add("test_in_new_origen_proc")
    q.put(("users", list(origen.users.keys())))


def func__test_with_config(q, options):
    setenv(pathlib.Path(__file__).parent,
           config_name="func__test_with_config",
           bypass_config_lookup=True)

    import origen
    q.put(("datasets", list(origen.current_user.datasets.keys())))
