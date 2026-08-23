Introduction
============

Origen separates test intent from tester file formats. Pattern source operates
on DUT pins, registers, protocols, and timesets; a tester backend renders the
resulting abstract syntax tree (AST). Test-program source builds a tester-neutral
flow model which a supported program backend translates into native resources.

This separation allows one application to share models and intent across tester
families while keeping tester-specific configuration explicit.

Selecting A Tester
------------------

Tester targets call ``origen.tester.target``:

.. code-block:: python

   import origen

   origen.tester.target("V93KSMT7")

Applications normally keep one file per tester under ``targets/tester`` and load
it with a DUT target:

.. code-block:: console

   $ origen target set dut/eagle tester/smt7

The active tester determines pattern syntax, file extension, timing translation,
and available program resources.

Tester-Neutral And Tester-Specific Code
---------------------------------------

Keep DUT operations and flow structure tester-neutral wherever possible. Use
``tester().eq(...)`` blocks in an application interface when a native resource
must differ:

.. code-block:: python

   with tester().eq("v93k") as v93k:
       # Create a V93K suite or method.
       pass

   with tester().eq("igxl") as igxl:
       # Create an IG-XL instance or pattern set.
       pass

Target conditions are evaluated for both pattern and program ASTs, so excluded
branches do not reach an incompatible backend.

Patterns Versus Programs
------------------------

Pattern support and test-program support are separate capabilities. A tester
may render vectors while its program renderer is incomplete. Consult
:doc:`supported_testers` rather than inferring program support from a valid
tester target.
