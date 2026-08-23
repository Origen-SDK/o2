Hierarchy
=========

Top-Level
---------

The top-level block is the DUT selected by a target. It provides the root address
space and owns shared pins, timing, and services. The active instance is exposed
as ``origen.dut``.

Use the common DUT block for resources shared by all derivatives and place
device-specific changes below ``blocks/dut/derivatives/<name>/``. This avoids
copying complete models for closely related products.

SubBlocks
---------

Declare child instances in ``sub_blocks.py`` with ``SubBlock``:

.. code-block:: python

   SubBlock("core0", "core")
   SubBlock("core1", "core", offset=0x1000_0000)
   SubBlock("adc", "adc.16_bit", offset=0x2000_0000)

The first argument is the instance name. The second identifies the block model.
An optional ``offset`` relocates the child's local address space in its parent.
The same block model can therefore be instantiated multiple times at different
addresses.

Plugin Blocks
^^^^^^^^^^^^^

Prefix the block path with a plugin package to instantiate reusable content:

.. code-block:: python

   SubBlock("dac", "company_common.dac", offset=0x8000_0000)

Some reusable services expose a class name or options:

.. code-block:: python

   SubBlock(
       "ram",
       "origen.memories",
       class_name="RAM",
       offset=0x1_0000,
       sb_options={"length": 0x1000},
   )

Paths And Addressing
--------------------

Resources are addressed through their instance path, for example
``dut.core1.adc0``. A register's absolute address is derived from its local
address plus all parent offsets. Keep the register definition local to its block;
do not bake a particular instance offset into reusable register source.

Controllers
-----------

Register writes and verifies flow through block controllers. A child controller
normally forwards a request to its parent until a controller implements the
transport, such as JTAG, SWD, or a memory access port. This lets one register
model work with different access strategies.

Validation Checklist
--------------------

Test at least one instance path and absolute address for every reusable block,
especially when the same model is instantiated more than once. Also test plugin
availability and required ``sb_options`` so missing dependencies fail early.
