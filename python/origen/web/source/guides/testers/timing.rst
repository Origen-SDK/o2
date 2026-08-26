Timing
======

Timing connects model pin actions to tester drive and compare edges. DUT timing
source defines timesets and waves; a pattern selects a timeset before cycling;
the active tester translates events into its native timing representation.

Modeling Timing
---------------

See :doc:`../device_modeling/timesets` for timeset, wavetable, wave, and event
construction. Keep wave actions semantic—``DriveHigh``, ``DriveLow``,
``VerifyHigh``, ``VerifyLow``, and ``HighZ``—so multiple testers can translate
the same model.

Selecting A Timeset
-------------------

Select by name or timeset object before emitting cycles:

.. code-block:: python

   origen.tester.set_timeset("functional")
   origen.dut.pin("clk").drive(1)
   origen.tester.cycle()

Origen lazily loads DUT timesets when necessary. A missing-timeset error usually
means the active DUT target does not define that name or a derivative failed to
load its timing source.

Periods
-------

Timesets provide a default period. Applications may use expressions based on
``period`` for edge positions. Validate the resolved values for every tester;
native tools can differ in supported resolution and edge ordering.

Symbol Mapping
--------------

The tester backend has default symbols for pin actions, and a timeset may
override them. A symbol change affects vectors without changing pattern source,
so symbol maps require regression coverage.

Validation
----------

For every supported tester, verify:

* the selected timeset exists;
* all pattern symbols resolve to waves;
* edge expressions resolve inside the period;
* drive and compare actions map to intended native edges; and
* generated timing resources and vector symbols agree.
