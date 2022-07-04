import pathlib
from tests._shared.for_proc import setenv

config_root = pathlib.Path(__file__).parent


def test_simple_ldap(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    assert len(origen.ldaps) == 1
    assert "simple" in origen.ldaps


def test_fully_configured_ldap(p, options):
    setenv(config_root, bypass_config_lookup=True)

    import pytest
    import origen
    assert len(origen.ldaps) == 1
    assert "full" in origen.ldaps
    l = origen.ldaps["full"]
    assert l.timeout == 45
    assert l.continuous_bind is True
    assert l.auth_config == {
        "scheme": "simple_bind",
        "username": "uname",
        "password": "pw",
        "allow_default_password": False,
        'use_default_motives': False,
        'priority_motives': ["p1", "p2"],
        'backup_motives': ["backup1", "backup2"],
    }

    assert l.populate_user_config == {
        "data_id": "invalid",
        "mapping": {
            "email": "contact",
            "last_name": "last",
            "full_name": "full"
        }
    }


def test_multiple_ldaps(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    assert len(origen.ldaps) == 3
    assert origen.ldaps["l1"].server == "ldap://ldap.duke.edu:389"
    assert origen.ldaps["l2"].server == "ldap://db.debian.org:389"
    assert origen.ldaps["l3"].server == "ldap://zflexldap.com:389"


def test_bad_ldap_config(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen


def test_empty_ldaps(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    assert len(origen.ldaps) == 0


def test_empty_ldap(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    assert len(origen.ldaps) == 0


def test_empty_config(q, options):
    setenv(config_root, bypass_config_lookup=True)

    import origen
    assert len(origen.ldaps) == 0
