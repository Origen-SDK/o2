The Origen Tester
=================

``origen.tester`` is the active tester facade. It records pattern operations,
tracks timesets and pin state, exposes tester conditions, and supplies
tester-family APIs used by program interfaces.

Pattern Operations
------------------

Common operations include:

* selecting a timeset;
* adding comments;
* cycling or repeating cycles;
* capturing pin data;
* applying overlays; and
* rendering the completed pattern AST.

For example:

.. code-block:: python

   tester = origen.tester
   tester.set_timeset("functional")
   tester.cc("Enter functional sequence")
   origen.dut.pin("reset").drive(0)
   tester.cycle()
   origen.dut.pin("reset").drive(1)
   tester.repeat(10)

Pin drive and verify calls update model state. A cycle commits that state using
the active timeset and tester backend.

Tester Conditions
-----------------

``eq`` scopes content to one or more compatible testers:

.. code-block:: python

   with origen.tester.eq("v93ksmt7") as v93k:
       # V93K-specific setup
       pass

Family names are supported: V93K covers SMT7 and SMT8, while IGXL covers J750
and UltraFLEX. Use ``neq`` only when exclusion is clearer than listing the
supported testers.

Tester APIs
-----------

The object yielded by ``eq`` exposes the appropriate family API, such as V93K
test methods/suites or IG-XL test instances and pattern sets. Construct those
resources in the application interface and add them to the active flow through
the interface base methods.

State And Isolation
-------------------

Tester state belongs to one generation invocation. Do not retain tester objects
in module globals across runs. Targets initialize the tester, pattern contexts
start and finish ASTs, and program rendering consumes the flow/model accumulated
for that invocation.
