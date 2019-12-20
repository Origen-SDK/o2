import origen

def test_memory_maps():
    origen.app.instantiate_dut("dut.falcon")
    assert origen.dut.memory_maps
    assert origen.dut.memory_map("default") == origen.dut.memory_maps["default"]
    assert len(origen.dut.memory_maps) == 1

    # Check some of the dict-like API
    assert "default" in origen.dut.memory_maps
    keys = origen.dut.memory_maps.keys()
    assert set(keys) == set(['default'])
    values = origen.dut.memory_maps.values()
    assert len(values) == 1
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


def test_address_blocks_can_be_fetched():
    pass

def test_regs_can_be_fetched():
    pass

def test_register_value_can_be_read():
    origen.app.instantiate_dut("dut.falcon")
    #assert origen.dut.regs["reg1"].data == 0


