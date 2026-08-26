Program Generation
==================

Test-program generation builds a tester-neutral flow AST and resource model,
then renders them for a supported backend. Flows describe ordering, conditions,
groups, bins, limits, and test intent. The application interface creates the
native test method, suite, or instance required by each tester.

Flow Source
-----------

A minimal flow uses the ``Flow`` context:

.. code-block:: python

   with Flow() as flow:
       flow.add_test("opens")
       flow.add_test("shorts")
       flow.func("functional_reset")
       flow.bin(1)

Large flows should be composed with includes and focused components instead of
one monolithic file.

Application Interface
---------------------

Flows call semantic methods such as ``func``. The application interface maps
those methods to tester resources:

.. code-block:: python

   def func(self, name, **options):
       with tester().eq("v93k") as v93k:
           method = v93k.new_test_method(
               "functional_test", library="ac_tml"
           )
           suite = v93k.new_test_suite(name)
           suite.test_method = method
           self.add_test(suite, **options)

       with tester().eq("igxl") as igxl:
           instance = igxl.new_test_instance(
               name, library="std", template="functional"
           )
           self.add_test(instance, **options)

Keep flow policy in flows and native resource construction in the interface.

Validation
----------

Before rendering, Origen validates identifiers, jobs, flags, relationships, and
tester conditions. Backend processors then normalize or reject operations based
on native capabilities. An unsupported flow node should fail generation rather
than disappear silently.

Output And References
---------------------

Generated files are returned by the backend and written below the selected
output directory. Approve representative flows for every supported tester,
including pass/fail branches, limits, bins, variables, and included subflows.

.. toctree::

  program_generation/program_api
