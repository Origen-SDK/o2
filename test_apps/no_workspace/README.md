No-Workspace tests require some setup to run. This can custom pyenv can be setup for this, or manual changes can be made. A similar sequence is contained in `regression_test.yml`.

### Ensure the origen plugin is up to date with the latest executable and `pyapi`:
~~~
cd rust/origen
cargo build --workspace
cp target/debug/origen ../../python/origen/origen/__bin__/bin/
cd ../pyapi
cargo build
# For linux
cp target/debug/lib_origen.so ../../python/origen/_origen.so
~~~

### Clean origen_metal tmp/ directory
For some reason, this confuses `pip`, but is an easy workaround:

~~~
rm python/origen_metal/tmp -r
~~~

### Remove Origen-Metal Dependency from Origen
Origen Metal version will likely not be checked in during development. Get around this by removing it as a dependency from Origen
Origen Metal can be installed manually afterwards.

~~~
cd python/origen
poetry remove origen_metal
~~~

### Install Origen & Origen Metal

~~~
pip install python/origen
pip install python/origen_metal
~~~

### Install Pytest
`pytest` is currently just a development dependency. Need to instal it manually:

~~~
pip install pytest==7.2.1
~~~

### Run Som Tests
~~~
pytest test_no_workspace.py::TestNoWorkspaceNoPlugins -vv
~~~

See the `regression_test.yml` for more tests to run in this setup.