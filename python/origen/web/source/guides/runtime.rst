Runtime
=======

The Origen runtime boots configuration, users, plugins, the application, active
targets, DUT, tester, and command context for each invocation. Understanding
that lifecycle helps keep application modules deterministic and prevents work
from happening before required state exists.

Invocation Lifecycle
--------------------

At a high level, Origen:

#. discovers global or application context from the working directory;
#. resolves site and application configuration;
#. initializes the Python environment and native extensions;
#. loads users, plugins, and the application object;
#. applies mode and target selections;
#. dispatches the requested core, application, plugin, or auxiliary command;
#. runs registered cleanup hooks; and
#. exits with the command's status.

Code that needs ``origen.dut`` or ``origen.tester`` must run after targets are
loaded. Avoid model-dependent work at module import time.

Runtime Context
---------------

Useful global accessors include ``origen.app``, ``origen.dut``,
``origen.tester``, ``origen.current_user``, ``origen.plugins``, and
``origen.current_command``. Treat them as runtime-owned objects; do not replace
or persist them across invocations.

Commands And Exit Status
------------------------

Core CLI commands dispatch into Python with parsed arguments and source
metadata. Application and plugin commands participate in the same lifecycle.
Failures must return a nonzero status so shells and CI can distinguish a logged
error from a successful operation.

Use ``origen exec`` for tools that should run inside the active environment:

.. code-block:: console

   $ origen exec pytest
   $ origen exec python scripts/check_model.py

Modes And Targets
-----------------

Mode represents application policy such as development or production. Targets
select concrete model/tester configuration. Keep mode checks focused on policy;
use tester conditions for output-format differences and DUT derivatives for
model differences.

.. toctree::

  runtime/utilities
