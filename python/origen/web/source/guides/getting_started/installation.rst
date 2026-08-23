.. _user-installation-guide:

Installation
============

Prerequisites
-------------

O2 supports Python 3.7 through 3.12 in the current package metadata. A normal
application workspace also requires Git and the standalone UV binary. Building O2 itself requires
the stable Rust toolchain and the platform development libraries described in
the :doc:`../developers/installation` guide.

Obtaining O2
------------

O2 development releases are published on public PyPI. Install the ``origen``
command as an isolated UV-managed tool. Because the current release is a
prerelease, allow prerelease versions or request one explicitly:

.. code-block:: console

   $ uv tool install --python 3.11 --prerelease allow origen

To install a specific release instead:

.. code-block:: console

   $ uv tool install --python 3.11 origen==2.0.0.dev8

UV keeps the command-line tool isolated from application dependencies. Existing
environments may still install the wheel with pip, but pip is a compatibility
option rather than O2's recommended environment-management workflow:

.. code-block:: console

   $ python -m pip install --pre --upgrade origen

Verify the installed version rather than relying on a version copied from this
guide:

.. code-block:: console

   $ origen -v

Published wheels are available for supported CPython versions on Linux and
Windows. An organization may instead mirror these wheels on an internal package
server; configure that source according to local policy.

Developers changing O2 itself should build and install it from a source checkout
rather than using the published wheel.

For a source checkout, follow :doc:`../developers/installation` to build the
Rust CLI and Python extensions. Add the resulting CLI directory to ``PATH``:

.. code-block:: bash

   export PATH="/path/to/o2/rust/origen/target/debug:$PATH"

Verify the installation from a directory outside an application:

.. code-block:: console

   $ origen --help
   Origen, The Semiconductor Developer's Kit

Application Environments
------------------------

Each application owns a UV environment described by ``pyproject.toml`` and
locked by ``uv.lock``.
After cloning an existing application, initialize or refresh that environment:

.. code-block:: console

   $ cd my_application
   $ origen env setup
   $ origen -v

Use ``origen exec`` to invoke tools inside the application environment:

.. code-block:: console

   $ origen exec pytest

This avoids accidentally running pytest, Sphinx, or another tool from a global
Python installation.

Applications created by older O2 releases may still contain
``[tool.poetry]`` metadata and ``poetry.lock``. Migrate that metadata before
running any UV-backed command:

.. code-block:: console

   $ origen env migrate --dry-run
   $ origen env migrate
   $ origen env setup

The dry run prints the proposed manifest diff without writing. The migration
converts supported dependencies and sources, generates ``uv.lock``, and removes
``poetry.lock`` only after the UV lock is validated. Unsupported Poetry-specific
configuration is reported before any file changes. Review and commit
``pyproject.toml`` and ``uv.lock`` together.

Troubleshooting
---------------

If ``origen`` cannot be found, verify ``PATH`` and the executable location. If
the CLI starts but Python imports fail, rebuild the native extensions for the
active Python version and rerun ``origen env setup``. If dependency resolution
fails, confirm that the configured package source contains compatible ``origen``
and ``origen_metal`` releases.
