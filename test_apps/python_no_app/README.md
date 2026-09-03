# About

This simple app is used to test the Origen python package when it is imported and used
within a Python app which is not an Origen app.

## Useful Commands

Setup the app:

~~~
uv sync --all-groups --no-editable
~~~

To install the latest Origen build run this command **from within this application directory**:

~~~
origen build
~~~

To run the tests:

~~~
uv run --no-editable pytest
~~~