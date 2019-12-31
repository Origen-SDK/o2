import origen

def test_registers_are_loaded():
    origen.app.instantiate_dut("dut.falcon")
    assert origen.dut.regs.len() == 4

    origen.app.instantiate_dut("dut.eagle")
    assert origen.dut.regs.len() == 3

def test_sub_blocks_are_loaded():
    origen.app.instantiate_dut("dut.falcon")
    assert origen.dut.sub_blocks["core0"].sub_blocks["adc0"].regs.len() == 1

def test_tree_method():
    # Just a simple test to make sure it doesn't crash and returns
    origen.app.instantiate_dut("dut.falcon")
    assert origen.dut.tree() is None
    assert origen.dut.core0.tree() is None
    assert origen.dut.core0.adc0.tree() is None