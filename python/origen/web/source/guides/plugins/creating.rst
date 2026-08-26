Creating a Plugin
=================

An Origen plugin is simply an Origen application like any other, no special setup is required
in order to make an existing application function as a plugin to another parent application.

Create a plugin workspace from the global environment:

.. code-block:: console

  $ origen new plugin protocol_common --desc "Shared protocol models"
  $ cd protocol_common
  $ origen env setup
  $ origen exec pytest

The generated package can contain blocks, commands, interfaces, templates,
patterns, flows, and documentation. Export only supported entry points and keep
test-only helpers out of the public namespace.

Design Guidelines
-----------------

* Keep reusable models independent of a specific parent DUT path.
* Accept instance configuration through block options instead of globals.
* Namespace commands and resources to avoid collisions.
* Declare compatible Origen versions in ``pyproject.toml``.
* Test the plugin alone and inside at least one representative parent app.
* Document configuration, targets, generated resources, and breaking changes.

To distribute an application, it must be packaged up by running the following command:

.. code:: none

  origen app package

This creates a `Python wheel <https://realpython.com/python-wheels/>`_ archive of the application
in the :code:`dist/` directory.

Releasing a Plugin
------------------

To release a plugin, build a wheel and publish it to the Python package index
used by consuming applications:

.. code-block:: console

  $ origen app package

The command creates a wheel below ``dist/``. Validate that wheel in a clean
environment before upload. Publishing credentials and repository URLs are
organization-specific; use the package server's supported secure upload flow.

Use semantic versioning for the plugin's public API. A model or template change
that alters generated output may require a release note even when Python call
signatures remain compatible.
