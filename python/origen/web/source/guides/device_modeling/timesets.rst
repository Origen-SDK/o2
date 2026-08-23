Timesets
========

A timeset defines the period and pin behavior for one tester cycle. Wavetables
contain wave groups, waves map symbols to pins, and events describe actions at
specific points in the period.

Simple Timesets
---------------

Create a timeset in ``timing.py``:

.. code-block:: python

   Timeset("simple", default_period=10)

Periods are resolved when a pattern is generated. Keep units consistent with
the application and tester timing conventions.

Wavetables And Waves
--------------------

Build explicit waveforms when pins need drive or compare edges:

.. code-block:: python

   t = Timeset("functional", default_period=40)
   wtbl = t.add_wavetable("default")
   ports = wtbl.add_waves("Ports")

   drive_high = ports.add_wave("1")
   drive_high.apply_to("porta", "portb")
   drive_high.push_event(
       at="period*0.25",
       unit="ns",
       action=drive_high.DriveHigh,
   )

   compare_high = ports.add_wave("H")
   compare_high.apply_to("porta", "portb")
   compare_high.push_event(
       at="period*0.10",
       unit="ns",
       action=compare_high.VerifyHigh,
   )

The ``at`` expression may reference ``period``. The tester backend resolves the
semantic action into its native edge and symbol representation.

Clock Waves
-----------

A clock wave normally contains multiple events in one cycle:

.. code-block:: python

   clocks = wtbl.add_waves("Clocks")
   clock = clocks.add_wave("1")
   clock.apply_to("clk")
   clock.push_event(at=0, unit="ns", action=clock.DriveHigh)
   clock.push_event(at="period/2", unit="ns", action=clock.DriveLow)

Symbol Maps
-----------

Timesets can override the symbols emitted for model actions:

.. code-block:: python

   t.symbol_map["1"] = "0"
   t.symbol_map["0"] = "1"
   t.symbol_map[origen.pins.PinActions.VerifyHigh()] = "L"
   t.symbol_map[origen.pins.PinActions.VerifyLow()] = "H"

Use overrides sparingly and document why the tester or protocol requires them.

Tester Translation
------------------

The same timeset model can feed different tester backends, but not every tester
supports every edge or expression. Consult the tester-specific guide and keep
approved output for each supported target.

Testing
-------

Test period evaluation, pin applicability, event ordering, symbol mappings, and
generated timing resources. Include boundary cases such as events at time zero,
half-period expressions, and incompatible drive/compare combinations.
