import origen
from origen.errors import *
import pytest
from tests.shared import *


@pytest.fixture(autouse=True)
def run_around_tests():
    global dut
    # Code that will run before each test
    instantiate_dut("dut.falcon")
    dut = origen.dut
    yield
    # Code that will run after each test


def test_memory_maps():
    assert dut.memory_maps
    assert dut.memory_map("default") == dut.memory_maps["default"]
    assert len(dut.memory_maps) == 3
    assert len(dut.core0.memory_maps) == 0
    assert len(dut.core0.adc0.memory_maps) == 1
    # Simple test to make sure the display method doesn't hang/crash
    assert dut.memory_maps.__repr__()
    assert dut.core0.memory_maps.__repr__()
    assert dut.core0.adc0.memory_maps.__repr__()

    # Check some of the dict-like API
    assert "default" in dut.memory_maps
    keys = dut.memory_maps.keys()
    assert set(keys) == set(['default', 'user', 'test'])
    values = dut.memory_maps.values()
    assert len(values) == 3
    assert type(values[0]).__name__ == "MemoryMap"
    for k in dut.memory_maps:
        assert k in keys
    d = dict(dut.memory_maps)
    assert isinstance(d, dict)
    assert type(d["default"]).__name__ == "MemoryMap"
    for k, v in dut.memory_maps.items():
        assert k in keys
        assert isinstance(k, str)
        assert type(v).__name__ == "MemoryMap"

    assert dut.memory_maps.user.regs.len() == 2


def test_address_blocks():
    assert dut.default.address_blocks
    assert dut.default.address_block(
        "default") == dut.default.address_blocks["default"]
    #assert len(dut.memory_maps) == 3
    #assert len(dut.core0.memory_maps) == 0
    #assert len(dut.core0.adc0.memory_maps) == 1
    ## Simple test to make sure the display method doesn't hang/crash
    #assert dut.memory_maps.__repr__()
    #assert dut.core0.memory_maps.__repr__()
    #assert dut.core0.adc0.memory_maps.__repr__()

    ## Check some of the dict-like API
    #assert "default" in dut.memory_maps
    #keys = dut.memory_maps.keys()
    #assert set(keys) == set(['default', 'user', 'test'])
    #values = dut.memory_maps.values()
    #assert len(values) == 3
    #assert type(values[0]).__name__ == "MemoryMap"
    #for k in dut.memory_maps:
    #    assert k in keys
    #d = dict(dut.memory_maps)
    #assert isinstance(d, dict)
    #assert type(d["default"]).__name__ == "MemoryMap"
    #for k, v in dut.memory_maps.items():
    #    assert k in keys
    #    assert isinstance(k, str)
    #    assert type(v).__name__ == "MemoryMap"

    #assert dut.memory_maps.user.regs.len() == 2
    #
    #assert dut.memory_maps.test.bank0.regs.len() == 2


def test_regs_can_be_added():
    base = dut.regs.len()
    dut.add_simple_reg("treg1", 0x1000)
    assert dut.regs.len() == base + 1
    with dut.add_reg("treg2", 0x1004) as reg:
        reg.Field('trim', offset=0, width=8)
    assert dut.regs.len() == base + 2


def test_address_blocks_can_be_fetched():
    assert dut
    pass


def test_regs_can_be_fetched():
    assert dut.reg("reg1")
    assert dut.reg("no_reg") == None
    #assert dut.regs["reg1"]


def test_register_reset_values():
    with dut.add_reg("t1", 0) as reg:
        reg.Field("f1", offset=0, width=8, reset=0x55)
    assert dut.t1.f1.data() == 0x55

    with dut.add_reg("t2", 0) as reg:
        reg.Field("f1",
                  offset=0,
                  width=8,
                  resets={
                      "hard": 0xAA,
                      "soft": 0xFF,
                  })
    assert dut.t2.f1.data() == 0xAA

    with dut.add_reg("t3", 0) as reg:
        reg.Field("f1",
                  offset=0,
                  width=8,
                  resets={
                      "hard": {
                          "value": 0xAA,
                          "mask": 0xF0
                      },
                      "soft": 0xFF,
                  })
    assert dut.t3.f1.data() == 0xA0


def test_reading_undefined_data_raises_error():
    dut.add_simple_reg("t1", 0)
    reg = dut.t1
    with pytest.raises(UndefinedDataError):
        reg.data()
    reg.set_data(0)
    assert reg.data() == 0


def test_reg_bits_attr_returns_a_list():
    with dut.add_reg("tr1", 0x0, size=8) as reg:
        reg.Field("b0", offset=5, reset=1)
        reg.Field("b1", offset=0, width=4, reset=3)
    reg = dut.tr1
    assert isinstance(reg.bits, list)
    assert len(reg.bits) == 8
    reg.set_data(0)
    reg.bits[0].set_data(1)
    reg.bits[2].set_data(1)
    reg.bits[6].set_data(1)  # is an unimplemented bit
    assert reg.data() == 0b101


