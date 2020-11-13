import origen  # pylint: disable=import-error
from tests.shared import *


def test_registers_are_loaded():
    instantiate_dut("dut.falcon")
    len1 = origen.dut.regs.len()
    instantiate_dut("dut.eagle")
    len2 = origen.dut.regs.len()
    assert len1 > 0
    assert len2 > 0
    assert len1 == len2 + 1


def test_sub_blocks_are_loaded():
    instantiate_dut("dut.falcon")
    assert origen.dut.sub_blocks["core0"].sub_blocks["adc0"].regs.len() == 1


def test_tree_method():
    # Just a simple test to make sure it doesn't crash and returns
    instantiate_dut("dut.falcon")
    assert origen.dut.tree() is None
    assert origen.dut.core0.tree() is None
    assert origen.dut.core0.adc0.tree() is None
