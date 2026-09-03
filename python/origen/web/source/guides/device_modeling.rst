Device Modeling
===============

The device model is the shared source of truth for DUT structure, registers,
pins, and timing. Patterns and test-program interfaces operate on this model
rather than duplicating tester-specific descriptions.

Model source belongs under the application's ``blocks/`` tree. A target selects
and instantiates the top-level DUT; block files then contribute attributes,
sub-blocks, registers, pins, services, and timing. Derivative directories extend
or override a reusable parent model.

The following pages build the model from its structural foundation outward.

.. toctree::
   
   device_modeling/basics
   device_modeling/hierarchy
   device_modeling/bits_and_registers
   device_modeling/pins
   device_modeling/timesets
