# The DUT Data Model

The DUT is the central data store for a device in Origen.

One instance of a DUT is created per target load in Origen and it contains all metadata and
state associated with the current device target. e.g. pins, registers, etc.

One of the challenges with Rust programming vs. something like Python is that you cannot easily
create the equivalent of instance variables on the Rust side to maintain state between function
calls.
When a call is made to a Rust function from Python application code, all variables created by
the Rust function are created on the stack and then at the end of the function the stack is
un-wound and all the variables are lost. The next function call starts from scratch again.
There are some ways around this by the use of statics, but generally the advice from Rust
experts is to avoid this except when dealing with immutable, truly global data. We use this
approach in Origen for things like the Origen and application configs which are constants, or
singletons like the LOGGER.
One of the problems that makes this approach not suitable for storing the DUT data is that
static memory cannot be reclaimed, so an application that consumed a large amount of memory
from a DUT model and then switched targets, would now have n DUTs in memory, which could
become a real problem when the DUTs are large SoCs.

In Origen we obviously need a solution to store the DUT in memory between function calls and
the way it is handled is as follows:

* This DUT struct has a corresponding struct in the Python API which instantiates a single
  instance of this DUT struct:

  ~~~rust
  struct PyDUT {
      dut: DUT,
  }
  ~~~

* PyDUT is instantiated on the Python side during target loading, meaning that the memory
  associated with it is owned by Python and the Python garbage collector will take care of
  freeing it whenever targets are switched and it goes out of scope.
* All calls to Rust functions, for example to add a register to the model or to get the
  current value of a register, must pass in a reference to this DUT object; thereby giving the
  Rust function access to the current DUT struct.
* Additionally, some functions may take a 2nd mandatory argument to specify which child block
  of the DUT to act on. E.g. adding a register:

  ~~~python
  # Add a 32-bit reg to the DUT top-level, note the presence of `dut.model` as the 1st argument
  # which is the current instance of PyDUT and `None` as the 2nd argument which means no
  # reference to a child node.
  _origen.dut.add_reg(dut.model, None, "my_reg", 0x0, 32)

  # Add a similar reg to a child block of the DUT, in this case the block that is instantiated
  # at `dut.adc0`
  _origen.dut.add_reg(dut.model, "adc0", "my_reg", 0x0, 32)
  ~~~

* Higher level APIs on the Python side will hide the passing of `dut.model` and hierarchical
  references from the user, that will all be done in the background depending on which object
  they are acting on.
* The data in the `DUT` struct will be organized to make the resolution of hierarchical
  references as efficient as possible. 

