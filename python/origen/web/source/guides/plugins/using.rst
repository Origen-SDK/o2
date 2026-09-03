Using a Plugin
==============

To use a plugin it should be added to the dependencies section of the application's :code:`pyproject.toml` file:

.. code:: toml

  [project]
  dependencies = ["my-plugin==2.13.0"]
    
In the example above, :code:`my_plugin` is the name of the Origen application to be plugged in and :code:`2.13.0`
is the version of it to be used.

It is also possible to specify a range of versions, for example if you want to automatically pick up newer versions
of the plugin as they are released.

See the `Python dependency specifier reference
<https://packaging.python.org/en/latest/specifications/dependency-specifiers/>`_
for supported version constraints.

After changing dependencies, refresh the workspace environment:

.. code-block:: console

  $ origen env setup
  $ origen plugins --help

Commit both the dependency declaration and lockfile used by the application.


Importing a DUT/Block
---------------------

Instantiating DUT's and (sub) blocks from a plugin is done in the same way as a locally owned block except that the
block path should be prefixed with the name of the plugin.
Here are some examples of Instantiating blocks from a plugin called *c16ff_common*:

.. code:: python

  # targets/dut/hawk.py
  origen.app.instantiate_dut("c16ff_common.dut.hawk")

  # <app_name>/blocks/my_block/sub_blocks.py
  SubBlock("usb", "c16ff_common.usb")

Plugin Commands
---------------

Plugins may contribute commands to the parent CLI. Discover installed plugins
and commands with ``origen plugins --help`` and ``origen plugin --help``. Keep
automation on canonical command paths rather than undocumented shortcuts.

Compatibility And Failures
--------------------------

If a plugin fails during boot, confirm:

* its version supports the active Origen release;
* all transitive dependencies resolve in the application environment;
* required site configuration is present;
* referenced targets, templates, and blocks are packaged in the wheel; and
* native extensions match the active Python and platform.

Pin a known-good plugin version while investigating; do not work around import
failures by copying plugin source into the parent application.


The plugin owner should provide the details of the paths to use in the plugin's documentation.


Generating Patterns and Flows
-----------------------------

Plugin patterns and flows are resolved through the installed package. The plugin
owner should document supported entry points, targets, templates, and required
parent-interface methods. Invoke them through the parent application's normal
generation command so its DUT, tester, configuration, and output policy remain
active:

.. code-block:: console

  $ origen generate path/provided/by/plugin.py \
      --target dut/my_device --target tester/smt7

Prefer stable plugin APIs or commands over reaching into undocumented package
paths. If a plugin supplies only components, include or call those components
from an application-owned pattern or flow.


Accessing a Plugin's Application Instance
-----------------------------------------

A plugin's application instance can be accessed via the following API:

.. code:: python

  origen.plugin("c16ff_common")   # => <Application object>

For example, to get the root of the plugin in the file system:


.. code:: python

  origen.plugin("c16ff_common").root   # => PosixPath('/my/path/to/c16ff_common')


That function will raise an error if the plugin is not found. If you want to test for the presence of a plugin instead,
then use:

.. code:: python

  origen.has_plugin("c16ff_common")         # => True
  origen.has_plugin("c16ff_common_other")   # => False
