Pattern API
===========

The primary pattern APIs are exposed by:

* :class:`origen.tester.Tester` and the native tester API;
* DUT pin and pin-group objects;
* register and bit-collection objects;
* protocol/service controllers; and
* the ``Pattern`` generation context.

The generated :link-to:`Origen API <origen_api>` and |_origen_api| references
contain the complete callable surface. The guide focuses on stable usage
patterns rather than duplicating generated signatures.

Common Operations
-----------------

.. list-table:: Pattern operations
   :header-rows: 1

   * - Intent
     - Typical API
   * - Select timing
     - ``origen.tester.set_timeset(name)``
   * - Add a comment
     - ``origen.tester.cc(text)``
   * - Emit one cycle
     - ``origen.tester.cycle()``
   * - Emit repeated cycles
     - ``origen.tester.repeat(count)``
   * - Drive or verify pins
     - ``pin.drive(value)`` / ``pin.verify(value)``
   * - Capture pin data
     - ``pin.capture(...)`` or tester capture APIs
   * - Apply an overlay
     - ``pin.overlay(...)`` or tester overlay APIs
   * - Write or verify a register
     - ``reg.write()`` / ``reg.verify()``

Exact keyword support can vary by release and tester backend. Use the generated
API for the installed O2 version and keep tester-specific behavior in regression
tests.

Comments
--------
