import pathlib
from tests._shared.for_proc import setenv

config_root = pathlib.Path(__file__).parent
err_root = config_root.joinpath("error_conditions")


def test_error_on_invalid_datasets_in_hierarchy(q, options):
    setenv(err_root)

    import origen
    q.put(("users__data_lookup_hierarchy", origen.users.data_lookup_hierarchy))
    q.put(("users__datasets", list(origen.users.datasets.keys())))
    q.put(("data_lookup_hierarchy", origen.current_user.data_lookup_hierarchy))
    q.put(("datasets", list(origen.current_user.datasets.keys())))


def test_error_on_duplicate_datasets_in_hierarchy(q, options):
    setenv(err_root)

    import origen
    q.put(("users__data_lookup_hierarchy", origen.users.data_lookup_hierarchy))
    q.put(("users__datasets", list(origen.users.datasets.keys())))
    q.put(("data_lookup_hierarchy", origen.current_user.data_lookup_hierarchy))
    q.put(("datasets", list(origen.current_user.datasets.keys())))


def test_error_on_default_data_and_invalid_hierarchy(q, options):
    setenv(err_root, bypass_config_lookup=True)
    import origen
    q.put(("users__hierarchy", origen.users.data_lookup_hierarchy))
    q.put(("users__datasets", list(origen.users.datasets.keys())))
    q.put(("hierarchy", origen.current_user.data_lookup_hierarchy))
    q.put(("datasets", list(origen.current_user.datasets.keys())))


def test_single_dataset_and_default_hierarchy(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    q.put(("users__hierarchy", origen.users.data_lookup_hierarchy))
    q.put(("users__datasets", list(origen.users.datasets.keys())))
    q.put(("hierarchy", origen.current_user.data_lookup_hierarchy))
    q.put(("datasets", list(origen.current_user.datasets.keys())))
    q.put(("first_name_unset", origen.current_user.first_name))
    q.put(("first_name_dataset_unset",
           origen.current_user.datasets["test"].first_name))

    # Set the first name
    origen.current_user.first_name = "Corey"
    q.put(("first_name", origen.current_user.first_name))
    q.put(("first_name_dataset",
           origen.current_user.datasets["test"].first_name))


def test_single_dataset_and_explicit_hierarchy(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    q.put(("users__hierarchy", origen.users.data_lookup_hierarchy))
    q.put(("users__datasets", list(origen.users.datasets.keys())))
    q.put(("hierarchy", origen.current_user.data_lookup_hierarchy))
    q.put(("datasets", list(origen.current_user.datasets.keys())))
    q.put(("first_name_unset", origen.current_user.first_name))
    q.put(("first_name_dataset_unset",
           origen.current_user.datasets["test"].first_name))

    # Set the first name
    origen.current_user.first_name = "Corey2"
    q.put(("first_name", origen.current_user.first_name))
    q.put(("first_name_dataset",
           origen.current_user.datasets["test"].first_name))


def test_single_dataset_and_empty_hierarchy(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    q.put(("users__hierarchy", origen.users.data_lookup_hierarchy))
    q.put(("users__datasets", list(origen.users.datasets.keys())))
    q.put(("hierarchy", origen.current_user.data_lookup_hierarchy))
    q.put(("datasets", list(origen.current_user.datasets.keys())))
    try:
        origen.current_user.first_name
    except Exception as e:
        q.put(("first_name_unset", e))
    q.put(("first_name_dataset_unset",
           origen.current_user.datasets["test"].first_name))

    origen.current_user.data_lookup_hierarchy = ["test"]
    q.put(("first_name_unset_2", origen.current_user.first_name))


def test_multi_datasets_and_default_hierarchy(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    q.put(("users__hierarchy", origen.users.data_lookup_hierarchy))
    q.put(("users__datasets", list(origen.users.datasets.keys())))
    q.put(("hierarchy", origen.current_user.data_lookup_hierarchy))
    q.put(("datasets", list(origen.current_user.datasets.keys())))
    origen.current_user.data_lookup_hierarchy = ["test_1st", "test_2nd"]
    q.put(("hierarchy_2", origen.current_user.data_lookup_hierarchy))


def test_default_dataset_and_hierarchy(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    q.put(("users__hierarchy", origen.users.data_lookup_hierarchy))
    q.put(("users__datasets", list(origen.users.datasets.keys())))
    q.put(("hierarchy", origen.current_user.data_lookup_hierarchy))
    q.put(("datasets", list(origen.current_user.datasets.keys())))
    q.put(("first_name_unset", origen.current_user.first_name))
    q.put(("first_name_dataset_unset",
           origen.current_user.datasets["__origen__default__"].first_name))

    origen.current_user.first_name = "COREY"
    q.put(("first_name", origen.current_user.first_name))
    q.put(("first_name_dataset",
           origen.current_user.datasets["__origen__default__"].first_name))


def test_empty_hierarchy_and_default_dataset(q, options):
    setenv(config_root, bypass_config_lookup=True)
    import origen
    q.put(("users__hierarchy", origen.users.data_lookup_hierarchy))
    q.put(("users__datasets", list(origen.users.datasets.keys())))
    q.put(("hierarchy", origen.current_user.data_lookup_hierarchy))
    q.put(("datasets", list(origen.current_user.datasets.keys())))


def test_empty_datasets(q, options):
    setenv(config_root, bypass_config_lookup=True)
    import origen
    q.put(("users__data_lookup_hierarchy", origen.users.data_lookup_hierarchy))
    q.put(("users__datasets", list(origen.users.datasets.keys())))
    q.put(("hierarchy", origen.current_user.data_lookup_hierarchy))
    q.put(("datasets", list(origen.current_user.datasets.keys())))

def test_autopopulated_user(q, options):
    setenv(config_root, bypass_config_lookup=True)
    import os
    os.environ["USERNAME"] = "tesla"

    import origen
    assert origen.current_user.id == "tesla"
    assert list(origen.ldaps.keys()) == ["forumsys_autopop"]
    assert list(origen.users.datasets.keys()) == ["autopop_ldap"]
    assert origen.users.data_lookup_hierarchy == ["autopop_ldap"]
    assert origen.current_user.datasets["autopop_ldap"].populated == True
    assert origen.current_user.email == "tesla@ldap.forumsys.com"
    assert origen.current_user.last_name == "Tesla"
    assert origen.current_user.display_name == "tesla"
    assert origen.current_user.username == "tesla"
    assert origen.current_user.other["full_name"] == "Nikola Tesla"
