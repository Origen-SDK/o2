The Origen App
==============

The application object represents the active Origen application. It owns the
workspace configuration and provides access to application-specific services,
paths, sessions, publishing hooks, and interfaces.

Application Class
-----------------

An application provides an ``Application`` class derived from
:class:`origen.application.Base`. Origen instantiates it while booting the
workspace. Application-specific behavior belongs in this class or in focused
modules called by it; DUT modeling should remain in block and target sources.

The active instance is available as ``origen.app``:

.. code-block:: python

   import origen

   print(origen.app.name)
   print(origen.app.root)
   print(origen.app.output_dir)

Do not construct another application instance manually.

Lifecycle And Context
---------------------

The application is initialized before application commands, targets, patterns,
and flows execute. During boot, Origen resolves configuration, initializes user
and plugin services, and then loads requested targets. Code that requires a DUT
or tester should run after target loading rather than at module import time.

Application Configuration
-------------------------

``config/application.toml`` defines application identity and defaults such as
targets, mode, revision control, and documentation paths. ``config/origen.toml``
contains Origen runtime/site configuration. See
:doc:`configuring_your_workspace` for precedence and environment overrides.

Directories
-----------

Common application paths include:

``root``
   The application workspace root.

``output_dir``
   Generated output, normally ``<APP ROOT>/output``.

``website_source_dir``
   Sphinx sources, normally ``web/source``.

``website_output_dir``
   Generated documentation, normally below ``output/web``.

Use these resolved paths instead of assuming the process was launched from a
specific working directory.

Plugins And Interfaces
----------------------

Plugins are installed Python dependencies which contribute reusable models,
commands, or generation content. The application interface translates
tester-neutral flow intent into tester-specific test objects. Keep policy in the
application and reusable device/test content in blocks or plugins.
