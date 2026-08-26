Program API
===========

Program generation has three cooperating API layers:

``Flow``
   Adds tester-neutral tests, groups, conditions, bins, flags, variables, and
   included flows to the AST.

Application interface
   Implements semantic test types and adds native test resources to the flow.

Tester-family API
   Creates V93K methods/suites or IG-XL instances, pattern sets, levels, and
   related resources.

Tester Conditions
-----------------

Use ``tester().eq`` to create resources only for compatible testers and
``tester().neq`` for a genuine exclusion case. Family selectors such as
``v93k`` and ``igxl`` cover their derivatives.

Resource Ownership
------------------

Create resources through the yielded tester API and register invocations through
the interface. Do not append directly to backend models. The interface handles
IDs, source metadata, conditions, and relationships used by validation and
rendering.

Reference
---------

See the generated ``origen_metal.prog_gen`` and tester API documentation for
installed signatures. Template libraries may add attributes beyond the built-in
model; pin those libraries and validate required parameters during generation.

Comments
--------
