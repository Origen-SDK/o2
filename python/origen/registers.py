import origen
from contextlib import contextmanager

# A middleman between the Python controller and the associated Rust model and
# which implements the application/user API for working with registers.
# An instance of this class is returned by <my_controller>.regs
class Proxy:
    def __init__(self, controller):
        self.controller = controller

    # Returns the number of contained registers
    def len(self):
        return origen.dut.db.number_of_regs(self.controller.path);

    #def __repr__(self):
    #    return f"Registers in {self.controller.block_path}:\n0  : Reg 1\n4  : Reg 2"

# This defines the methods for defining registers in Python and then handles serializing
# the definitions and handing them over to the Rust model for instantiation.
class Loader:
    def __init__(self, controller):
        self.controller = controller
        self.memory_map = None
        self.address_block = None

    @contextmanager
    def Reg(self, id, address_offset, size=32):
        origen.dut.db.create_reg(self.controller.path, self.memory_map, self.address_block, id, address_offset, size);
        yield self

    def SimpleReg(self, id, address_offset, size=32):
        origen.dut.db.create_reg(self.controller.path, self.memory_map, self.address_block, id, address_offset, size);

    def bit(self, number, id, access="rw", reset=0):
        pass

    @contextmanager
    def MemoryMap(self, id):
        yield self

    # Defines the methods that are accessible within blocks/<block>/registers.py
    def api(self):
        return {
            "Reg": self.Reg, 
            "SimpleReg": self.SimpleReg, 
            "MemoryMap": self.MemoryMap, 
        }
