Plugins
=======

Plugins package reusable Origen models, protocols, generation interfaces,
commands, and templates as Python distributions. Use a plugin when content has
a lifecycle independent of the consuming application or is shared by multiple
products.

Plugins execute in the parent application's runtime, so version compatibility
and tests matter. Keep public entry points stable, avoid modifying parent global
state during import, and document required targets or configuration.

.. toctree::

  plugins/introduction
  plugins/creating
  plugins/using
