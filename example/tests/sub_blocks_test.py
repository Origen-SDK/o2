import origen
import pytest
import pdb

@pytest.fixture(autouse=True)
def run_around_tests():
    global dut
    # Code that will run before each test
    origen.app.instantiate_dut("dut.falcon")
    dut = origen.dut
    yield
    # Code that will run after each test

def test_sub_blocks_can_be_added():
    assert dut.sub_blocks.len() == 4
    assert list(dut.sub_blocks.keys()) == ['core0', 'core1', 'core2', 'core3']

    # Test adding a sub_block to the top-level
    block = dut.add_sub_block("core4", block_path="core")
    assert dut.sub_blocks.len() == 5
    assert list(dut.sub_blocks.keys()) == ['core0', 'core1', 'core2', 'core3', 'core4']
    assert block.name == "core4"

    # Test adding a sub_block to an embedded block...
    assert dut.core0.adc0.sub_blocks.len() == 0
    assert list(dut.core0.adc0.sub_blocks.keys()) == []
    block = dut.core0.adc0.add_sub_block("my_block", block_path="adc.8_bit")
    assert dut.core0.adc0.sub_blocks.len() == 1
    assert list(dut.core0.adc0.sub_blocks.keys()) == ["my_block"]
    assert block.name == "my_block"

def test_sub_block_iteration():
    expected = ['core0', 'core1', 'core2', 'core3']
    collected = []
    for name in dut.sub_blocks:
        collected.append(name)
    assert collected == expected
    collected = []
    for name, sub_block in dut.sub_blocks.items():
        collected.append(sub_block.name)
    assert collected == expected

def test_sub_block_offset():
    assert dut.core0.offset == 0
    assert dut.core1.offset == 0x1000_0000
    assert dut.core2.offset == 0x2000_0000
    assert dut.core3.offset == 0x3000_0000

def test_sub_block_address_method():
    assert dut.core0.address() == 0
    assert dut.core0.adc0.address() == 0
    assert dut.core0.adc1.address() == 0x1000
    assert dut.core1.address() == 0x1000_0000
    assert dut.core1.adc0.address() == 0x1000_0000
    assert dut.core1.adc1.address() == 0x1000_1000
    assert dut.core2.address() == 0x2000_0000
    assert dut.core2.adc0.address() == 0x2000_0000
    assert dut.core2.adc1.address() == 0x2000_1000
    assert dut.core3.address() == 0x3000_0000
    assert dut.core3.adc0.address() == 0x3000_0000
    assert dut.core3.adc1.address() == 0x3000_1000
    
def test_address_unit_bits():
    pass
