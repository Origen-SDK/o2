Creating A New Application
==========================

Create an application from the global environment:

.. code-block:: console

   $ origen new application my_chip --desc "My chip test application"
   $ cd my_chip

Use ``--path`` to choose a parent directory. Run
``origen new application --help`` for the exact options supported by the active
CLI.

Generated Structure
-------------------

The application generator creates a UV-managed Python project with:

* ``pyproject.toml`` for Python and Origen dependencies;
* ``config/application.toml`` for application defaults;
* ``config/origen.toml`` for site/runtime configuration;
* a Python package containing blocks, patterns, flows, and interfaces;
* ``targets/`` for DUT, tester, and environment selection;
* ``tests/`` for regression tests; and
* ``web/`` for Sphinx documentation.

Set up and verify the environment:

.. code-block:: console

   $ origen env setup
   $ origen -v
   $ origen target view
   $ origen exec pytest

Initial Configuration
---------------------

Review ``pyproject.toml`` before committing the generated application. Pin an
Origen version available from the configured package source and add plugins as
normal Python dependencies.

Set application defaults in ``config/application.toml``. For example:

.. code-block:: toml

   name = "my_chip"
   target = ["dut/my_chip", "tester/smt7"]

Keep organization-wide package-server and user configuration in an appropriate
site ``origen.toml`` rather than embedding credentials in the application.

Next Steps
----------

#. Create the DUT target and block model.
#. Select a tester target.
#. Add a small generation source and approved output.
#. Replace the generated documentation index with application-specific guides.
#. Run the tests and documentation build before the first check-in.
