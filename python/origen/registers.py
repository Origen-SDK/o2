import origen
<<<<<<< HEAD
import _origen
=======
from origen.errors import *
>>>>>>> master
from contextlib import contextmanager

# This defines the API for defining registers in Python and then handles serializing
# the definitions and handing them over to the Rust model for instantiation.
class Loader:
    def __init__(self, controller):
        self.controller = controller
        self.memory_map = None
        self.address_block = None
        self.fields = None

    def current_memory_map(self):
        if self.memory_map is not None:
            return self.memory_map
        else:
            return origen.dut.db.get_or_create_memory_map(self.controller.model_id, "default")

    def current_address_block(self):
        if self.address_block is not None:
            return self.address_block

        elif self.memory_map is None and self.address_block is None:
            if self.controller._default_default_address_block is None:
                mm = origen.dut.db.get_or_create_memory_map(self.controller.model_id, "default")
                ab = origen.dut.db.get_or_create_address_block(mm.id, "default")
                self.controller._default_default_address_block = ab
                return ab
            else:
                return self.controller._default_default_address_block
        else:
            return origen.dut.db.get_or_create_address_block(self.memory_map.id, "default")

    @contextmanager
    def Reg(self, name, address_offset, size=32):
        self.fields = []
        yield self
        _origen.dut.registers.create(self.current_address_block().id, name, address_offset, size, self.fields)
        self.fields = None

    def SimpleReg(self, name, address_offset, size=32, reset=0):
        field = _origen.dut.registers.Field("data", "", 0, size, "rw", reset, [])
        _origen.dut.registers.create(self.current_address_block().id, name, address_offset, size, [field])

    def Field(self, name, offset, width=1, access="rw", reset=0, enums=None, description=""):
        if self.fields is not None:
            e = []
            if enums is not None:
                for enum_name, attrs in enums.items():
                    if isinstance(attrs, dict):
                        #e.append(_origen.dut.registers.FieldEnum(enum_name, attrs.get("description"), attrs.get("usage", "rw"), attrs["value"]))
                        e.append(_origen.dut.registers.FieldEnum(enum_name, attrs.get("description", ""), attrs["value"]))
                    else:
                        #e.append(_origen.dut.registers.FieldEnum(enum_name, "", "rw", attrs))
                        e.append(_origen.dut.registers.FieldEnum(enum_name, "", attrs))
            self.fields.append(_origen.dut.registers.Field(name, description, offset, width, access, reset, e))
        else:
            raise RuntimeError(f"A Field can only be defined within a 'with Reg' definition block")

    @contextmanager
    def MemoryMap(self, name):
        if self.memory_map is not None:
            raise RuntimeError(f"Attempted to open memory map '{name}' when memory map '{self.memory_map.name}' is already open")
        self.memory_map = origen.dut.db.get_or_create_memory_map(self.controller.model_id, name)
        yield self
        self.memory_map = None

    @contextmanager
    def AddressBlock(self, name):
        if self.address_block is not None:
            raise RuntimeError(f"Attempted to open address block '{name}' when address block '{self.address_block.name}' is already open")
        self.address_block = origen.dut.db.get_or_create_address_block(self.current_memory_map().id, name)
        yield self
        self.address_block = None

    # Defines the methods that are accessible within blocks/<block>/registers.py
    def api(self):
        return {
            "Reg": self.Reg, 
            "Field": self.Field, 
            "SimpleReg": self.SimpleReg, 
            "MemoryMap": self.MemoryMap, 
            "AddressBlock": self.AddressBlock, 
        }
