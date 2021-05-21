import getpass
from tests.shared import in_new_origen_proc
from _shared.configs import func__test_with_config, func__test_users_are_reset


def test_in_new_origen_proc():
    import origen

    assert "test_in_new_origen_proc" not in origen.users.keys()
    out = in_new_origen_proc(func__test_users_are_reset)
    assert out["users"] == [getpass.getuser(), "test_in_new_origen_proc"]

    assert "test_in_new_origen_proc" not in origen.current_user().datasets
    out = in_new_origen_proc(func__test_with_config)
    assert out["datasets"] == ["test_in_new_origen_proc"]
