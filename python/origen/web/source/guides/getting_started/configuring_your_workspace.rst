Configuring Your Workspace
==========================

The configuration can be accessed programmatically with the :meth:`origen.config`

Site Configuration
------------------

Origen's runtime is configured through a single (or series) of |toml| files. If inside an application, Origen will look for a file ``config/origen.toml``, representing Origen's *site config for this application*. From there, Origen will walk up the directory tree, all the way to the root, looking for any ``origen.toml`` files.

Origen will do the same starting at the location of the ``Origen executable``, and walk up the directory tree from there.

Configurations will override each other on a per-value basis, allowing for global, or site-specific, defaults to be set at a level closer to the root (or residing with the executable) and be overridden "deeper" as needed. Given this scheme, if an application is present, ``config/origen.toml`` will have the highest priority, or in a global context, any ``origen.toml`` residing in the invocation directory.

Overriding Specific Values in the Environment
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

One extra level above the application or current invocation directory is available: the shell's ``environment``. Any value can be overridden by setting the environment variable ``ORIGEN_<VARIABLE NAME>``.

Inserting Configuration files
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

In between the application or global invocation directory's config files and the environment is another layer. Configurations can be enumerated in the ``origen_config_paths`` environment variable. This can be either a single value or a list of values separated by the ``PATH`` separator (e.g., ``:`` on Linux). The first path provided will have the highest priority within this group.

These paths can be either directories or files ending in ``.toml``. If they are files, they are used directly. Directories will be searched non-recursively for an ``origen.toml`` file.

Bypassing Automatic Configurations
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

The configuration resolution scheme outlined in the :link-to:`site config section <origen_site_config>` can by bypassed completely by setting the environment variable ``origen_bypass_config_lookup=1``. This will skip all configuration resolutions, including any present in the application or in the invocation directory, leaving only those enumerated by the ``origen_config_paths`` environment variable or those set by the environment itself.

Configuring Your Application
----------------------------

Configuration Options
^^^^^^^^^^^^^^^^^^^^^

Application behavior is split between two files:

``config/application.toml``
   Application identity and defaults such as targets, mode, revision control,
   sessions, publishing, and documentation paths.

``config/origen.toml``
   Origen runtime and site services such as package sources, users, LDAP,
   mailer, plugins, and auxiliary commands.

Keep secrets out of both files. Use the user/session credential APIs or the
organization's secret-management mechanism.

Environment Variable Mapping
^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Configuration keys can be overridden through prefixed environment variables.
Use this for CI and deployment-specific values, not as a substitute for
committed defaults. Document required overrides in the application README and
fail with a clear message when a mandatory value is missing.

Target Defaults
^^^^^^^^^^^^^^^

Default targets are an ordered list in ``application.toml``:

.. code-block:: toml

   target = ["dut/eagle", "tester/smt7"]

Users can override them with ``origen target`` commands or command-line target
options. Keep defaults usable for a safe development workflow; production-only
targets should not be the only way to boot the application.

Configuration Changes
^^^^^^^^^^^^^^^^^^^^^^

Add tests for configuration that affects model structure, generated output, or
external services. When introducing a key, document its type, default,
precedence, security implications, and whether changing it is backward
compatible.
^^^^^^^^^^^^^^^^^^^^^

.. _app-config-output-dir:

Output Directory
""""""""""""""""

Use the ``toml`` to set the application's output directory.
