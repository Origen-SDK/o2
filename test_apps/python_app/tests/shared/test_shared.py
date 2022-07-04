import getpass
from tests.shared import in_new_origen_proc
from _shared.configs import func__test_with_config, func__test_users_are_reset


def test_in_new_origen_proc():
    import origen

    assert "test_in_new_origen_proc" not in origen.users.keys()
    out = in_new_origen_proc(func__test_users_are_reset)
    assert set(out["users"]) == {getpass.getuser(), "dummy_ldap_read_only", "test_in_new_origen_proc"}

    assert "test_in_new_origen_proc" not in origen.current_user.datasets
    out = in_new_origen_proc(func__test_with_config)
    assert out["datasets"] == ["test_in_new_origen_proc"]
