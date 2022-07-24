To run regression tests, from the project root, run `poetry run pytest`.

Running all tests require internet access. If running offline, `poetry run pytest -m "not online"` will bypass tests marked as needing an internet connection.

A subset of the _online_ tests connect to a free dummy LDAP for testing purposes. This LDAP is not operated by the Origen team and, should this go down or otherwise become unavailable, these tests can be using `poetry run pytest -m "not ldap"`.