Supported Testers
=================

Pattern and test-program capabilities are tracked independently.

.. list-table:: Built-in tester status
   :header-rows: 1

   * - Tester
     - Pattern rendering
     - Test-program rendering
   * - Advantest V93K SmarTest 7
     - Implemented
     - Implemented
   * - Advantest V93K SmarTest 8
     - Implemented
     - Implemented
   * - Teradyne J750 (IG-XL)
     - Implemented
     - Not implemented by the current core renderer
   * - Teradyne UltraFLEX (IG-XL)
     - Implemented
     - Implemented by the UltraFLEX program-generation backend
   * - Simulator
     - Implemented for supported simulation operations
     - Not applicable

``V93K`` and ``IGXL`` are family selectors used in tester conditions; select a
concrete tester for rendering. ``UFLEX`` is accepted as an alias for
``ULTRAFLEX``.

“Implemented” does not imply every native feature is modeled. Consult backend
tests and tester-specific documentation for captures, overlays, timing, flow
nodes, and resource limitations. Keep approved outputs for the exact O2 release
used by an application.

See :doc:`ultraflex` for UltraFLEX instances, limits, pattern resources, IG-XL
worksheets, DUT-derived resources, generated files, and current limitations.
