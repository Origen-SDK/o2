import origen

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
    assert origen.dut.regs.len() == 4
    origen.dut.add_simple_reg("treg1", 0x1000)
    assert origen.dut.regs.len() == 5
    with origen.dut.add_reg("treg2", 0x1004) as reg:
        reg.bit([7,0], 'trim')
    assert origen.dut.regs.len() == 6

def test_address_blocks_can_be_fetched():
    origen.app.instantiate_dut("dut.falcon")
    assert origen.dut
    pass

def test_regs_can_be_fetched():
    assert origen.dut.reg("reg1")
    assert origen.dut.reg("no_reg") == None
    #assert origen.dut.regs["reg1"]

def test_register_value_can_be_read():
    origen.app.instantiate_dut("dut.falcon")
    #assert origen.dut.regs["reg1"].data == 0


