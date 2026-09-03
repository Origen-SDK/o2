No-Workspace tests require some setup to run. A custom pyenv can be setup for this, or manual changes can be made. A similar sequence is contained in `regression_test.yml`.

### Ensure the origen plugin is up to date with the latest executable and `pyapi`:
~~~
cd rust/origen
cargo build --workspace
cp target/debug/origen ../../python/origen/origen/__bin__/bin/
cd ../pyapi
cargo build
# For linux
cp target/debug/lib_origen.so ../../python/origen/_origen.so
# For Windows
 cp .\target\debug\_origen.dll ..\..\python\origen\_origen.pyd
~~~

### Clean origen_metal tmp/ directory
For some reason, this confuses `pip`, but is an easy workaround:

~~~
rm python/origen_metal/tmp -r
~~~

### Install Origen & Origen Metal

~~~
cd test_apps/no_workspace/user_install
uv sync --all-groups --no-editable
~~~

### Install Pytest
`pytest` is currently just a development dependency. Need to install it manually:

~~~
pip install pytest==7.2.1
~~~

### Run Some Tests
~~~
pytest test_no_workspace.py::TestNoWorkspaceNoPlugins -vv
~~~

See the `regression_test.yml` for more tests to run in this setup.
