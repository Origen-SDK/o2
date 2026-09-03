Basics
======

An Origen device model is a hierarchy of blocks. The top-level block represents
the DUT and every child represents a reusable functional block, memory, debug
service, or other addressable component.

Block Source Layout
-------------------

A generated block can contain these source files:

``attributes.py``
   Static metadata and model attributes.

``sub_blocks.py``
   Child block instances and address offsets.

``registers.py``
   Memory maps, address blocks, registers, and fields.

``pins.py``
   Physical pins, aliases, groups, and pin headers.

``timing.py``
   Timesets, wavetables, waves, and events.

``controller.py``
   Register transaction behavior and other block-specific control logic.

``services.py`` and ``levels.py``
   Optional services and electrical-level definitions.

These files are evaluated by Origen in a model-building context. Constructors
such as ``Reg``, ``Field``, ``Pin``, and ``SubBlock`` are supplied by that
context; application model files do not normally import them.

Targets Instantiate Models
--------------------------

Defining a block does not make it the active DUT. A DUT target selects the model
for an invocation. Tester targets are typically loaded alongside it:

.. code-block:: console

   $ origen target set dut/eagle tester/smt7
   $ origen target view

After target loading, the active model is available through ``origen.dut``.
Generation code should query the model rather than instantiate a second copy.

Inheritance And Derivatives
---------------------------

Common content belongs in a parent block. A derivative adds or changes only the
content which differs for that device. For example, common DUT registers may be
defined in ``blocks/dut/registers.py`` while package-specific pins live in
``blocks/dut/derivatives/eagle/pins.py``.

Modeling Guidelines
-------------------

* Give blocks and resources stable, lowercase identifiers.
* Keep addresses local to a block and use sub-block offsets for composition.
* Model semantic intent; keep tester formatting in tester implementations.
* Put transaction behavior in controllers rather than register declarations.
* Add tests for hierarchy paths, addresses, resets, aliases, and timing symbols.
* Treat model changes as generated-output changes and review their references.
