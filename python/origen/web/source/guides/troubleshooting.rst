Troubleshooting
===============

Start With Context
------------------

Most O2 failures depend on the active Python environment, application, targets,
and tester. Capture these before changing code:

.. code-block:: console

   $ origen -v
   $ origen target view
   $ uv --version
   $ python --version

Repeat a failing command with ``-v`` or ``-vv`` when more diagnostics are
needed. Do not post logs containing credentials or internal URLs.

Command Not Found
-----------------

If ``origen`` is not found, verify the wheel's console script or the source
checkout's ``rust/origen/target/debug`` directory is on ``PATH``. Inside an
application, rerun ``origen env setup`` and use ``origen exec`` for environment
tools.

Console Scripts On Windows With Python 3.7
-------------------------------------------

UV writes Windows console scripts as a small executable with a zip archive
appended, and runs them by handing that executable back to Python. CPython 3.7
and earlier cannot read the archive and instead report a ``SyntaxError``
mentioning a ``.exe`` file. CPython 3.8 replaced the module responsible, so
newer versions are unaffected, and no Linux version is affected.

O2 works around this: on Windows with Python 3.7, ``origen env setup``
installs the environment with pip, whose launchers that interpreter can run.
The installed contents still come from ``uv.lock``, so the resolution is the
same as on every other platform. Nothing needs to be done differently.

If an environment on that combination was created by calling ``uv sync``
directly rather than through ``origen env setup``, its console scripts will not
run. Rerun ``origen env setup`` to reinstall it.

Python Import Or Native-Library Failures
----------------------------------------

Errors importing ``_origen`` or ``_origen_metal`` usually mean the native wheel
does not match the active Python/platform, or a source checkout has not been
rebuilt. Confirm the interpreter, reinstall the matching wheel, or rebuild both
native extensions. Do not rename a library built for another Python ABI.

Dependency Resolution
---------------------

Check that ``origen`` and ``origen_metal`` requirements overlap and that the
configured package source contains both. Keep UV lockfiles consistent with
``pyproject.toml``. For prerelease installations, pip requires ``--pre`` unless
the version is requested explicitly.

Missing DUT, Pin, Register, Or Timeset
--------------------------------------

Verify the intended DUT target is active and that the resource is defined in
the selected derivative. Remember that timesets may load lazily. Inspect the
fully qualified hierarchy and test aliases separately from physical pin names.

Generation Changes
------------------

When output differs unexpectedly, record:

* Origen and Origen Metal versions;
* active DUT and tester targets;
* plugin and template-library versions;
* model, timing, or interface changes; and
* the first semantic difference in approved output.

Avoid approving a broad output change until its source is understood.

Documentation Builds
--------------------

Use the user-facing command:

.. code-block:: console

   $ origen web build --clean

For a faster authored-content check:

.. code-block:: console

   $ origen web build --clean --no-api \
       --sphinx-args="-D origen_bypass_subprojects=True"

The full build additionally requires Rustdoc, AutoAPI, and configured
subprojects. Treat missing generated API pages as a failed full build, not as an
authored-content warning to ignore.

Reporting A Problem
-------------------

Include a minimal reproducer, versions, platform, command, complete traceback,
and whether the issue reproduces in a clean workspace. For generator bugs,
include minimal source and the relevant generated fragment rather than a full
proprietary program.
