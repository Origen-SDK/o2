import origen
import _origen
from origen.errors import *
from contextlib import contextmanager
from inspect import getframeinfo, stack
import pdb


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
            return origen.dut.db.get_or_create_memory_map(
                self.controller.model_id, "default")

    def current_address_block(self):
        if self.address_block is not None:
            return self.address_block

        elif self.memory_map is None and self.address_block is None:
            if self.controller._default_default_address_block is None:
                mm = origen.dut.db.get_or_create_memory_map(
                    self.controller.model_id, "default")
                ab = origen.dut.db.get_or_create_address_block(
                    mm.id, "default")
                self.controller._default_default_address_block = ab
                return ab
            else:
                return self.controller._default_default_address_block
        else:
            return origen.dut.db.get_or_create_address_block(
                self.memory_map.id, "default")

    @contextmanager
    def Reg(self,
            name,
            address_offset,
            size=32,
            bit_order="lsb0",
            _called_from_controller=False,
            description=None,
            reset=None,
            resets=None,
            access=None):
        if origen._reg_description_parsing:
            if _called_from_controller:
                caller = getframeinfo(stack()[4][0])
            else:
                caller = getframeinfo(stack()[2][0])
            filename = caller.filename
            lineno = caller.lineno
        else:
            filename = None
            lineno = None

        self.fields = []
        yield self
        # TODO: The None here is for an optional register file ID, which is not hooked up yet
        reg = _origen.dut.registers.create(self.current_address_block().id,
                                           None, name, address_offset, size,
                                           bit_order, self.fields, filename,
                                           lineno, description,
                                           self.clean_resets(reset,
                                                             resets), access)
        self.fields = None

    def SimpleReg(self,
                  name,
                  address_offset,
                  size=32,
                  reset=None,
                  resets=None,
                  enums=None,
                  bit_order="lsb0",
                  _called_from_controller=False,
                  description=None,
                  access=None):
        if origen._reg_description_parsing:
            if _called_from_controller:
                caller = getframeinfo(stack()[2][0])
            else:
                caller = getframeinfo(stack()[1][0])
            filename = caller.filename
            lineno = caller.lineno
        else:
            filename = None
            lineno = None
        field = _origen.dut.registers.Field("data", "", 0, size, access,
                                            self.clean_resets(reset, resets),
                                            self.clean_enums(enums))
        # TODO: The None here is for an optional register file ID, which is not hooked up yet
        _origen.dut.registers.create(self.current_address_block().id, None,
                                     name, address_offset, size, bit_order,
                                     [field], filename, lineno, description,
                                     None, None)

    def Field(self,
              name,
              offset,
              width=1,
              access=None,
              reset=None,
              resets=None,
              enums=None,
              description=None):
        if origen._reg_description_parsing:
            caller = getframeinfo(stack()[1][0])
            filename = caller.filename
            lineno = caller.lineno
        else:
            filename = None
            lineno = None
        if self.fields is not None:
            self.fields.append(
                _origen.dut.registers.Field(name, description, offset, width,
                                            access,
                                            self.clean_resets(reset, resets),
                                            self.clean_enums(enums), filename,
                                            lineno))
        else:
            raise RuntimeError(
                f"A Field can only be defined within a 'with Reg' definition block"
            )

    @contextmanager
    def MemoryMap(self, name):
        if self.memory_map is not None:
            raise RuntimeError(
                f"Attempted to open memory map '{name}' when memory map '{self.memory_map.name}' is already open"
            )
        self.memory_map = origen.dut.db.get_or_create_memory_map(
            self.controller.model_id, name)
        yield self
        self.memory_map = None

    @contextmanager
    def AddressBlock(self, name):
        if self.address_block is not None:
            raise RuntimeError(
                f"Attempted to open address block '{name}' when address block '{self.address_block.name}' is already open"
            )
        self.address_block = origen.dut.db.get_or_create_address_block(
            self.current_memory_map().id, name)
        yield self
        self.address_block = None

    def clean_enums(self, enums):
        e = []
        if enums is not None:
            for enum_name, attrs in enums.items():
                if isinstance(attrs, dict):
                    #e.append(_origen.dut.registers.FieldEnum(enum_name, attrs.get("description"), attrs.get("usage", "rw"), attrs["value"]))
                    e.append(
                        _origen.dut.registers.FieldEnum(
                            enum_name, attrs.get("description", ""),
                            attrs["value"]))
                else:
                    #e.append(_origen.dut.registers.FieldEnum(enum_name, "", "rw", attrs))
                    e.append(
                        _origen.dut.registers.FieldEnum(enum_name, "", attrs))
        return e

    def clean_resets(self, reset, resets):
        r = None
        if resets is None:
            resets = reset
        if resets is not None:
            r = []
            if isinstance(resets, dict):
                for reset_name, attrs in resets.items():
                    if isinstance(attrs, dict):
                        r.append(
                            _origen.dut.registers.ResetVal(
                                reset_name, attrs["value"], attrs.get("mask")))
                    else:
                        r.append(
                            _origen.dut.registers.ResetVal(
                                reset_name, attrs, None))
            else:
                r.append(_origen.dut.registers.ResetVal("hard", resets, None))

        return r

    # Defines the methods that are accessible within blocks/<block>/registers.py
    def api(self):
        return {
            "Reg": self.Reg,
            "Field": self.Field,
            "SimpleReg": self.SimpleReg,
            "MemoryMap": self.MemoryMap,
            "AddressBlock": self.AddressBlock,
        }
