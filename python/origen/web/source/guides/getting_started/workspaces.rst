Workspaces
==========

The Application Workspace
-------------------------

An application workspace is a directory containing the application's
``pyproject.toml``, ``config/`` directory, Python package, targets, tests, and
generated-output directories. Entering that directory changes the CLI from the
global command set to the application command set.

For an existing checkout, initialize its environment before use:

.. code-block:: console

   $ cd my_application
   $ origen env setup
   $ origen -v

The environment setup resolves the application's Python dependencies and makes
its Origen CLI launcher available. Repeat it after material dependency changes.

Local State
-----------

Workspace-local selections and session data may be stored under ``.origen/``.
Generated files are normally written below ``output/`` and documentation below
``output/web``. These directories should be treated as disposable unless an
application explicitly uses generated files as approved references.

Targets And Modes
-----------------

The active targets determine the DUT, tester, and environment loaded for a
command. View or change them with:

.. code-block:: console

   $ origen target view
   $ origen target set dut/my_device tester/uflex
   $ origen target clear

Commands may also accept target overrides. Prefer committed default targets for
repeatable team workflows and use local overrides for focused development.

Standalone Workspaces
---------------------

``origen new workspace`` creates an environment without creating a full
application package. Use it for evaluation or orchestration that needs an Origen
environment but owns no distributable DUT or plugin content. Use
``origen new application`` for product code intended to be tested and shared.
