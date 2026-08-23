Global Environment
==================

Origen can run with or without an active application. The global environment is
the context used when the current directory is not inside an application
workspace. It is primarily used to create or inspect environments:

.. code-block:: console

   $ origen new --help
   $ origen new application my_chip
   $ origen new workspace evaluation

Application-only commands are intentionally absent in this context because no
DUT, target configuration, or application environment has been selected.

Configuration
-------------

Global configuration is read from applicable ``origen.toml`` files and
environment overrides. Site administrators can use this mechanism to provide a
package server or organization defaults. Application configuration has higher
priority once a workspace is entered. See
:doc:`configuring_your_workspace` for the resolution order.

Confirming Context
------------------

Run ``origen -v`` to see the active Origen versions and application context.
If application commands unexpectedly disappear, verify that the current path is
within a workspace containing ``config/origen.toml`` and a valid application
configuration.
