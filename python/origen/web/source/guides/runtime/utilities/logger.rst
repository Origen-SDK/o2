Logger
======

Origen's logger provides user-facing output, diagnostic verbosity, keyword
filters, and optional log files. Use it instead of ``print`` for application and
plugin diagnostics so CLI verbosity works consistently.

Basic Logging
-------------

The logger is available as both ``origen.logger`` and ``origen.log``:

.. code-block:: python

   import origen

   origen.log.info("Loading device model")
   origen.log.warning("Using fallback calibration")
   origen.log.error("Generation could not continue")

Use display-oriented methods for messages that should always be shown to the
user and diagnostic levels for implementation details.

Verbosity
---------

Increase terminal verbosity with repeated ``-v`` options:

.. code-block:: console

   $ origen -v generate pattern.py
   $ origen -vv generate pattern.py

Do not make correctness depend on verbosity. It controls reporting only.

Keyword Filtering
-----------------

Verbose listeners can be filtered with verbosity keywords:

.. code-block:: console

   $ origen --verbosity_keywords timing generate pattern.py

Use stable, documented keywords for noisy subsystems such as timing, model
loading, or program generation.

Errors And Exceptions
---------------------

Logging an error does not necessarily stop execution. Raise an exception or
return a failed outcome when a command cannot complete. CLI entry points must
propagate a nonzero exit status to CI.

Sensitive Data
--------------

Never log passwords, tokens, private keys, or complete credential-bearing URLs.
Verbose and debug logs are still persistent artifacts and should be treated as
potentially shareable diagnostic data.
