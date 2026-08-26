Bits And Registers
==================

Registers describe addressable DUT state. Fields add names, access policies,
reset values, and enumerations to bit ranges. Pattern code then operates on those
semantic objects instead of assembling raw transactions repeatedly.

Bits
----

A field selects one or more bits within a register. Its ``offset`` is the least
significant bit position and ``width`` defaults to one. Origen tracks data,
verify masks, access policy, and reset information for the resulting bit
collection.

Use field and register APIs to stage data before dispatching a transaction. The
controller determines how the transaction reaches the DUT.

Registers
---------

Simple Registers
^^^^^^^^^^^^^^^^

Use ``SimpleReg`` when every bit shares the default behavior:

.. code-block:: python

   SimpleReg("control", 0x00)          # Defaults to 32 bits
   SimpleReg("status", 0x04, size=16)

Registers declared without an explicit map are placed in the default memory map
and default address block.

Fields
^^^^^^

Use ``Reg`` as a context manager for field declarations:

.. code-block:: python

   with Reg("adc_cfg", 0x24, size=16):
       Field("complete", offset=7, access="ro")
       Field("irq_enable", offset=6)
       Field(
           "channel",
           offset=0,
           width=5,
           reset=0x1F,
           enums={
               "temperature": 3,
               "bandgap": {
                   "value": 5,
                   "usage": "w",
                   "description": "Internal bandgap channel",
               },
           },
       )

Use access values such as ``ro`` only when the hardware policy differs from the
default. Enumerations make patterns readable and centralize legal values.

Reset Values
^^^^^^^^^^^^

``reset`` defines the common hard-reset value. Use ``resets`` when a field has
multiple reset domains or a partial reset mask:

.. code-block:: python

   with Reg("mode", 0x28, size=16):
       Field(
           "select",
           offset=0,
           width=5,
           resets={
               "hard": 5,
               "async": {"value": 0xF, "mask": 0b1010},
           },
       )

Memory Maps And Address Blocks
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Use explicit scopes when a block has multiple buses or banks:

.. code-block:: python

   with MemoryMap("debug"):
       with AddressBlock("bank0"):
           SimpleReg("control", 0x00)

Names may be reused in different maps or address blocks. Code that relies on a
non-default scope should use the fully qualified model path.

Transactions
^^^^^^^^^^^^

Calling a register write or verify sends the modeled value, size, and address to
the owning block controller. Keep bus sequences in the controller so the same
register declarations can be reused by different DUT derivatives and testers.

Testing
^^^^^^^

Verify register size, field offsets, reset values, enum mappings, and absolute
addresses. Include negative tests for overlapping fields, invalid values, and
write attempts to read-only fields where the API enforces them.
