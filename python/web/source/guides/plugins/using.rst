Using a Plugin
==============

To use a plugin it should be added to the dependencies section of the application's :code:`pyproject.toml` file:

.. code:: toml

  [tool.poetry.dependencies]
  my_plugin = "2.13.0"
    
In the example above, :code:`my_plugin` is the name of the Origen application to be plugged in and :code:`2.13.0`
is the version of it to be used.

It is also possible to specify a range of versions, for example if you want to automatically pick up newer versions
of the plugin as they are released.

For more information on how to specify version requirements see here: `<https://python-poetry.org/docs/versions/>`_.


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


The plugin owner should provide the details of the paths to use in the plugin's documentation.


Generating Patterns and Flows
-----------------------------

.. include:: /_templates/doc_fail.rst


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
