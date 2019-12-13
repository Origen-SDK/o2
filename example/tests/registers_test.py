import origen

def test_memory_maps_can_be_fetched():
    origen.app.instantiate_dut("dut.falcon")
    assert origen.dut.memory_maps
    assert len(origen.dut.memory_maps) == 1
    assert origen.dut.memory_map("default") == origen.dut.memory_maps["default"]

def test_address_blocks_can_be_fetched():
    pass

def test_regs_can_be_fetched():
    pass

def test_register_value_can_be_read():
    origen.app.instantiate_dut("dut.falcon")
    #assert origen.dut.regs["reg1"].data == 0

