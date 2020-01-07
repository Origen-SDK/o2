import origen
import pdb

def test_sub_blocks_can_be_added():
    origen.app.instantiate_dut("dut.falcon")
    assert origen.dut.sub_blocks.len() == 4
    assert list(origen.dut.sub_blocks.keys()) == ['core0', 'core1', 'core2', 'core3']

    # Test adding a sub_block to the top-level
    block = origen.dut.add_sub_block("core4", block_path="core")
    assert origen.dut.sub_blocks.len() == 5
    assert list(origen.dut.sub_blocks.keys()) == ['core0', 'core1', 'core2', 'core3', 'core4']
    assert block.name == "core4"

    # Test adding a sub_block to an embedded block...
    assert origen.dut.core0.adc0.sub_blocks.len() == 0
    assert list(origen.dut.core0.adc0.sub_blocks.keys()) == []
    block = origen.dut.core0.adc0.add_sub_block("my_block", block_path="adc.8_bit")
    assert origen.dut.core0.adc0.sub_blocks.len() == 1
    assert list(origen.dut.core0.adc0.sub_blocks.keys()) == ["my_block"]
    assert block.name == "my_block"

def test_sub_block_iteration():
    origen.app.instantiate_dut("dut.falcon")
    expected = ['core0', 'core1', 'core2', 'core3']
    collected = []
    for name in origen.dut.sub_blocks:
        collected.append(name)
    assert collected == expected
    collected = []
    for name, sub_block in origen.dut.sub_blocks.items():
        collected.append(sub_block.name)
    assert collected == expected