def test_reg_fields_attr_returns_a_dict():
    with dut.add_reg("tr1", 0x0, size=8) as reg:
        reg.Field("b0", offset=5, reset=1)
        reg.Field("b1", offset=0, width=4, reset=3)
    reg = dut.tr1
    assert isinstance(reg.fields, dict)
    assert reg.fields["b0"].len() == 1
    assert reg.fields["b1"].len() == 4
    reg.set_data(0)
    reg.fields["b0"].set_data(0b1)
    reg.fields["b1"].set_data(0b1100)
    assert reg.data() == 0b101100


def test_msb0_behavior():
    with dut.add_reg("tr1", 0, size=16, bit_order="msb0") as reg:
        reg.Field("coco", offset=7, access="ro", reset=0)
        reg.Field("aien", offset=6, reset=0)
        reg.Field("diff", offset=5, reset=0)
        reg.Field("adch", offset=0, width=5, reset=0x1F)
    reg = dut.tr1
    assert reg.data() == 0xF800
    reg.set_data(0)
    assert reg.diff.data() == 0
    reg[10].set_data(1)
    assert reg.diff.data() == 1
    reg.set_data(0)
    assert reg.diff.data() == 0
    reg.with_msb0()[5].set_data(1)
    assert reg.diff.data() == 1
    assert reg.data() == 0x400
    assert reg.with_msb0().data() == 0x400
    reg.adch.set_data(0)
    reg[13:11].set_data(7)
    assert reg.adch.data() == 7
    reg.adch.set_data(0)
    assert reg.adch.data() == 0
    reg.with_msb0()[2:4].set_data(7)
    assert reg.adch.data() == 7


def test_filename_and_lineno():
    if origen.status["on_windows"]:
        assert "example\\blocks\\dut\\registers.py" in dut.breg0.filename
        assert "example\\blocks\\dut\\registers.py" in dut.reg1.filename
    else:
        assert "example/blocks/dut/registers.py" in dut.breg0.filename
        assert "example/blocks/dut/registers.py" in dut.reg1.filename
    with origen.reg_description_parsing():
        # Test a locally defined register to verify that the user can add info about the file
        with dut.add_reg("tr1", 0x0, size=8) as reg:
            reg.Field("b0", offset=5, reset=1)
            reg.Field("b1", offset=0, width=4, reset=3)
        dut.add_simple_reg("tr2", 0x1000)

    if origen.status["on_windows"]:
        assert "tests\\registers_test.py" in dut.tr1.filename
        assert "tests\\registers_test.py" in dut.tr2.filename
    else:
        assert "tests/registers_test.py" in dut.tr1.filename
        assert "tests/registers_test.py" in dut.tr2.filename


def test_register_dirty_tracking():
    dut.add_simple_reg("treg1", 0x1000, reset=0)
    reg = dut.treg1

    assert reg.data() == 0
    assert reg.is_modified_since_reset() == False
    assert reg.is_in_reset_state() == True
    reg.set_data(0x1234)
    assert reg.is_modified_since_reset() == True
    assert reg.is_in_reset_state() == False
    reg.set_data(0)
    assert reg.is_modified_since_reset() == True
    assert reg.is_in_reset_state() == True
    reg.reset()
    assert reg.is_modified_since_reset() == False
    assert reg.is_in_reset_state() == True


def test_snapshots():
    dut.add_simple_reg("treg1", 0x1000, reset=0)
    reg = dut.treg1

    reg.set_data(0x1234)
    reg.set_overlay("blah")
    reg.snapshot("snap1")
    assert reg.is_changed("snap1") == False
    reg.set_data(0xFFFF)
    reg.snapshot("snap2")
    assert reg.is_changed("snap1") == True
    assert reg.is_changed("snap2") == False
    reg.rollback("snap1")
    assert reg.data() == 0x1234
    assert reg.is_changed("snap1") == False
    assert reg.is_changed("snap2") == True
    reg.rollback("snap1")
    assert reg.is_changed("snap1") == False
    reg.clear_overlay()
    assert reg.is_changed("snap1") == True
    assert reg.get_overlay() == None
    reg.rollback("snap1")
    assert reg.get_overlay().label == "blah"


def test_x_bits_reset_correctly():
    assert dut.areg0.aien.has_known_value() == False
    assert dut.areg0.set_data(0)
    assert dut.areg0.aien.has_known_value() == True
    assert dut.areg0.reset()
    assert dut.areg0.aien.has_known_value() == False


def test_reg_dirty_collection():
    dut.add_simple_reg("tr1", 0x1000, reset=0)
    dut.add_simple_reg("tr2", 0x1000, reset=0)
    dut.add_simple_reg("tr3", 0x1000, reset=0)
