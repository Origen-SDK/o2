Pattern Generation
==================

Patterns are Python sources evaluated in an Origen application context. They
operate on the active DUT and tester and are rendered by the tester selected in
the target set.

Basic Pattern
-------------

Use a ``Pattern`` context to delimit generation:

.. code-block:: python

   with Pattern(pin_header="all") as pat:
       origen.tester.set_timeset("functional")
       origen.tester.cc("Reset sequence")
       origen.dut.pin("reset").drive(0)
       origen.tester.repeat(5)
       origen.dut.pin("reset").drive(1)
       origen.tester.cycle()

Applications may use the injected ``dut()`` and ``tester()`` helpers or the
explicit ``origen.dut`` and ``origen.tester`` objects. Prefer one style
consistently within a project.

Pin State And Cycles
--------------------

Driving or verifying a pin changes modeled state. ``cycle`` commits one cycle;
``repeat`` commits repeated cycles. Set a timeset before cycling whenever the
target does not establish a suitable default.

Register And Protocol Operations
--------------------------------

Register writes and verifies call block controllers, which translate them into
protocol pin operations. This keeps pattern source at the register-intent level:

.. code-block:: python

   reg = origen.dut.reg("control")
   reg.set_data(0x5)
   reg.write()

Captures And Overlays
---------------------

Capture marks cycles whose pin data should be returned by the tester. Overlay
marks data that a downstream tool may substitute. Support and syntax vary by
backend; keep approved output for patterns using either feature.

Generating
----------

Invoke generation with the application command and active targets:

.. code-block:: console

   $ origen generate example/patterns/reset.py \
       --target dut/eagle --target tester/smt7

Run ``origen generate --help`` for output and reference-directory options.

Regression Strategy
-------------------

Test pattern source at two levels: inspect model/AST behavior where practical,
and compare representative generated files against approved references. Avoid
approving large diffs without identifying the model or renderer change that
caused them.

.. toctree::

  pattern_generation/pattern_api.rst
