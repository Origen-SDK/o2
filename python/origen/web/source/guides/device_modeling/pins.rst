Pins
====

Pins connect tester actions to DUT terminals. Define physical pins first, then
add aliases and headers used by protocols, timing, and generated patterns.

Defining Pins
-------------

Declare scalar or vector pins in ``pins.py``:

.. code-block:: python

   Pin("porta", width=2)
   Pin("portb", width=4)
   Pin("reset", reset_action="1")
   Pin("clk", reset_action="0")

A vector declaration creates indexed pins such as ``porta0`` and ``porta1``.
``reset_action`` defines the model state used when pins are reset.

Aliases
-------

Aliases give one physical pin names appropriate to different protocols:

.. code-block:: python

   Alias("clk", "swd_clk", "swdclk", "tclk")
   Alias("porta0", "swdio")

Use aliases when the electrical connection is identical. Do not create separate
physical pins for alternate functional names.

Pin Headers
-----------

Headers define ordered collections used by pattern formats and protocol drivers:

.. code-block:: python

   PinHeader("ports", "porta", "portb")
   PinHeader("swd", "reset", "swdclk", "swdio")
   PinHeader("all", "clk", "reset", "porta", "portb")

Header order is significant because vector-based testers render states in that
order. Add a regression test when changing it.

Actions And State
-----------------

Pattern APIs apply semantic actions such as drive high, drive low, verify high,
verify low, high impedance, capture, and overlay. The active tester maps those
actions to its symbols. Timing waves determine when drive and compare events
occur within a cycle.

Derivatives
-----------

Package- or product-specific pins belong in the appropriate DUT derivative. Put
only truly shared pins in the parent DUT model. This keeps a package change from
silently affecting unrelated devices.

Testing
-------

Test pin widths, indexed names, aliases, header membership and order, reset
actions, and tester symbol mappings. A missing alias or reordered header can
change every vector while leaving pattern source untouched.
