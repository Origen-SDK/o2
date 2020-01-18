import origen
from origen.errors import *
import pytest
import pdb

def test_memory_maps():
    origen.app.instantiate_dut("dut.falcon")
    assert origen.dut.memory_maps
    assert origen.dut.memory_map("default") == origen.dut.memory_maps["default"]
    assert len(origen.dut.memory_maps) == 3
    assert len(origen.dut.core0.memory_maps) == 0
    assert len(origen.dut.core0.adc0.memory_maps) == 1
    # Simple test to make sure the display method doesn't hang/crash
    assert origen.dut.memory_maps.__repr__()
    assert origen.dut.core0.memory_maps.__repr__()
    assert origen.dut.core0.adc0.memory_maps.__repr__()

    # Check some of the dict-like API
    assert "default" in origen.dut.memory_maps
    keys = origen.dut.memory_maps.keys()
    assert set(keys) == set(['default', 'user', 'test'])
    values = origen.dut.memory_maps.values()
    assert len(values) == 3
    assert type(values[0]).__name__ == "MemoryMap"
    for k in origen.dut.memory_maps:
        assert k in keys
    d = dict(origen.dut.memory_maps)
    assert isinstance(d, dict)
    assert type(d["default"]).__name__ == "MemoryMap"
    for k, v in origen.dut.memory_maps.items():
        assert k in keys
        assert isinstance(k, str)
        assert type(v).__name__ == "MemoryMap"

    assert origen.dut.memory_maps.user.regs.len() == 2

def test_address_blocks():
    origen.app.instantiate_dut("dut.falcon")
    assert origen.dut.default.address_blocks
    assert origen.dut.default.address_block("default") == origen.dut.default.address_blocks["default"]
    #assert len(origen.dut.memory_maps) == 3
    #assert len(origen.dut.core0.memory_maps) == 0
    #assert len(origen.dut.core0.adc0.memory_maps) == 1
    ## Simple test to make sure the display method doesn't hang/crash
    #assert origen.dut.memory_maps.__repr__()
    #assert origen.dut.core0.memory_maps.__repr__()
    #assert origen.dut.core0.adc0.memory_maps.__repr__()

    ## Check some of the dict-like API
    #assert "default" in origen.dut.memory_maps
    #keys = origen.dut.memory_maps.keys()
    #assert set(keys) == set(['default', 'user', 'test'])
    #values = origen.dut.memory_maps.values()
    #assert len(values) == 3
    #assert type(values[0]).__name__ == "MemoryMap"
    #for k in origen.dut.memory_maps:
    #    assert k in keys
    #d = dict(origen.dut.memory_maps)
    #assert isinstance(d, dict)
    #assert type(d["default"]).__name__ == "MemoryMap"
    #for k, v in origen.dut.memory_maps.items():
    #    assert k in keys
    #    assert isinstance(k, str)
    #    assert type(v).__name__ == "MemoryMap"

    #assert origen.dut.memory_maps.user.regs.len() == 2
    #
    #assert origen.dut.memory_maps.test.bank0.regs.len() == 2

def test_regs_can_be_added():
    origen.app.instantiate_dut("dut.falcon")
    base = origen.dut.regs.len()
    origen.dut.add_simple_reg("treg1", 0x1000)
    assert origen.dut.regs.len() == base + 1
    with origen.dut.add_reg("treg2", 0x1004) as reg:
        reg.Field('trim', offset=0, width=8)
    assert origen.dut.regs.len() == base + 2

def test_address_blocks_can_be_fetched():
    origen.app.instantiate_dut("dut.falcon")
    assert origen.dut
    pass

def test_regs_can_be_fetched():
    assert origen.dut.reg("reg1")
    assert origen.dut.reg("no_reg") == None
    #assert origen.dut.regs["reg1"]

def test_register_reset_values():
    origen.app.instantiate_dut("dut.falcon")

    with origen.dut.add_reg("t1", 0) as reg:
        reg.Field("f1", offset=0, width=8, reset=0x55)
    assert origen.dut.t1.f1.data() == 0x55

    with origen.dut.add_reg("t2", 0) as reg:
        reg.Field("f1", offset=0, width=8, resets={
            "hard": 0xAA,
            "soft": 0xFF,
        })
    assert origen.dut.t2.f1.data() == 0xAA

    with origen.dut.add_reg("t3", 0) as reg:
        reg.Field("f1", offset=0, width=8, resets={
            "hard": { "value": 0xAA, "mask": 0xF0 },
            "soft": 0xFF,
        })
    assert origen.dut.t3.f1.data() == 0xA0
        

def test_reading_undefined_data_raises_error():
    origen.app.instantiate_dut("dut.falcon")
    origen.dut.add_simple_reg("t1", 0)
    reg = origen.dut.t1
    with pytest.raises(UndefinedDataError):
        reg.data()
    reg.set_data(0)
    assert reg.data() == 0