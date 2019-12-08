import origen

def test_registers_are_loaded():
    origen.app.instantiate_block("dut.falcon")
    assert origen.dut.regs.len() == 3

    origen.app.instantiate_block("dut.eagle")
    assert origen.dut.regs.len() == 2

def test_sub_blocks_are_loaded():
    origen.app.instantiate_block("dut.falcon")
    assert origen.dut.sub_blocks["adc0"].regs.len() == 1
