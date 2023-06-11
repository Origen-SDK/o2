Note that the `tests` folder is the place to create tests to verify the
functionality of this Origen application, it does not refer to silicon
tests!

This is setup to use a popular Python test framework called `pytest` which
requires that following naming convention is followed:

* All files containing tests must be named `<NAME>_test.py`, e.g. `tests/example_test.py`
* Functions that define tests within such files must be named `test_<NAME>`,
  e.g. `def test_example():`

To execute the tests, run the following command from the command line:

~~~
poetry run pytest
~~~