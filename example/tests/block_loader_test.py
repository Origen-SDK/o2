import origen # pylint: disable=import-error

def test_registers_are_loaded():
    origen.app.instantiate_dut("dut.falcon")
    assert origen.dut.regs.len() == 4

    origen.app.instantiate_dut("dut.eagle")
    assert origen.dut.regs.len() == 3

def test_sub_blocks_are_loaded():
    origen.app.instantiate_dut("dut.falcon")
    assert origen.dut.sub_blocks["core0"].sub_blocks["adc0"].regs.len() == 1
