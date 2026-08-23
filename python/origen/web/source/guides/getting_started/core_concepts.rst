Core Concepts
=============

The Origen CLI
--------------

The ``origen`` executable discovers its context from the current directory.
Outside an application it exposes global commands such as ``new``. Inside an
application it additionally exposes targets, generation, application commands,
and documentation commands.

Useful discovery commands are:

.. code-block:: console

   $ origen --help
   $ origen <command> --help
   $ origen -v

``origen exec`` runs another program in the active application environment.
Use it for tests and development tools rather than relying on global packages.

Frontend And Backend
--------------------

User applications import the Python package ``origen``. It provides the public
frontend API and delegates model operations to the compiled ``_origen`` and
``origen_metal`` extensions. Application code should use ``origen`` rather than
importing ``_origen`` directly.

Applications And Workspaces
---------------------------

An application is a distributable Python project containing Origen models and
generation sources. A workspace is a checked-out, configured instance in which
the application is executed. The same application may have many workspaces with
different local targets, credentials, or generated output.

Targets
-------

A target is executable configuration. Target files commonly instantiate a DUT,
select a tester, or set environment-specific values. Multiple target files may
be active together:

.. code-block:: console

   $ origen target set dut/my_device tester/smt7
   $ origen target view

The application may define default targets in ``config/application.toml``.
Command-line target options override those defaults for one invocation.

Source And Generated Output
---------------------------

Models, patterns, and flows are source code and belong in revision control.
Generated tester files normally live below ``output/``. Reference files and
tests should be used to make changes to generated output explicit and reviewable.
